use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
pub enum ApiError {
    NotFound(String),
    InvalidInput(String),
    InternalServerError(String),
    DatabaseError(String),
    Unauthorized(String),
    Conflict(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            ApiError::InvalidInput(msg) => write!(f, "Invalid Input: {}", msg),
            ApiError::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
            ApiError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
            ApiError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            ApiError::Conflict(msg) => write!(f, "Conflict: {}", msg),
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::NotFound(msg) => HttpResponse::build(StatusCode::NOT_FOUND).json(msg),
            ApiError::InvalidInput(msg) => HttpResponse::build(StatusCode::BAD_REQUEST).json(msg),
            ApiError::InternalServerError(msg) => HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).json(msg),
            ApiError::DatabaseError(msg) => HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).json(msg),
            ApiError::Unauthorized(msg) => HttpResponse::build(StatusCode::UNAUTHORIZED).json(msg),
            ApiError::Conflict(msg) => HttpResponse::build(StatusCode::CONFLICT).json(msg),
        }
    }
}


impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> ApiError {
        ApiError::DatabaseError(err.to_string())
    }
}


impl From<validator::ValidationErrors> for ApiError {
    fn from(err: validator::ValidationErrors) -> ApiError {
        ApiError::InvalidInput(serde_json::to_string(&err).unwrap_or_else(|_| "Validation error".to_string()))
    }
}