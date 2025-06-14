use crate::self_service::{
    RouteState,
    utils::{EmptyOkResponse, ResponseDataWrapper, empty_ok, ok},
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use lynx_db::dao::api_debug_dao::{
    ApiDebugDao, ApiDebugListResponse, ApiDebugQueryParams, ApiDebugResponse, ApiDebugStats,
    CreateApiDebugRequest, UpdateApiDebugRequest,
};
use lynx_db::entities::api_debug::{HttpMethod, RequestStatus};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

/// Query parameters for listing API debug entries
#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DebugQueryParams {
    /// Page number, starting from 1
    pub page: Option<u64>,
    /// Number of items per page, default 20
    pub per_page: Option<u64>,
    /// Filter by HTTP method
    pub method: Option<HttpMethod>,
    /// Filter by request status
    pub status: Option<RequestStatus>,
    /// Search in name and URL
    pub search: Option<String>,
}

#[utoipa::path(
    post,
    path = "/debug",
    tags = ["API Debug"],
    request_body = CreateApiDebugRequest,
    responses(
        (status = 200, description = "API debug entry created successfully", body = ResponseDataWrapper<ApiDebugResponse>),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Failed to create API debug entry")
    )
)]
async fn create_debug_entry(
    State(RouteState { db, .. }): State<RouteState>,
    Json(request): Json<CreateApiDebugRequest>,
) -> Result<Json<ResponseDataWrapper<ApiDebugResponse>>, StatusCode> {
    let dao = ApiDebugDao::new(db);

    let result = dao.create(request).await.map_err(|e| {
        tracing::error!("Failed to create API debug entry: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ok(result)))
}

#[utoipa::path(
    get,
    path = "/debug",
    tags = ["API Debug"],
    params(DebugQueryParams),
    responses(
        (status = 200, description = "Successfully retrieved API debug entries", body = ResponseDataWrapper<ApiDebugListResponse>),
        (status = 500, description = "Failed to get API debug entries")
    )
)]
async fn list_debug_entries(
    State(RouteState { db, .. }): State<RouteState>,
    Query(params): Query<DebugQueryParams>,
) -> Result<Json<ResponseDataWrapper<ApiDebugListResponse>>, StatusCode> {
    let dao = ApiDebugDao::new(db);

    // Convert local query params to DAO query params
    let dao_params = ApiDebugQueryParams {
        page: params.page,
        per_page: params.per_page,
        method: params.method,
        status: params.status,
        search: params.search,
    };

    let result = dao.list(dao_params).await.map_err(|e| {
        tracing::error!("Failed to list API debug entries: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ok(result)))
}

#[utoipa::path(
    get,
    path = "/debug/{id}",
    tags = ["API Debug"],
    params(
        ("id" = i32, Path, description = "API debug entry ID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved API debug entry", body = ResponseDataWrapper<ApiDebugResponse>),
        (status = 404, description = "API debug entry not found"),
        (status = 500, description = "Failed to get API debug entry")
    )
)]
async fn get_debug_entry(
    State(RouteState { db, .. }): State<RouteState>,
    Path(id): Path<i32>,
) -> Result<Json<ResponseDataWrapper<ApiDebugResponse>>, StatusCode> {
    let dao = ApiDebugDao::new(db);

    let result = dao.get_by_id(id).await.map_err(|e| {
        tracing::error!("Failed to get API debug entry: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match result {
        Some(entry) => Ok(Json(ok(entry))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[utoipa::path(
    put,
    path = "/debug/{id}",
    tags = ["API Debug"],
    params(
        ("id" = i32, Path, description = "API debug entry ID")
    ),
    request_body = UpdateApiDebugRequest,
    responses(
        (status = 200, description = "API debug entry updated successfully", body = ResponseDataWrapper<ApiDebugResponse>),
        (status = 404, description = "API debug entry not found"),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Failed to update API debug entry")
    )
)]
async fn update_debug_entry(
    State(RouteState { db, .. }): State<RouteState>,
    Path(id): Path<i32>,
    Json(request): Json<UpdateApiDebugRequest>,
) -> Result<Json<ResponseDataWrapper<ApiDebugResponse>>, StatusCode> {
    let dao = ApiDebugDao::new(db);

    let result = dao.update(id, request).await.map_err(|e| {
        tracing::error!("Failed to update API debug entry: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match result {
        Some(entry) => Ok(Json(ok(entry))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[utoipa::path(
    delete,
    path = "/debug/{id}",
    tags = ["API Debug"],
    params(
        ("id" = i32, Path, description = "API debug entry ID")
    ),
    responses(
        (status = 200, description = "API debug entry deleted successfully", body = EmptyOkResponse),
        (status = 404, description = "API debug entry not found"),
        (status = 500, description = "Failed to delete API debug entry")
    )
)]
async fn delete_debug_entry(
    State(RouteState { db, .. }): State<RouteState>,
    Path(id): Path<i32>,
) -> Result<Json<EmptyOkResponse>, StatusCode> {
    let dao = ApiDebugDao::new(db);

    let result = dao.delete(id).await.map_err(|e| {
        tracing::error!("Failed to delete API debug entry: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if result {
        Ok(Json(empty_ok()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[utoipa::path(
    get,
    path = "/debug/stats",
    tags = ["API Debug"],
    responses(
        (status = 200, description = "Successfully retrieved API debug statistics", body = ResponseDataWrapper<ApiDebugStats>),
        (status = 500, description = "Failed to get API debug statistics")
    )
)]
async fn get_debug_stats(
    State(RouteState { db, .. }): State<RouteState>,
) -> Result<Json<ResponseDataWrapper<ApiDebugStats>>, StatusCode> {
    let dao = ApiDebugDao::new(db);

    let result = dao.get_stats().await.map_err(|e| {
        tracing::error!("Failed to get API debug statistics: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ok(result)))
}

pub fn router(state: RouteState) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(create_debug_entry))
        .routes(routes!(list_debug_entries))
        .routes(routes!(get_debug_entry))
        .routes(routes!(update_debug_entry))
        .routes(routes!(delete_debug_entry))
        .routes(routes!(get_debug_stats))
        .with_state(state)
}
