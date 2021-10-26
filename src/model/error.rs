use actix_web::ResponseError;
use anyhow;
use juniper::{graphql_value, FieldError};
use thiserror::Error;

pub trait SpecificError {
  fn get_code(&self) -> &str;
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum FormatError {
  #[error("server error: {0}")]
  ServerError(String),
  #[error("not found: {0}")]
  NotFoundError(String),
  #[error("forbidden: {0}")]
  Forbidden(String),
  #[error("bad request: {0}")]
  BadRequest(String),
  #[error("validation failed: {0}")]
  ValidationFailed(String),
  #[error("Unauthenticated: {0}")]
  Unauthenticated(String),
  #[error("duplicate value found: {0}")]
  DuplicateError(String),
}

impl SpecificError for FormatError {
  fn get_code(&self) -> &str {
    match self {
      FormatError::ServerError(_) => "SERVER_ERROR",
      FormatError::NotFoundError(_) => "NOT_FOUND",
      FormatError::Forbidden(_) => "FORBIDDEN",
      FormatError::ValidationFailed(_) => "VALIDATION_FAILED",
      FormatError::Unauthenticated(_) => "UNAUTHENTICATED",
      FormatError::BadRequest(_) => "BAD_REQUEST",
      FormatError::DuplicateError(_) => "DUPLICATE_ERROR",
    }
  }
}

impl ResponseError for FormatError {}

impl SpecificError for anyhow::Error {
  fn get_code(&self) -> &str {
    match self.downcast_ref::<FormatError>() {
      Some(format_error) => format_error.get_code(),
      None => "UNEXPECTED_ERROR",
    }
  }
}

pub fn to_juniper_field_error(err: anyhow::Error) -> FieldError {
  let code = err.get_code();
  FieldError::new(format!("{:#}", err), graphql_value!({ "code": code }))
}
