use std::sync::Arc;

use anyhow::{Ok, Result};

use lynx_core::proxy_server::ProxyServer;
use lynx_db::dao::https_capture_dao::{CaptureFilter, HttpsCaptureDao};
use lynx_mock::{client::MockClient, server::MockServer};

use super::{setup_mock_server::setup_mock_server, setup_proxy_server::setup_proxy_server};

#[allow(dead_code)]
pub async fn setup_api_debug_server() -> Result<(ProxyServer, MockServer, MockClient)> {
    let mock_server = setup_mock_server().await?;
    let proxy_server = setup_proxy_server(Some(Arc::new(vec![mock_server.cert.clone()]))).await?;
    let proxy_server_root_ca = proxy_server.server_ca_manager.ca_cert.clone();

    HttpsCaptureDao::new(proxy_server.db_connect.clone())
        .update_capture_filter(CaptureFilter {
            enabled: true,
            include_domains: vec![],
            exclude_domains: vec![],
        })
        .await?;

    let proxy_addr = format!("http://{}", proxy_server.access_addr_list.first().unwrap());

    let client = MockClient::new(
        Some(vec![mock_server.cert.clone(), proxy_server_root_ca]),
        Some(proxy_addr),
    )?;
    Ok((proxy_server, mock_server, client))
}
