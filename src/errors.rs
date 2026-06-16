use anyhow::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManagerError {
    #[error("Home Path not found")]
    HomePathNotFound,

    #[error("as mut array failed")]
    AsMutArrayFailed,

    #[error("processing 'Value' failed: {0}")]
    Value(String),

    #[error("can't find project's index")]
    Index,
}

pub fn error_printer(err: Error) {
    println!("Error: {}", err);
    println!("Caused by: {}", err.root_cause());
    println!("backtrace: {:?}", err.backtrace());
}
