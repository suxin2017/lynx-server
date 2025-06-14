use anyhow::{Ok, Result};
use lynx_db::dao::https_capture_dao::{CaptureFilter, HttpsCaptureDao};
use lynx_mock::client::MockClient;
use setup::{setup_mock_server::setup_mock_server, setup_proxy_server::setup_proxy_server};
use std::sync::Arc;
mod setup;

#[tokio::test]
async fn proxy_test() -> Result<()> {
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
    client.test_request_http_request(&mock_server).await?;
    client.test_request_https_request(&mock_server).await?;
    client.test_request_websocket(&mock_server).await?;
    client.test_request_tls_websocket(&mock_server).await?;

    Ok(())
}

#[tokio::test]
#[ignore = "need stable network connect"]
async fn test_real_world_request() -> Result<()> {
    let mock_server = setup_mock_server().await?;
    let proxy_server = setup_proxy_server(Some(Arc::new(vec![mock_server.cert.clone()]))).await?;
    let proxy_server_root_ca = proxy_server.server_ca_manager.ca_cert.clone();

    let proxy_addr = format!("http://{}", proxy_server.access_addr_list.first().unwrap());

    let client = MockClient::new(Some(vec![proxy_server_root_ca]), Some(proxy_addr))?;
    client.test_real_world_http_request().await?;
    client.test_real_world_https_request().await?;
    // FIXME: The websocket test is not working due to the server not being able to handle the request.
    // client.test_real_world_websocket_request().await?;
    // client.test_real_world_tls_websocket_request().await?;
    Ok(())
}
