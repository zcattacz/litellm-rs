//! HTTP request handlers for API key management

use super::types::{
    CreateKeyRequest, CreateKeyResponse, KeyErrorResponse, KeyResponse, KeyUsageResponse,
    ListKeysQuery, ListKeysResponse, PaginationInfo, RevokeKeyResponse, RotateKeyResponse,
    UpdateKeyRequest, VerifyKeyRequest, VerifyKeyResponse,
};
use crate::core::keys::KeyManager;
use crate::core::keys::{CreateKeyConfig, KeyStatus, UpdateKeyConfig};
use crate::server::routes::ApiResponse;
use crate::server::state::AppState;
use actix_web::{HttpResponse, Result as ActixResult, web};
use tracing::{error, info, warn};
use uuid::Uuid;

/// POST /v1/keys - Create a new API key
pub async fn create_key(
    state: web::Data<AppState>,
    request: web::Json<CreateKeyRequest>,
) -> ActixResult<HttpResponse> {
    info!("Creating new API key: {}", request.name);

    // Get key manager from state
    let key_manager = get_key_manager(&state)?;

    // Build creation config
    let config = CreateKeyConfig {
        name: request.name.clone(),
        description: request.description.clone(),
        user_id: request.user_id,
        team_id: request.team_id,
        budget_id: request.budget_id,
        permissions: request.permissions.clone().unwrap_or_default(),
        rate_limits: request.rate_limits.clone().unwrap_or_default(),
        expires_at: request.expires_at,
        metadata: request.metadata.clone().unwrap_or(serde_json::Value::Null),
    };

    // Generate the key
    match key_manager.generate_key(config).await {
        Ok((key_id, raw_key)) => {
            // Get key info
            let key_info = key_manager
                .get_key(key_id)
                .await
                .map_err(|e| {
                    error!("Failed to get key info: {}", e);
                    actix_web::error::ErrorInternalServerError("Failed to get key info")
                })?
                .ok_or_else(|| {
                    actix_web::error::ErrorInternalServerError("Key not found after creation")
                })?;

            let response = CreateKeyResponse {
                id: key_id,
                key: raw_key,
                info: key_info,
                warning: "Store this key securely. It will not be shown again.".to_string(),
            };

            info!("API key created successfully: {}", key_id);
            Ok(HttpResponse::Created().json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to create API key: {}", e);
            let error_response = KeyErrorResponse::validation(e.to_string());
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(error_response.error)))
        }
    }
}

/// GET /v1/keys - List API keys
pub async fn list_keys(
    state: web::Data<AppState>,
    query: web::Query<ListKeysQuery>,
) -> ActixResult<HttpResponse> {
    info!("Listing API keys");

    let key_manager = get_key_manager(&state)?;

    // Calculate offset from page
    let offset = ((query.page.saturating_sub(1)) * query.limit) as usize;
    let limit = query.limit as usize;

    // Get keys based on filters
    let keys = if let Some(user_id) = query.user_id {
        key_manager.list_user_keys(user_id).await
    } else if let Some(team_id) = query.team_id {
        key_manager.list_team_keys(team_id).await
    } else {
        key_manager
            .list_keys(query.status, Some(limit), Some(offset))
            .await
    };

    match keys {
        Ok(keys) => {
            // Get total count for pagination
            let total = key_manager.count_keys(query.status).await.unwrap_or(0);

            let response = ListKeysResponse {
                keys,
                pagination: PaginationInfo::new(total, query.page, query.limit),
            };

            Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to list API keys: {}", e);
            let error_response = KeyErrorResponse::internal("Failed to list keys");
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error(error_response.error)))
        }
    }
}

/// GET /v1/keys/{id} - Get a specific API key
pub async fn get_key(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> ActixResult<HttpResponse> {
    let key_id = path.into_inner();
    info!("Getting API key: {}", key_id);

    let key_manager = get_key_manager(&state)?;

    match key_manager.get_key(key_id).await {
        Ok(Some(key)) => {
            let response = KeyResponse { key };
            Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
        }
        Ok(None) => {
            warn!("API key not found: {}", key_id);
            let error_response = KeyErrorResponse::not_found("API key");
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(error_response.error)))
        }
        Err(e) => {
            error!("Failed to get API key: {}", e);
            let error_response = KeyErrorResponse::internal("Failed to get key");
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error(error_response.error)))
        }
    }
}

/// PUT /v1/keys/{id} - Update an API key
pub async fn update_key(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    request: web::Json<UpdateKeyRequest>,
) -> ActixResult<HttpResponse> {
    let key_id = path.into_inner();
    info!("Updating API key: {}", key_id);

    let key_manager = get_key_manager(&state)?;

    let config = UpdateKeyConfig {
        name: request.name.clone(),
        description: request.description.clone(),
        permissions: request.permissions.clone(),
        rate_limits: request.rate_limits.clone(),
        budget_id: request.budget_id,
        expires_at: request.expires_at,
        metadata: request.metadata.clone(),
    };

    match key_manager.update_key(key_id, config).await {
        Ok(key) => {
            info!("API key updated successfully: {}", key_id);
            let response = KeyResponse { key };
            Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to update API key: {}", e);
            let error_str = e.to_string();
            if error_str.contains("not found") {
                let error_response = KeyErrorResponse::not_found("API key");
                Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(error_response.error)))
            } else if error_str.contains("revoked") {
                let error_response = KeyErrorResponse::conflict("Cannot update a revoked key");
                Ok(HttpResponse::Conflict().json(ApiResponse::<()>::error(error_response.error)))
            } else {
                let error_response = KeyErrorResponse::validation(error_str);
                Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(error_response.error)))
            }
        }
    }
}

/// DELETE /v1/keys/{id} - Revoke an API key
pub async fn revoke_key(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> ActixResult<HttpResponse> {
    let key_id = path.into_inner();
    info!("Revoking API key: {}", key_id);

    let key_manager = get_key_manager(&state)?;

    match key_manager.revoke_key(key_id).await {
        Ok(()) => {
            info!("API key revoked successfully: {}", key_id);
            let response = RevokeKeyResponse {
                key_id,
                status: KeyStatus::Revoked,
                message: "API key has been revoked successfully".to_string(),
            };
            Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to revoke API key: {}", e);
            let error_str = e.to_string();
            if error_str.contains("not found") {
                let error_response = KeyErrorResponse::not_found("API key");
                Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(error_response.error)))
            } else if error_str.contains("already revoked") {
                let error_response = KeyErrorResponse::conflict("API key is already revoked");
                Ok(HttpResponse::Conflict().json(ApiResponse::<()>::error(error_response.error)))
            } else {
                let error_response = KeyErrorResponse::internal("Failed to revoke key");
                Ok(HttpResponse::InternalServerError()
                    .json(ApiResponse::<()>::error(error_response.error)))
            }
        }
    }
}

/// POST /v1/keys/{id}/rotate - Rotate an API key
pub async fn rotate_key(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> ActixResult<HttpResponse> {
    let key_id = path.into_inner();
    info!("Rotating API key: {}", key_id);

    let key_manager = get_key_manager(&state)?;

    match key_manager.rotate_key(key_id).await {
        Ok((new_key_id, new_raw_key)) => {
            // Get new key info
            let key_info = key_manager
                .get_key(new_key_id)
                .await
                .map_err(|e| {
                    error!("Failed to get new key info: {}", e);
                    actix_web::error::ErrorInternalServerError("Failed to get new key info")
                })?
                .ok_or_else(|| {
                    actix_web::error::ErrorInternalServerError("New key not found after rotation")
                })?;

            info!("API key rotated successfully: {} -> {}", key_id, new_key_id);

            let response = RotateKeyResponse {
                old_key_id: key_id,
                new_key_id,
                new_key: new_raw_key,
                info: key_info,
                warning: "Store this new key securely. It will not be shown again. The old key has been revoked.".to_string(),
            };

            Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to rotate API key: {}", e);
            let error_str = e.to_string();
            if error_str.contains("not found") {
                let error_response = KeyErrorResponse::not_found("API key");
                Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(error_response.error)))
            } else if error_str.contains("revoked") {
                let error_response = KeyErrorResponse::conflict("Cannot rotate a revoked key");
                Ok(HttpResponse::Conflict().json(ApiResponse::<()>::error(error_response.error)))
            } else {
                let error_response = KeyErrorResponse::internal("Failed to rotate key");
                Ok(HttpResponse::InternalServerError()
                    .json(ApiResponse::<()>::error(error_response.error)))
            }
        }
    }
}

/// GET /v1/keys/{id}/usage - Get usage statistics for an API key
pub async fn get_key_usage(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> ActixResult<HttpResponse> {
    let key_id = path.into_inner();
    info!("Getting usage stats for API key: {}", key_id);

    let key_manager = get_key_manager(&state)?;

    match key_manager.get_usage_stats(key_id).await {
        Ok(usage) => {
            let response = KeyUsageResponse { key_id, usage };
            Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to get usage stats: {}", e);
            let error_str = e.to_string();
            if error_str.contains("not found") {
                let error_response = KeyErrorResponse::not_found("API key");
                Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(error_response.error)))
            } else {
                let error_response = KeyErrorResponse::internal("Failed to get usage stats");
                Ok(HttpResponse::InternalServerError()
                    .json(ApiResponse::<()>::error(error_response.error)))
            }
        }
    }
}

/// POST /v1/keys/verify - Verify an API key
pub async fn verify_key(
    state: web::Data<AppState>,
    request: web::Json<VerifyKeyRequest>,
) -> ActixResult<HttpResponse> {
    info!("Verifying API key");

    let key_manager = get_key_manager(&state)?;

    match key_manager.validate_key(&request.key).await {
        Ok(result) => {
            let response = VerifyKeyResponse { result };
            Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to verify API key: {}", e);
            let error_response = KeyErrorResponse::internal("Failed to verify key");
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error(error_response.error)))
        }
    }
}

/// Helper function to get KeyManager from AppState
///
/// Note: This is a placeholder. In a real implementation, the KeyManager
/// would be stored in AppState or created with the appropriate repository.
fn get_key_manager(_state: &web::Data<AppState>) -> ActixResult<KeyManager> {
    // In production, this would retrieve the KeyManager from AppState
    // For now, we create a new one with an in-memory repository
    use crate::core::keys::InMemoryKeyRepository;

    // TODO: Replace with proper KeyManager retrieval from AppState
    // The KeyManager should be initialized during application startup
    // and stored in AppState for shared access across handlers
    Ok(KeyManager::new(InMemoryKeyRepository::new()))
}

#[cfg(test)]
mod handler_tests {
    use super::*;

    #[test]
    fn test_create_key_config_from_request() {
        let request = CreateKeyRequest {
            name: "Test Key".to_string(),
            description: Some("A test".to_string()),
            user_id: None,
            team_id: None,
            budget_id: None,
            permissions: None,
            rate_limits: None,
            expires_at: None,
            metadata: None,
        };

        let config = CreateKeyConfig {
            name: request.name.clone(),
            description: request.description.clone(),
            user_id: request.user_id,
            team_id: request.team_id,
            budget_id: request.budget_id,
            permissions: request.permissions.clone().unwrap_or_default(),
            rate_limits: request.rate_limits.clone().unwrap_or_default(),
            expires_at: request.expires_at,
            metadata: request.metadata.clone().unwrap_or(serde_json::Value::Null),
        };

        assert_eq!(config.name, "Test Key");
        assert!(config.description.is_some());
    }
}
