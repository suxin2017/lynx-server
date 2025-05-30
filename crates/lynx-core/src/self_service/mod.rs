use std::net::SocketAddr;
use std::sync::Arc;

use crate::common::Req;
use crate::layers::extend_extension_layer::DbExtensionsExt;
use crate::layers::message_package_layer::message_event_store::MessageEventCache;
use crate::layers::message_package_layer::message_event_store::MessageEventStoreExtensionsExt;
use crate::proxy_server::StaticDir;
use crate::proxy_server::server_config::ProxyServerConfig;
use crate::proxy_server::server_config::ProxyServerConfigExtensionsExt;
use anyhow::Result;
use api::{base_info, net_request};
use axum::Router;
use axum::response::Response;
use file_service::get_file;
use http::Method;
use tower::ServiceExt;
use utoipa::openapi::OpenApi;
use utoipa::openapi::Server;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;
pub mod api;
pub mod file_service;
pub mod utils;
use tower_http::cors::{Any, CorsLayer};

pub const SELF_SERVICE_PATH_PREFIX: &str = "/api";

pub fn is_self_service(req: &Req) -> bool {
    let access_addr_list = req.extensions().get::<Arc<Vec<SocketAddr>>>();

    access_addr_list
        .map(|list| {
            list.iter().any(|addr| {
                req.headers().get("host").is_some_and(|host| {
                    if let Ok(host) = host.to_str() {
                        let host_ip = if host.starts_with("localhost") {
                            "127.0.0.1"
                        } else {
                            host.split(':').next().unwrap_or(host)
                        };
                        let host_port = if host.contains(':') {
                            host.split(':').nth(1).unwrap_or("80")
                        } else {
                            "80"
                        };
                        let host = format!("{}:{}", host_ip, host_port);
                        host == addr.to_string()
                    } else {
                        false
                    }
                })
            })
        })
        .unwrap_or(false)
}

#[utoipa::path(get,  path = "/health", responses((status = OK, body = String)))]
async fn get_health() -> &'static str {
    "ok"
}

#[derive(Clone, Debug)]
pub struct RouteState {
    pub db: Arc<sea_orm::DatabaseConnection>,
    pub net_request_cache: Arc<MessageEventCache>,
    pub proxy_config: Arc<ProxyServerConfig>,
    pub access_addr_list: Arc<Vec<SocketAddr>>,
    pub static_dir: Option<Arc<StaticDir>>,
}

pub async fn self_service_router(req: Req) -> Result<Response> {
    let static_dir = req.extensions().get::<Option<Arc<StaticDir>>>();

    let state = RouteState {
        db: req.extensions().get_db(),
        net_request_cache: req.extensions().get_message_event_store(),
        proxy_config: req.extensions().get_proxy_server_config(),
        access_addr_list: req
            .extensions()
            .get::<Arc<Vec<SocketAddr>>>()
            .expect("access_addr_list not found")
            .clone(),
        static_dir: static_dir.cloned().flatten(),
    };
    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_origin(Any);

    let (router, mut openapi): (axum::Router, OpenApi) = OpenApiRouter::new()
        .routes(routes!(get_health))
        .layer(cors)
        .with_state(state.clone())
        .nest("/net_request", net_request::router(state.clone()))
        .nest("/certificate", api::certificate::router(state.clone()))
        .nest("/base_info", base_info::router(state.clone()))
        .nest("/https_capture", api::https_capture::router(state.clone()))
        .nest(
            "/request_processing",
            api::request_processing::router(state.clone()),
        )
        .split_for_parts();

    openapi.servers = Some(vec![Server::new(SELF_SERVICE_PATH_PREFIX)]);
    let swagger_path = format!("{}/swagger-ui", SELF_SERVICE_PATH_PREFIX);
    let api_docs_path = format!("{}/api-docs/openapi.json", SELF_SERVICE_PATH_PREFIX);

    let swagger_router =
        Router::new().merge(SwaggerUi::new(swagger_path).url(api_docs_path, openapi));

    let router = Router::new()
        .fallback(get_file)
        .with_state(state.clone())
        .nest(SELF_SERVICE_PATH_PREFIX, router)
        .merge(swagger_router);

    let router = router;

    router
        .oneshot(req)
        .await
        .map_err(|e| anyhow::anyhow!(e).context("Error handling request"))
}
