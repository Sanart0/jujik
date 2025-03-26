use std::fmt::Debug;

pub enum CustomError {
    IO(std::io::Error),
    EFrame(eframe::Error),
    Logger(log::SetLoggerError),
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

impl Debug for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CustomError::IO(error) => error.to_string(),
                CustomError::EFrame(error) => error.to_string(),
                CustomError::Logger(error) => error.to_string(),
                CustomError::Other(error) => error.to_string(),
            }
        )
    }
}

unsafe impl Send for CustomError {}
