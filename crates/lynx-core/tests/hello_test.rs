use anyhow::Result;
use setup::setup_self_service_test_server::{base_url, setup_self_service_test_server};
mod setup;

#[tokio::test]
async fn hello_test() -> Result<()> {
    let (server, client) = setup_self_service_test_server().await?;
    let base_url = base_url(&server);
    let res = client
        .get_request_client()
        .get(format!("{}/health", base_url))
        .send()
        .await?;
    assert_eq!("ok", res.text().await?);

    Ok(())
}
