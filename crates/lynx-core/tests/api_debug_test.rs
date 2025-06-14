use anyhow::Result;
use http::StatusCode;
use lynx_db::dao::api_debug_dao::{CreateApiDebugRequest, UpdateApiDebugRequest};
use lynx_db::entities::api_debug::{HttpMethod, RequestStatus};
use serde_json::{Value, json};
use setup::{base_url, setup_self_service_test_server::setup_self_service_test_server};

mod setup;

#[tokio::test]
async fn test_create_and_get_api_debug_entry() -> Result<()> {
    let (server, client) = setup_self_service_test_server().await?;
    let base_url = base_url(&server);

    // Create entry
    let create_request = CreateApiDebugRequest {
        name: "Test API Call".to_string(),
        method: HttpMethod::Get,
        url: "https://httpbin.org/get".to_string(),
        headers: Some(json!({
            "User-Agent": "Lynx-Test"
        })),
        body: None,
        content_type: None,
        timeout: Some(30),
    };

    let response = client
        .get_request_client()
        .post(format!("{}/api_debug/debug", base_url))
        .json(&create_request)
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = response.json().await?;
    assert_eq!(body["code"], "ok");
    assert!(body["data"]["id"].is_number());
    let entry_id = body["data"]["id"].as_i64().unwrap();

    // Get entry
    let get_response = client
        .get_request_client()
        .get(format!("{}/api_debug/debug/{}", base_url, entry_id))
        .send()
        .await?;

    assert_eq!(get_response.status(), StatusCode::OK);
    let get_body: Value = get_response.json().await?;
    assert_eq!(get_body["data"]["name"], "Test API Call");

    Ok(())
}

#[tokio::test]
async fn test_list_api_debug_entries() -> Result<()> {
    let (server, client) = setup_self_service_test_server().await?;
    let base_url = base_url(&server);

    let response = client
        .get_request_client()
        .get(format!("{}/api_debug/debug", base_url))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await?;
    assert_eq!(body["code"], "ok");
    assert!(body["data"]["data"].is_array());

    Ok(())
}

#[tokio::test]
async fn test_update_and_delete_api_debug_entry() -> Result<()> {
    let (server, client) = setup_self_service_test_server().await?;
    let base_url = base_url(&server);

    // Create entry first
    let create_request = CreateApiDebugRequest {
        name: "Update Test".to_string(),
        method: HttpMethod::Get,
        url: "https://httpbin.org/get".to_string(),
        headers: None,
        body: None,
        content_type: None,
        timeout: Some(30),
    };

    let create_response = client
        .get_request_client()
        .post(format!("{}/api_debug/debug", base_url))
        .json(&create_request)
        .send()
        .await?;

    let create_body: Value = create_response.json().await?;
    let entry_id = create_body["data"]["id"].as_i64().unwrap();

    // Update entry
    let update_request = UpdateApiDebugRequest {
        name: Some("Updated Test".to_string()),
        method: None,
        url: None,
        headers: None,
        body: None,
        content_type: None,
        timeout: None,
        status: Some(RequestStatus::Success),
        response_status: Some(200),
        response_headers: None,
        response_body: None,
        response_time: Some(1000),
        error_message: None,
    };

    let update_response = client
        .get_request_client()
        .put(format!("{}/api_debug/debug/{}", base_url, entry_id))
        .json(&update_request)
        .send()
        .await?;

    assert_eq!(update_response.status(), StatusCode::OK);

    // Delete entry
    let delete_response = client
        .get_request_client()
        .delete(format!("{}/api_debug/debug/{}", base_url, entry_id))
        .send()
        .await?;

    assert_eq!(delete_response.status(), StatusCode::OK);

    Ok(())
}
