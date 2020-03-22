use actix_web::{Error as ActixError, HttpResponse, ResponseError};
use deadpool_postgres::PoolError;
use derive_more::{Display, From};
use reqwest::Error as ReqwestError;
use tokio_postgres::error::Error as PGError;

#[derive(Display, From, Debug)]
pub enum AppError {
    NotFound,
    PGError(PGError),
    PoolError(PoolError),
    ActixError(ActixError),
    ReqwestError(ReqwestError),
}
impl std::error::Error for AppError {}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            AppError::NotFound => HttpResponse::NotFound().finish(),
            AppError::PGError(ref err) => HttpResponse::InternalServerError().body(err.to_string()),
            AppError::PoolError(ref err) => {
                HttpResponse::InternalServerError().body(err.to_string())
            }
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}
