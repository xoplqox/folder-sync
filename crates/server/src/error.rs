use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn forbidden(message: impl Into<String>) -> Self {
        ApiError {
            status: StatusCode::FORBIDDEN,
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        ApiError {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }
}

impl From<folder_sync_core::CoreError> for ApiError {
    fn from(err: folder_sync_core::CoreError) -> Self {
        ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: err.to_string(),
        }
    }
}

impl From<folder_sync_core::batch::PlanError> for ApiError {
    fn from(err: folder_sync_core::batch::PlanError) -> Self {
        use folder_sync_core::batch::PlanError;
        let status = match err {
            PlanError::NotFound => StatusCode::NOT_FOUND,
            PlanError::HasConflict => StatusCode::CONFLICT,
            PlanError::NotAFile
            | PlanError::NothingToSync
            | PlanError::AlreadyInSync
            | PlanError::NothingToDelete => StatusCode::BAD_REQUEST,
        };
        ApiError {
            status,
            message: err.to_string(),
        }
    }
}

impl From<folder_sync_core::batch::RemoveError> for ApiError {
    fn from(err: folder_sync_core::batch::RemoveError) -> Self {
        use folder_sync_core::batch::RemoveError;
        let status = match err {
            RemoveError::NotFound => StatusCode::NOT_FOUND,
            RemoveError::Running => StatusCode::CONFLICT,
        };
        ApiError {
            status,
            message: err.to_string(),
        }
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                error: self.message,
            }),
        )
            .into_response()
    }
}
