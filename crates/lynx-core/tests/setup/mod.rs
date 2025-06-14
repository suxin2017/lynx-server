use lynx_core::{proxy_server::ProxyServer, self_service::SELF_SERVICE_PATH_PREFIX};
use lynx_mock::server::MockServer;

pub mod mock_rule;
pub mod setup_api_debug_server;
pub mod setup_mock_server;
pub mod setup_proxy_handler_server;
pub mod setup_proxy_server;
pub mod setup_self_service_test_server;
pub mod setup_tracing;

#[allow(dead_code)]
pub fn base_url(proxy_server: &ProxyServer) -> String {
    format!(
        "http://{}{}",
        proxy_server
            .access_addr_list
            .first()
            .expect("show get access addr"),
        SELF_SERVICE_PATH_PREFIX
    )
}

#[allow(dead_code)]
pub fn mock_base_url(mock_server: &MockServer) -> String {
    format!("http://{}", mock_server.addr)
}
