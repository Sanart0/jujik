use std::{any::Any, fmt::Debug};

pub enum CustomError {
    IO(std::io::Error),
    EFrame(eframe::Error),
    Logger(log::SetLoggerError),
    Thread(Box<dyn Any + Send>),
    Other(String),
}

impl CustomError {}

impl From<std::io::Error> for CustomError {
    fn from(value: std::io::Error) -> Self {
        CustomError::IO(value)
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
                CustomError::IO(e) => e.to_string(),
                CustomError::EFrame(e) => e.to_string(),
                CustomError::Logger(e) => e.to_string(),
                CustomError::Thread(e) => format!("{:?}", e),
                CustomError::Other(e) => e.to_string(),
            }
        )
    }
}

unsafe impl Send for CustomError {}
