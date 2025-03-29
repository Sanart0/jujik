use crate::commands::Command;
use std::{any::Any, fmt::Debug};

pub enum CustomError {
    IO(std::io::Error),
    Send(std::sync::mpsc::SendError<Command>),
    Recv(std::sync::mpsc::RecvError),
    RecvTimeout(std::sync::mpsc::RecvTimeoutError),
    EFrame(eframe::Error),
    Logger(log::SetLoggerError),
    Thread(Box<dyn Any + Send>),
    Other(String),
}

impl CustomError {
    #[allow(clippy::unnecessary_literal_unwrap)]
    pub fn handle_err<E: std::error::Error>(err: &E) {
        eprintln!("{}", err);
    }
}

impl From<std::io::Error> for CustomError {
    fn from(value: std::io::Error) -> Self {
        CustomError::IO(value)
    }
}

impl From<std::sync::mpsc::SendError<Command>> for CustomError {
    fn from(value: std::sync::mpsc::SendError<Command>) -> Self {
        CustomError::Send(value)
    }
}

impl From<std::sync::mpsc::RecvError> for CustomError {
    fn from(value: std::sync::mpsc::RecvError) -> Self {
        CustomError::Recv(value)
    }
}

impl From<std::sync::mpsc::RecvTimeoutError> for CustomError {
    fn from(value: std::sync::mpsc::RecvTimeoutError) -> Self {
        CustomError::RecvTimeout(value)
    }
}

impl From<eframe::Error> for CustomError {
    fn from(value: eframe::Error) -> Self {
        CustomError::EFrame(value)
    }
}

impl From<log::SetLoggerError> for CustomError {
    fn from(value: log::SetLoggerError) -> Self {
        CustomError::Logger(value)
    }
}

impl From<Box<dyn Any + Send>> for CustomError {
    fn from(value: Box<dyn Any + Send>) -> Self {
        CustomError::Thread(value)
    }
}

impl Debug for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CustomError::IO(err) => err.to_string(),
                CustomError::Send(err) => err.to_string(),
                CustomError::Recv(err) => err.to_string(),
                CustomError::RecvTimeout(err) => err.to_string(),
                CustomError::EFrame(err) => err.to_string(),
                CustomError::Logger(err) => err.to_string(),
                CustomError::Thread(err) => format!("{:?}", err),
                CustomError::Other(err) => err.to_string(),
            }
        )
    }
}

unsafe impl Send for CustomError {}
