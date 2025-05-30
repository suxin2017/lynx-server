use anyhow::{Ok, Result};

use lynx_core::{proxy_server::ProxyServer, self_service::SELF_SERVICE_PATH_PREFIX};
use lynx_mock::client::MockClient;

use super::setup_proxy_server::setup_proxy_server;

#[allow(dead_code)]
pub async fn setup_self_service_test_server() -> Result<(ProxyServer, MockClient)> {
    let proxy_server = setup_proxy_server(None).await?;
    let proxy_server_root_ca = proxy_server.server_ca_manager.ca_cert.clone();
    let client = MockClient::new(Some(vec![proxy_server_root_ca]), None)?;

    Ok((proxy_server, client))
}

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
