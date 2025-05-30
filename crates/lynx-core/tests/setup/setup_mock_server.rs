use anyhow::{Ok, Result};

use lynx_mock::server::MockServer;

#[allow(dead_code)]
pub async fn setup_mock_server() -> Result<MockServer> {
    let mut mock_server = MockServer::new(None);
    mock_server.start_server().await?;
    Ok(mock_server)
}
