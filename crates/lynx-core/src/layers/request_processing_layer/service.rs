use super::handler_trait::{HandleRequestType, HandlerTrait};
use crate::{
    common::Req,
    layers::{extend_extension_layer::DbExtensionsExt, trace_id_layer::service::TraceIdExt},
    utils::full,
};
use anyhow::Result;
use axum::response::{IntoResponse, Response};
use lynx_db::dao::request_processing_dao::{
    RequestProcessingDao, handlers::handler_rule::HandlerRuleType,
};
use std::{future::Future, pin::Pin, task::Poll};
use tower::Service;
use tracing::{Instrument, trace_span};

#[derive(Clone)]
pub struct RequestProcessingService<S> {
    pub service: S,
}

impl<S> RequestProcessingService<S> {
    pub fn new(service: S) -> Self {
        Self { service }
    }
}

impl<S> Service<Req> for RequestProcessingService<S>
where
    S: Service<Req, Future: Future + Send + 'static, Response = Response, Error = anyhow::Error>
        + Clone
        + Send
        + Sync
        + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, request: Req) -> Self::Future {
        let span = trace_span!("request_processing_layer");

        let db: std::sync::Arc<sea_orm::DatabaseConnection> = request.extensions().get_db();
        let trace_id = request.extensions().get_trace_id();

        let mut inner = self.service.clone();
        Box::pin(
            async move {
                tracing::trace!(
                    "Starting request processing, method: {}, uri: {}",
                    request.method(),
                    request.uri()
                );

                let dao = RequestProcessingDao::new(db.clone());
                // 查找匹配的规则
                tracing::trace!("Searching for matching rules for request");
                let matching_rules = match dao.find_matching_rules(&request).await {
                    Ok(rules) => {
                        tracing::trace!("Found {} matching rules", rules.len());
                        rules
                    }
                    Err(e) => {
                        tracing::warn!("Failed to find matching rules: {}", e);
                        tracing::trace!("Bypassing request processing due to rule lookup failure");
                        // 如果查找规则失败，直接继续处理请求
                        return inner.call(request).await;
                    }
                };

                if matching_rules.is_empty() {
                    tracing::trace!("No matching rules found, proceeding with original request");
                    return inner.call(request).await;
                }

                let mut all_handlers = Vec::new();

                for rule in &matching_rules {
                    tracing::trace!(
                        "Processing rule: '{}', enabled: {}",
                        rule.name,
                        rule.enabled
                    );
                    if rule.enabled {
                        for handler in &rule.handlers {
                            tracing::trace!(
                                "Processing handler: '{}', enabled: {}, execution_order: {}",
                                handler.name,
                                handler.enabled,
                                handler.execution_order
                            );
                            if handler.enabled {
                                all_handlers.push(handler);
                            }
                        }
                    }
                }

                // 按执行顺序排序
                all_handlers.sort_by_key(|h| h.execution_order);
                tracing::trace!(
                    "Collected {} enabled handlers for execution",
                    all_handlers.len()
                );

                // 执行处理器
                let mut current_request = request;

                for (index, handler) in all_handlers.iter().enumerate() {
                    tracing::trace!(
                        "Executing handler {}/{}: '{}' (type: {:?})",
                        index + 1,
                        all_handlers.len(),
                        handler.name,
                        handler.handler_type
                    );

                    let handler_result = match &handler.handler_type {
                        HandlerRuleType::Block(block_handler_config) => {
                            tracing::trace!("Executing block handler for '{}'", handler.name);
                            block_handler_config.handle_request(current_request).await
                        }
                        HandlerRuleType::LocalFile(local_file_config) => {
                            tracing::trace!("Executing local file handler for '{}'", handler.name);
                            local_file_config.handle_request(current_request).await
                        }
                        HandlerRuleType::ModifyRequest(modify_request_config) => {
                            tracing::trace!(
                                "Executing modify request handler for '{}'",
                                handler.name
                            );
                            modify_request_config.handle_request(current_request).await
                        }
                        HandlerRuleType::ModifyResponse(modify_response_config) => {
                            tracing::trace!(
                                "Executing modify response handler for '{}'",
                                handler.name
                            );
                            modify_response_config.handle_request(current_request).await
                        }
                        HandlerRuleType::ProxyForward(proxy_forward_config) => {
                            tracing::trace!(
                                "Executing proxy forward handler for '{}'",
                                handler.name
                            );
                            proxy_forward_config.handle_request(current_request).await
                        }
                        HandlerRuleType::HtmlScriptInjector(html_script_injector_config) => {
                            tracing::trace!(
                                "Executing HTML script injector handler for '{}'",
                                handler.name
                            );
                            html_script_injector_config.handle_request(current_request).await
                        }
                        HandlerRuleType::Delay(delay_config) => {
                            tracing::trace!(
                                "Executing delay handler for '{}' (delay: {}ms, variance: {:?}ms)",
                                handler.name,
                                delay_config.delay_ms,
                                delay_config.variance_ms
                            );
                            delay_config.handle_request(current_request).await
                        }
                    };

                    match handler_result {
                        Ok(HandleRequestType::Request(req)) => {
                            tracing::trace!(
                                "Handler '{}' modified the request, continuing with next handler",
                                handler.name
                            );
                            current_request = req;
                        }
                        Ok(HandleRequestType::Response(mut response)) => {
                            tracing::trace!(
                                "Handler '{}' returned a response (status: {}), short-circuiting",
                                handler.name,
                                response.status()
                            );
                            response.extensions_mut().insert(trace_id.clone());
                            // 如果处理器返回响应，直接返回该响应
                            return Ok(response);
                        }
                        Err(e) => {
                            tracing::warn!("Handler '{}' failed: {}", handler.name, e);
                            tracing::trace!("Creating error response for handler failure");
                            // 如果处理器失败，我们需要创建一个错误响应
                            let error_response = Response::builder()
                                .status(500)
                                .header("content-type", "text/plain")
                                .body(full(format!("Handler processing failed: {}", e)))
                                .unwrap_or_else(|_| Response::new(full("Internal server error")));
                            return Ok(error_response.into_response());
                        }
                    }
                }

                tracing::trace!(
                    "All handlers executed successfully, proceeding with modified request"
                );
                // 所有处理器执行完毕后，继续处理请求
                let mut response = inner.call(current_request).await?;

                if !all_handlers.is_empty() {
                    tracing::trace!(
                        "Processing response with {} response handlers",
                        all_handlers.len()
                    );

                    for (index, handler) in all_handlers.iter().enumerate() {
                        tracing::trace!(
                            "Executing response handler {}/{}: '{}' (type: {:?})",
                            index + 1,
                            all_handlers.len(),
                            handler.name,
                            handler.handler_type
                        );

                        match &handler.handler_type {
                            HandlerRuleType::ModifyResponse(modify_response_config) => {
                                tracing::trace!(
                                    "Executing modify response handler for '{}'",
                                    handler.name
                                );
                                response = modify_response_config.handle_response(response).await?;
                            }
                            HandlerRuleType::HtmlScriptInjector(html_script_injector_config) => {
                                tracing::trace!(
                                    "Executing HTML script injector response handler for '{}'",
                                    handler.name
                                );
                                response = html_script_injector_config.handle_response(response).await?;
                            }
                            HandlerRuleType::Delay(delay_config) => {
                                tracing::trace!(
                                    "Executing delay response handler for '{}' (type: {:?})",
                                    handler.name,
                                    delay_config.delay_type
                                );
                                response = delay_config.handle_response(response).await?;
                            }
                            _ => {
                                tracing::trace!(
                                    "Handler type '{}' does not support response processing",
                                    handler.name
                                );
                                continue;
                            }
                        };
                    }
                }

                // 确保响应包含 trace_id
                response.extensions_mut().insert(trace_id.clone());

                Ok(response)
            }
            .instrument(span),
        )
    }
}
