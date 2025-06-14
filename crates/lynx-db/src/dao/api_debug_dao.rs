use crate::entities::api_debug::{self, ActiveModel, Entity, HttpMethod, Model, RequestStatus};
use anyhow::Result;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use utoipa::ToSchema;

/// Request for creating a new API debug entry
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiDebugRequest {
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    pub headers: Option<JsonValue>,
    pub body: Option<String>,
    pub content_type: Option<String>,
    pub timeout: Option<i32>,
}

/// Request for updating an API debug entry
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateApiDebugRequest {
    pub name: Option<String>,
    pub method: Option<HttpMethod>,
    pub url: Option<String>,
    pub headers: Option<JsonValue>,
    pub body: Option<String>,
    pub content_type: Option<String>,
    pub timeout: Option<i32>,
    pub status: Option<RequestStatus>,
    pub response_status: Option<i32>,
    pub response_headers: Option<JsonValue>,
    pub response_body: Option<String>,
    pub response_time: Option<i32>,
    pub error_message: Option<String>,
}

/// Response for API debug operations
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiDebugResponse {
    pub id: i32,
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    pub headers: Option<JsonValue>,
    pub body: Option<String>,
    pub content_type: Option<String>,
    pub timeout: Option<i32>,
    pub status: RequestStatus,
    pub response_status: Option<i32>,
    pub response_headers: Option<JsonValue>,
    pub response_body: Option<String>,
    pub response_time: Option<i32>,
    pub error_message: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<Model> for ApiDebugResponse {
    fn from(model: Model) -> Self {
        let body_string = model.get_body_as_string();
        let response_body_string = model.get_response_body_as_string();

        Self {
            id: model.id,
            name: model.name,
            method: model.method,
            url: model.url,
            headers: model.headers,
            body: body_string,
            content_type: model.content_type,
            timeout: model.timeout,
            status: model.status,
            response_status: model.response_status,
            response_headers: model.response_headers,
            response_body: response_body_string,
            response_time: model.response_time,
            error_message: model.error_message,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl From<CreateApiDebugRequest> for ActiveModel {
    fn from(req: CreateApiDebugRequest) -> Self {
        let now = chrono::Utc::now().timestamp();
        ActiveModel {
            id: NotSet,
            name: Set(req.name),
            method: Set(req.method),
            url: Set(req.url),
            headers: Set(req.headers),
            body: Set(req.body.map(|s| s.into_bytes())),
            content_type: Set(req.content_type),
            timeout: Set(req.timeout),
            status: Set(RequestStatus::Pending),
            response_status: NotSet,
            response_headers: NotSet,
            response_body: NotSet,
            response_time: NotSet,
            error_message: NotSet,
            created_at: Set(now),
            updated_at: Set(now),
        }
    }
}

/// Query parameters for listing API debug entries
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiDebugQueryParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub method: Option<HttpMethod>,
    pub status: Option<RequestStatus>,
    pub search: Option<String>,
}

/// Paginated response for API debug entries
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiDebugListResponse {
    pub data: Vec<ApiDebugResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

pub struct ApiDebugDao {
    db: Arc<DatabaseConnection>,
}

impl ApiDebugDao {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Create a new API debug entry
    pub async fn create(&self, req: CreateApiDebugRequest) -> Result<ApiDebugResponse> {
        let active_model: ActiveModel = req.into();
        let model = Entity::insert(active_model)
            .exec_with_returning(self.db.as_ref())
            .await?;

        Ok(model.into())
    }

    /// Get API debug entry by ID
    pub async fn get_by_id(&self, id: i32) -> Result<Option<ApiDebugResponse>> {
        let model = Entity::find_by_id(id).one(self.db.as_ref()).await?;

        Ok(model.map(|m| m.into()))
    }

    /// Update API debug entry
    pub async fn update(
        &self,
        id: i32,
        req: UpdateApiDebugRequest,
    ) -> Result<Option<ApiDebugResponse>> {
        let model = Entity::find_by_id(id).one(self.db.as_ref()).await?;

        if let Some(model) = model {
            let mut active_model: ActiveModel = model.into();

            if let Some(name) = req.name {
                active_model.name = Set(name);
            }
            if let Some(method) = req.method {
                active_model.method = Set(method);
            }
            if let Some(url) = req.url {
                active_model.url = Set(url);
            }
            if let Some(headers) = req.headers {
                active_model.headers = Set(Some(headers));
            }
            if let Some(body) = req.body {
                active_model.body = Set(Some(body.into_bytes()));
            }
            if let Some(content_type) = req.content_type {
                active_model.content_type = Set(Some(content_type));
            }
            if let Some(timeout) = req.timeout {
                active_model.timeout = Set(Some(timeout));
            }
            if let Some(status) = req.status {
                active_model.status = Set(status);
            }
            if let Some(response_status) = req.response_status {
                active_model.response_status = Set(Some(response_status));
            }
            if let Some(response_headers) = req.response_headers {
                active_model.response_headers = Set(Some(response_headers));
            }
            if let Some(response_body) = req.response_body {
                active_model.response_body = Set(Some(response_body.into_bytes()));
            }
            if let Some(response_time) = req.response_time {
                active_model.response_time = Set(Some(response_time));
            }
            if let Some(error_message) = req.error_message {
                active_model.error_message = Set(Some(error_message));
            }

            // Always update the updated_at timestamp
            active_model.updated_at = Set(chrono::Utc::now().timestamp());

            let updated_model = Entity::update(active_model).exec(self.db.as_ref()).await?;

            Ok(Some(updated_model.into()))
        } else {
            Ok(None)
        }
    }

    /// Delete API debug entry by ID
    pub async fn delete(&self, id: i32) -> Result<bool> {
        let result = Entity::delete_by_id(id).exec(self.db.as_ref()).await?;

        Ok(result.rows_affected > 0)
    }

    /// List API debug entries with pagination and filtering
    pub async fn list(&self, params: ApiDebugQueryParams) -> Result<ApiDebugListResponse> {
        let page = params.page.unwrap_or(1);
        let per_page = params.per_page.unwrap_or(20).min(100); // Max 100 per page

        let mut query = Entity::find();

        // Apply filters
        if let Some(method) = params.method {
            query = query.filter(api_debug::Column::Method.eq(method));
        }

        if let Some(status) = params.status {
            query = query.filter(api_debug::Column::Status.eq(status));
        }

        if let Some(search) = params.search {
            query = query.filter(
                Condition::any()
                    .add(api_debug::Column::Name.contains(&search))
                    .add(api_debug::Column::Url.contains(&search)),
            );
        }

        // Get total count
        let total = query.clone().count(self.db.as_ref()).await?;

        // Apply pagination and ordering
        let models = query
            .order_by_desc(api_debug::Column::CreatedAt)
            .paginate(self.db.as_ref(), per_page)
            .fetch_page(page - 1)
            .await?;

        let data = models.into_iter().map(|m| m.into()).collect();
        let total_pages = (total + per_page - 1) / per_page;

        Ok(ApiDebugListResponse {
            data,
            total,
            page,
            per_page,
            total_pages,
        })
    }

    /// Get recent API debug entries
    pub async fn get_recent(&self, limit: u64) -> Result<Vec<ApiDebugResponse>> {
        let models = Entity::find()
            .order_by_desc(api_debug::Column::CreatedAt)
            .limit(limit)
            .all(self.db.as_ref())
            .await?;

        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    /// Get API debug entries by status
    pub async fn get_by_status(&self, status: RequestStatus) -> Result<Vec<ApiDebugResponse>> {
        let models = Entity::find()
            .filter(api_debug::Column::Status.eq(status))
            .order_by_desc(api_debug::Column::CreatedAt)
            .all(self.db.as_ref())
            .await?;

        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    /// Get statistics
    pub async fn get_stats(&self) -> Result<ApiDebugStats> {
        let total = Entity::find().count(self.db.as_ref()).await?;

        let success_count = Entity::find()
            .filter(api_debug::Column::Status.eq(RequestStatus::Success))
            .count(self.db.as_ref())
            .await?;

        let failed_count = Entity::find()
            .filter(api_debug::Column::Status.eq(RequestStatus::Failed))
            .count(self.db.as_ref())
            .await?;

        let pending_count = Entity::find()
            .filter(api_debug::Column::Status.eq(RequestStatus::Pending))
            .count(self.db.as_ref())
            .await?;

        Ok(ApiDebugStats {
            total,
            success_count,
            failed_count,
            pending_count,
        })
    }

    /// Clear all API debug entries
    pub async fn clear_all(&self) -> Result<u64> {
        let result = Entity::delete_many().exec(self.db.as_ref()).await?;

        Ok(result.rows_affected)
    }
}

/// Statistics for API debug entries
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiDebugStats {
    pub total: u64,
    pub success_count: u64,
    pub failed_count: u64,
    pub pending_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migration::Migrator;
    use sea_orm::Database;
    use sea_orm_migration::MigratorTrait;
    use serde_json::json;

    async fn setup_test_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        Migrator::up(&db, None).await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_create_api_debug() {
        let db = Arc::new(setup_test_db().await);
        let dao = ApiDebugDao::new(db);

        let req = CreateApiDebugRequest {
            name: "Test API".to_string(),
            method: HttpMethod::Get,
            url: "https://api.example.com/test".to_string(),
            headers: Some(json!({"Content-Type": "application/json"})),
            body: Some(r#"{"test": "data"}"#.to_string()),
            content_type: Some("application/json".to_string()),
            timeout: Some(30),
        };

        let result = dao.create(req).await.unwrap();
        assert_eq!(result.name, "Test API");
        assert_eq!(result.method, HttpMethod::Get);
        assert_eq!(result.status, RequestStatus::Pending);
    }

    #[tokio::test]
    async fn test_get_by_id() {
        let db = Arc::new(setup_test_db().await);
        let dao = ApiDebugDao::new(db);

        let req = CreateApiDebugRequest {
            name: "Test API".to_string(),
            method: HttpMethod::Post,
            url: "https://api.example.com/test".to_string(),
            headers: None,
            body: None,
            content_type: None,
            timeout: None,
        };

        let created = dao.create(req).await.unwrap();
        let retrieved = dao.get_by_id(created.id).await.unwrap();

        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.name, "Test API");
    }

    #[tokio::test]
    async fn test_list_with_pagination() {
        let db = Arc::new(setup_test_db().await);
        let dao = ApiDebugDao::new(db);

        // Create multiple entries
        for i in 1..=25 {
            let req = CreateApiDebugRequest {
                name: format!("Test API {}", i),
                method: HttpMethod::Get,
                url: format!("https://api.example.com/test{}", i),
                headers: None,
                body: None,
                content_type: None,
                timeout: None,
            };
            dao.create(req).await.unwrap();
        }

        let params = ApiDebugQueryParams {
            page: Some(1),
            per_page: Some(10),
            method: None,
            status: None,
            search: None,
        };

        let result = dao.list(params).await.unwrap();
        assert_eq!(result.data.len(), 10);
        assert_eq!(result.total, 25);
        assert_eq!(result.page, 1);
        assert_eq!(result.per_page, 10);
        assert_eq!(result.total_pages, 3);
    }
}
