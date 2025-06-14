use anyhow::Result;
use http::StatusCode;
use serde_json::{Value, json};
use setup::{base_url, mock_base_url, setup_api_debug_server::setup_api_debug_server};

mod setup;

#[tokio::test]
async fn test_api_debug_executor_with_proxy_handler_server() -> Result<()> {
    let (proxy_server, mock_server, client) = setup_api_debug_server().await?;
    let base_url = base_url(&proxy_server);
    let mock_base_url = mock_base_url(&mock_server);

    // Create an execute request targeting the mock server
    let execute_request = json!({
        "name": "Test Mock Server Request",
        "method": "GET",
        "url": format!("{}/hello", mock_base_url),
        "headers": {
            "User-Agent": "Lynx-Test-Executor"
        },
        "timeout": 30
    });

    let response = client
        .get_request_client()
        .post(format!("{}/api_debug_executor/execute", base_url))
        .json(&execute_request)
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await?;
    assert_eq!(body["code"], "ok");
    assert!(body["data"]["id"].is_number());
    let entry_id = body["data"]["id"].as_i64().unwrap();
    let status = body["data"]["status"].as_str().unwrap();
    let response_status = body["data"]["responseStatus"].as_i64().unwrap();
    let response_body = body["data"]["responseBody"].as_str().unwrap();

    // Verify the execution was successful
    assert_eq!(status, "success");
    assert_eq!(response_status, 200);
    assert_eq!(response_body, "Hello, World!");

    // Verify the entry was created in database
    let get_response = client
        .get_request_client()
        .get(format!("{}/api_debug/debug/{}", base_url, entry_id))
        .send()
        .await?;

    assert_eq!(get_response.status(), StatusCode::OK);
    let get_body: Value = get_response.json().await?;
    assert_eq!(get_body["data"]["name"], "Test Mock Server Request");
    assert_eq!(get_body["data"]["url"], format!("{}/hello", mock_base_url));

    Ok(())
}

#[tokio::test]
async fn test_api_debug_executor_with_different_http_methods() -> Result<()> {
    let (proxy_server, mock_server, client) = setup_api_debug_server().await?;
    let base_url = base_url(&proxy_server);
    let mock_base_url = mock_base_url(&mock_server);

    // Test POST request to post_echo endpoint
    let execute_request = json!({
        "name": "Test POST Echo",
        "method": "POST",
        "url": format!("{}/post_echo", mock_base_url),
        "headers": {
            "Content-Type": "application/json"
        },
        "body": r#"{"test": "data"}"#,
        "timeout": 30
    });

    let response = client
        .get_request_client()
        .post(format!("{}/api_debug_executor/execute", base_url))
        .json(&execute_request)
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = response.json().await?;
    assert_eq!(body["code"], "ok");
    let status = body["data"]["status"].as_str().unwrap();
    let response_status = body["data"]["responseStatus"].as_i64().unwrap();
    let response_body = body["data"]["responseBody"].as_str().unwrap();

    // Verify the execution was successful
    assert_eq!(status, "success");
    assert_eq!(response_status, 200);
    // The post_echo endpoint should return the request body
    assert_eq!(response_body, r#"{"test": "data"}"#);

    Ok(())
}

#[tokio::test]
async fn test_api_debug_executor_error_handling() -> Result<()> {
    let (proxy_server, mock_server, client) = setup_api_debug_server().await?;
    let base_url = base_url(&proxy_server);
    let mock_base_url = mock_base_url(&mock_server);

    // Create an execute request to a non-existent endpoint on mock server
    let execute_request = json!({
        "name": "Test Non-existent Endpoint",
        "method": "GET",
        "url": format!("{}/non-existent-endpoint", mock_base_url),
        "timeout": 5
    });

    let response = client
        .get_request_client()
        .post(format!("{}/api_debug_executor/execute", base_url))
        .json(&execute_request)
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await?;
    println!("Response body: {}", body);
    assert_eq!(body["code"], "ok");

    let status = body["data"]["status"].as_str().unwrap();
    let response_status = body["data"]["responseStatus"].as_i64();

    // Verify the execution failed due to 404 response
    assert_eq!(status, "failed");
    // For 404, we get a response status but it's considered failed
    assert_eq!(response_status, Some(404));
    // For HTTP errors like 404, errorMessage might be null as it's a valid HTTP response

    Ok(())
}

#[tokio::test]
async fn test_api_debug_executor_with_custom_headers() -> Result<()> {
    let (proxy_server, mock_server, client) = setup_api_debug_server().await?;
    let base_url = base_url(&proxy_server);
    let mock_base_url = mock_base_url(&mock_server);

    // Test request with custom headers
    let execute_request = json!({
        "name": "Test Custom Headers",
        "method": "GET",
        "url": format!("{}/headers", mock_base_url),
        "headers": {
            "X-Custom-Header": "Custom-Value",
            "Authorization": "Bearer test-token"
        },
        "timeout": 30
    });

    let response = client
        .get_request_client()
        .post(format!("{}/api_debug_executor/execute", base_url))
        .json(&execute_request)
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await?;
    assert_eq!(body["code"], "ok");

    let status = body["data"]["status"].as_str().unwrap();
    let response_status = body["data"]["responseStatus"].as_i64().unwrap();

    // Verify the execution was successful
    assert_eq!(status, "success");
    assert_eq!(response_status, 200);

    Ok(())
}

#[tokio::test]
async fn test_api_debug_executor_timeout_handling() -> Result<()> {
    let (proxy_server, mock_server, client) = setup_api_debug_server().await?;
    let base_url = base_url(&proxy_server);
    let mock_base_url = mock_base_url(&mock_server);

    // Test request with short timeout to slow endpoint (3 second delay)
    let execute_request = json!({
        "name": "Test Timeout",
        "method": "GET",
        "url": format!("{}/slow", mock_base_url),
        "timeout": 1  // 1 second timeout
    });

    let response = client
        .get_request_client()
        .post(format!("{}/api_debug_executor/execute", base_url))
        .json(&execute_request)
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await?;
    assert_eq!(body["code"], "ok");

    let status = body["data"]["status"].as_str().unwrap();
    let error_message = body["data"]["errorMessage"].as_str(); // Verify the execution failed due to timeout
    assert_eq!(status, "failed");
    assert!(error_message.is_some());
    // The error message should indicate a request failure
    if let Some(error) = error_message {
        println!("Error message: {}", error);
        assert!(
            error.contains("error sending request")
                || error.to_lowercase().contains("timeout")
                || error.to_lowercase().contains("time")
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_api_debug_executor_json_endpoint() -> Result<()> {
    let (proxy_server, mock_server, client) = setup_api_debug_server().await?;
    let base_url = base_url(&proxy_server);
    let mock_base_url = mock_base_url(&mock_server);

    // Test JSON endpoint
    let execute_request = json!({
        "name": "Test JSON Endpoint",
        "method": "GET",
        "url": format!("{}/json", mock_base_url),
        "timeout": 30
    });

    let response = client
        .get_request_client()
        .post(format!("{}/api_debug_executor/execute", base_url))
        .json(&execute_request)
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = response.json().await?;
    assert_eq!(body["code"], "ok");
    let status = body["data"]["status"].as_str().unwrap();
    let response_status = body["data"]["responseStatus"].as_i64().unwrap();
    let response_body = body["data"]["responseBody"].as_str().unwrap();

    // Verify the execution was successful
    assert_eq!(status, "success");
    assert_eq!(response_status, 200);

    // Parse and verify the JSON response
    let json_response: Value = serde_json::from_str(response_body)?;
    assert_eq!(json_response["message"], "Hello from JSON endpoint");
    assert!(json_response["timestamp"].is_number());
    assert!(json_response["data"]["success"].as_bool().unwrap());

    Ok(())
}

#[tokio::test]
async fn test_api_debug_executor_status_endpoint() -> Result<()> {
    let (proxy_server, mock_server, client) = setup_api_debug_server().await?;
    let base_url = base_url(&proxy_server);
    let mock_base_url = mock_base_url(&mock_server);

    // Test status endpoint with custom status code
    let execute_request = json!({
        "name": "Test Status Code",
        "method": "GET",
        "url": format!("{}/status?code=201", mock_base_url),
        "timeout": 30
    });

    let response = client
        .get_request_client()
        .post(format!("{}/api_debug_executor/execute", base_url))
        .json(&execute_request)
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = response.json().await?;
    assert_eq!(body["code"], "ok");

    let status = body["data"]["status"].as_str().unwrap();
    let response_status = body["data"]["responseStatus"].as_i64().unwrap();
    let response_body = body["data"]["responseBody"].as_str().unwrap(); // Verify the execution was successful but returned the requested status code
    assert_eq!(status, "success");
    assert_eq!(response_status, 201);
    assert!(response_body.contains("Status: 201"));

    Ok(())
}
