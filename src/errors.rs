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

    #[error("a project with name={0} already exists, Ignore the Error if you are not using manage command.")]
    Duplicate(String),

    #[error("Check Internet Connection : {0}.")]
    Network(String),

    #[error("Python is not installed or not found")]
    PythonNotInstalled,

    #[error("{0} is not a valid project name")]
    NotValidProjectName(String),

    #[error("docker failed : {0}")]
    Docker(String)
}

pub fn error_printer(err: Error) {
    println!("Error: {}", err);
    println!("Caused by: {}", err.root_cause());
    println!("backtrace: {:?}", err.backtrace());
}
