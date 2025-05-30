use anyhow::Result;

use lynx_db::dao::request_processing_dao::{
    CaptureRule, RequestProcessingDao, RequestRule,
    types::{CaptureCondition, SimpleCaptureCondition},
};
use lynx_db::entities::capture::CaptureType;
use reqwest::{Method, StatusCode};
use serde_json::{Value, json};

use setup::setup_self_service_test_server::{base_url, setup_self_service_test_server};
mod setup;
// Core functionality tests - reduced to essential features

#[tokio::test]
async fn test_list_rules_empty() -> Result<()> {
    let (server, client) = setup_self_service_test_server().await?;
    let base_url = base_url(&server);
    let response = client
        .get_request_client()
        .get(format!("{}/request_processing/rules", base_url))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = response.json().await?;
    assert_eq!(body["code"], "ok");
    assert_eq!(body["data"]["total"], 0);
    assert_eq!(body["data"]["page"], 1);
    assert!(body["data"]["rules"].is_array());

    Ok(())
}

#[tokio::test]
async fn test_get_rule_crud_operations() -> Result<()> {
    let (server, client) = setup_self_service_test_server().await?;
    let base_url = base_url(&server);

    // Create a test rule
    let dao = RequestProcessingDao::new(server.db_connect.clone());
    let rule_id = create_test_rule(&dao, "Test CRUD Rule", true).await?;

    // Test GET rule
    let get_response = client
        .get_request_client()
        .get(format!("{}/request_processing/rules/{}", base_url, rule_id))
        .send()
        .await?;

    assert_eq!(get_response.status(), StatusCode::OK);
    let get_body: Value = get_response.json().await?;
    assert_eq!(get_body["code"], "ok");
    assert_eq!(get_body["data"]["id"], rule_id);
    assert_eq!(get_body["data"]["name"], "Test CRUD Rule");

    // Test DELETE rule
    let delete_response = client
        .get_request_client()
        .request(
            Method::DELETE,
            format!("{}/request_processing/rules/{}", base_url, rule_id),
        )
        .send()
        .await?;

    assert_eq!(delete_response.status(), StatusCode::OK);

    // Verify rule is deleted
    let get_after_delete = client
        .get_request_client()
        .get(format!("{}/request_processing/rules/{}", base_url, rule_id))
        .send()
        .await?;

    assert_eq!(get_after_delete.status(), StatusCode::NOT_FOUND);

    Ok(())
}

#[tokio::test]
async fn test_toggle_rule() -> Result<()> {
    let (server, client) = setup_self_service_test_server().await?;
    let base_url = base_url(&server);

    // Create a disabled test rule
    let dao = RequestProcessingDao::new(server.db_connect.clone());
    let rule_id = create_test_rule(&dao, "Toggle Test Rule", false).await?;

    // Toggle to enabled
    let toggle_request = json!({ "enabled": true });
    let toggle_response = client
        .get_request_client()
        .request(
            Method::PATCH,
            format!("{}/request_processing/rules/{}/toggle", base_url, rule_id),
        )
        .json(&toggle_request)
        .send()
        .await?;

    assert_eq!(toggle_response.status(), StatusCode::OK);

    // Verify the rule is now enabled
    let get_response = client
        .get_request_client()
        .get(format!("{}/request_processing/rules/{}", base_url, rule_id))
        .send()
        .await?;

    let get_body: Value = get_response.json().await?;
    assert_eq!(get_body["data"]["enabled"], true);

    Ok(())
}

#[tokio::test]
async fn test_get_template_handlers() -> Result<()> {
    let (server, client) = setup_self_service_test_server().await?;
    let base_url = base_url(&server);

    let response = client
        .get_request_client()
        .get(format!(
            "{}/request_processing/templates/handlers",
            base_url
        ))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = response.json().await?;
    assert_eq!(body["code"], "ok");
    assert!(body["data"]["handlers"].is_array());

    Ok(())
}

#[tokio::test]
async fn test_list_rules_with_pagination() -> Result<()> {
    let (server, client) = setup_self_service_test_server().await?;
    let base_url = base_url(&server);
    let dao = RequestProcessingDao::new(server.db_connect.clone());

    // Create multiple test rules
    for i in 1..=5 {
        create_test_rule(&dao, &format!("Test Rule {}", i), i % 2 == 0).await?;
    }

    // Test pagination
    let response = client
        .get_request_client()
        .get(format!(
            "{}/request_processing/rules?page=1&pageSize=3",
            base_url
        ))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await?;
    assert_eq!(body["data"]["total"], 5);
    assert_eq!(body["data"]["page"], 1);
    assert_eq!(body["data"]["pageSize"], 3);
    assert_eq!(body["data"]["rules"].as_array().unwrap().len(), 3);

    Ok(())
}

#[tokio::test]
async fn test_list_rules_enabled_only() -> Result<()> {
    let (server, client) = setup_self_service_test_server().await?;
    let base_url = base_url(&server);
    let dao = RequestProcessingDao::new(server.db_connect.clone());

    // Create mixed enabled/disabled rules
    create_test_rule(&dao, "Enabled Rule 1", true).await?;
    create_test_rule(&dao, "Disabled Rule", false).await?;
    create_test_rule(&dao, "Enabled Rule 2", true).await?;

    // Test enabled only filter
    let response = client
        .get_request_client()
        .get(format!(
            "{}/request_processing/rules?enabledOnly=true",
            base_url
        ))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await?;
    assert_eq!(body["data"]["total"], 2);

    let rules = body["data"]["rules"].as_array().unwrap();
    for rule in rules {
        assert_eq!(rule["enabled"], true);
    }

    Ok(())
}

#[tokio::test]
async fn test_rule_not_found_scenarios() -> Result<()> {
    let (server, client) = setup_self_service_test_server().await?;
    let base_url = base_url(&server);

    let non_existent_id = 99999;

    // Test GET non-existent rule
    let get_response = client
        .get_request_client()
        .get(format!(
            "{}/request_processing/rules/{}",
            base_url, non_existent_id
        ))
        .send()
        .await?;
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);

    // Test DELETE non-existent rule
    let delete_response = client
        .get_request_client()
        .request(
            Method::DELETE,
            format!("{}/request_processing/rules/{}", base_url, non_existent_id),
        )
        .send()
        .await?;
    assert_eq!(delete_response.status(), StatusCode::NOT_FOUND);

    // Test TOGGLE non-existent rule
    let toggle_request = json!({ "enabled": true });
    let toggle_response = client
        .get_request_client()
        .request(
            Method::PATCH,
            format!(
                "{}/request_processing/rules/{}/toggle",
                base_url, non_existent_id
            ),
        )
        .json(&toggle_request)
        .send()
        .await?;
    assert_eq!(toggle_response.status(), StatusCode::NOT_FOUND);

    Ok(())
}

#[tokio::test]
async fn test_toggle_rule_disable() -> Result<()> {
    let (server, client) = setup_self_service_test_server().await?;
    let base_url = base_url(&server);
    let dao = RequestProcessingDao::new(server.db_connect.clone());

    // Create an enabled test rule
    let rule_id = create_test_rule(&dao, "Enable to Disable Rule", true).await?;

    // Toggle to disabled
    let toggle_request = json!({ "enabled": false });
    let toggle_response = client
        .get_request_client()
        .request(
            Method::PATCH,
            format!("{}/request_processing/rules/{}/toggle", base_url, rule_id),
        )
        .json(&toggle_request)
        .send()
        .await?;

    assert_eq!(toggle_response.status(), StatusCode::OK);

    // Verify the rule is now disabled
    let get_response = client
        .get_request_client()
        .get(format!("{}/request_processing/rules/{}", base_url, rule_id))
        .send()
        .await?;

    let get_body: Value = get_response.json().await?;
    assert_eq!(get_body["data"]["enabled"], false);

    Ok(())
}

#[tokio::test]
async fn test_list_rules_with_large_page_size() -> Result<()> {
    let (server, client) = setup_self_service_test_server().await?;
    let base_url = base_url(&server);
    let dao = RequestProcessingDao::new(server.db_connect.clone());

    // Create a few test rules
    create_test_rule(&dao, "Rule A", true).await?;
    create_test_rule(&dao, "Rule B", false).await?;

    // Test with page size larger than total items
    let response = client
        .get_request_client()
        .get(format!(
            "{}/request_processing/rules?page=1&pageSize=100",
            base_url
        ))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await?;
    assert_eq!(body["data"]["total"], 2);
    assert_eq!(body["data"]["rules"].as_array().unwrap().len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_template_handlers_response_structure() -> Result<()> {
    let (server, client) = setup_self_service_test_server().await?;
    let base_url = base_url(&server);

    let response = client
        .get_request_client()
        .get(format!(
            "{}/request_processing/templates/handlers",
            base_url
        ))
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await?;

    // Verify the response structure matches expected format
    assert_eq!(body["code"], "ok");
    assert!(body["data"]["handlers"].is_array());

    // Verify response structure
    assert!(body["data"].is_object());
    let data = &body["data"];
    assert!(data.get("handlers").is_some());

    Ok(())
}

// Helper functions

async fn create_test_rule(dao: &RequestProcessingDao, name: &str, enabled: bool) -> Result<i32> {
    let rule = RequestRule {
        id: None,
        name: name.to_string(),
        description: Some("Test rule description".to_string()),
        enabled,
        priority: 1,
        capture: create_basic_capture_rule(),
        handlers: vec![],
    };

    dao.create_rule(rule).await
}

fn create_basic_capture_rule() -> CaptureRule {
    CaptureRule {
        id: None,
        condition: CaptureCondition::Simple(SimpleCaptureCondition {
            capture_type: CaptureType::Glob,
            pattern: "/api/*".to_string(),
            method: Some("GET".to_string()),
            host: None,
            config: json!({}),
        }),
    }
}
