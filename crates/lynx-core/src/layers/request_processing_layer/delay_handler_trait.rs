use anyhow::Result;
use lynx_db::dao::request_processing_dao::handlers::{DelayHandlerConfig, DelayType};
use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;

use super::handler_trait::{HandleRequestType, HandlerTrait};
use crate::common::Req;

#[async_trait::async_trait]
impl HandlerTrait for DelayHandlerConfig {
    async fn handle_request(&self, request: Req) -> Result<HandleRequestType> {
        // Calculate actual delay duration
        let actual_delay_ms = if let Some(variance) = self.variance_ms {
            let mut rng = rand::thread_rng();
            let min_delay = self.delay_ms.saturating_sub(variance);
            let max_delay = self.delay_ms.saturating_add(variance);
            rng.gen_range(min_delay..=max_delay)
        } else {
            self.delay_ms
        };

        tracing::debug!(
            "Delay handler applying {}ms delay (base: {}ms, variance: {:?}ms, type: {:?})",
            actual_delay_ms,
            self.delay_ms,
            self.variance_ms,
            self.delay_type
        );

        // Apply delay based on delay type
        match self.delay_type {
            DelayType::BeforeRequest => {
                tracing::trace!("Applying delay before request processing");
                sleep(Duration::from_millis(actual_delay_ms)).await;
            }
            DelayType::AfterRequest => {
                tracing::trace!("Delay will be applied after request processing");
                // For AfterRequest, we don't delay here but store the delay info
                // The delay will be applied in the response processing phase
            }
            DelayType::Both => {
                tracing::trace!("Applying half delay before request processing");
                let half_delay = actual_delay_ms / 2;
                sleep(Duration::from_millis(half_delay)).await;
            }
        }

        // Return the original request (delay handlers don't modify the request)
        Ok(HandleRequestType::Request(request))
    }

    async fn handle_response(&self, response: axum::response::Response) -> Result<axum::response::Response> {
        // Apply delay for AfterRequest and Both types
        match self.delay_type {
            DelayType::AfterRequest => {
                let actual_delay_ms = if let Some(variance) = self.variance_ms {
                    let mut rng = rand::thread_rng();
                    let min_delay = self.delay_ms.saturating_sub(variance);
                    let max_delay = self.delay_ms.saturating_add(variance);
                    rng.gen_range(min_delay..=max_delay)
                } else {
                    self.delay_ms
                };

                tracing::trace!("Applying {}ms delay after request processing", actual_delay_ms);
                sleep(Duration::from_millis(actual_delay_ms)).await;
            }
            DelayType::Both => {
                let actual_delay_ms = if let Some(variance) = self.variance_ms {
                    let mut rng = rand::thread_rng();
                    let min_delay = self.delay_ms.saturating_sub(variance);
                    let max_delay = self.delay_ms.saturating_add(variance);
                    rng.gen_range(min_delay..=max_delay)
                } else {
                    self.delay_ms
                };

                let half_delay = actual_delay_ms / 2;
                tracing::trace!("Applying remaining {}ms delay after request processing", half_delay);
                sleep(Duration::from_millis(half_delay)).await;
            }
            DelayType::BeforeRequest => {
                // No delay needed in response phase for BeforeRequest type
            }
        }

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Method;
    use http::Request;
    use std::time::Instant;
    use crate::utils::full;

    #[tokio::test]
    async fn test_delay_handler_before_request() -> Result<()> {
        let handler = DelayHandlerConfig {
            delay_ms: 100,
            variance_ms: None,
            delay_type: DelayType::BeforeRequest,
        };

        let request = Request::builder()
            .method(Method::GET)
            .uri("http://example.com/test")
            .body(full("test request body"))?;

        let start_time = Instant::now();
        let result = handler.handle_request(request).await?;
        let elapsed = start_time.elapsed();

        // Should have delayed for approximately 100ms
        assert!(elapsed >= Duration::from_millis(90)); // Allow some tolerance
        assert!(elapsed <= Duration::from_millis(200)); // Upper bound for CI

        // Should return the original request
        match result {
            HandleRequestType::Request(_) => (),
            HandleRequestType::Response(_) => panic!("Expected request, got response"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_delay_handler_with_variance() -> Result<()> {
        let handler = DelayHandlerConfig {
            delay_ms: 100,
            variance_ms: Some(50),
            delay_type: DelayType::BeforeRequest,
        };

        let request = Request::builder()
            .method(Method::GET)
            .uri("http://example.com/test")
            .body(full("test request body"))?;

        let start_time = Instant::now();
        let result = handler.handle_request(request).await?;
        let elapsed = start_time.elapsed();

        // Should have delayed for 50-150ms (100 ± 50)
        assert!(elapsed >= Duration::from_millis(40)); // Allow some tolerance
        assert!(elapsed <= Duration::from_millis(200)); // Upper bound

        // Should return the original request
        match result {
            HandleRequestType::Request(_) => (),
            HandleRequestType::Response(_) => panic!("Expected request, got response"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_delay_handler_after_request() -> Result<()> {
        let handler = DelayHandlerConfig {
            delay_ms: 50,
            variance_ms: None,
            delay_type: DelayType::AfterRequest,
        };

        let request = Request::builder()
            .method(Method::GET)
            .uri("http://example.com/test")
            .body(full("test request body"))?;

        // handle_request should not delay for AfterRequest type
        let start_time = Instant::now();
        let result = handler.handle_request(request).await?;
        let elapsed = start_time.elapsed();

        // Should not have significant delay
        assert!(elapsed <= Duration::from_millis(10));

        // Should return the original request
        match result {
            HandleRequestType::Request(_) => (),
            HandleRequestType::Response(_) => panic!("Expected request, got response"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_delay_handler_both() -> Result<()> {
        let handler = DelayHandlerConfig {
            delay_ms: 100,
            variance_ms: None,
            delay_type: DelayType::Both,
        };

        let request = Request::builder()
            .method(Method::GET)
            .uri("http://example.com/test")
            .body(full("test request body"))?;

        // handle_request should delay for half the time
        let start_time = Instant::now();
        let result = handler.handle_request(request).await?;
        let elapsed = start_time.elapsed();

        // Should have delayed for approximately 50ms (half of 100ms)
        assert!(elapsed >= Duration::from_millis(40)); // Allow some tolerance
        assert!(elapsed <= Duration::from_millis(80)); // Upper bound

        // Should return the original request
        match result {
            HandleRequestType::Request(_) => (),
            HandleRequestType::Response(_) => panic!("Expected request, got response"),
        }

        Ok(())
    }
}
