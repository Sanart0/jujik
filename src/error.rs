use crate::commands::Command;
use std::{
    any::Any,
    fmt::{Debug, Display},
};

pub enum JujikError {
    None,
    IO(std::io::Error),
    //TODO contains a Command
    // Send(std::sync::mpsc::SendError<Command>),
    Send(String),
    Recv(std::sync::mpsc::RecvError),
    TryRecv(std::sync::mpsc::TryRecvError),
    RecvTimeout(std::sync::mpsc::RecvTimeoutError),
    EFrame(eframe::Error),
    Logger(log::SetLoggerError),
    Thread(Box<dyn Any + Send>),
    Nix(nix::errno::Errno),
    Other(String),
}

impl JujikError {
    // #[allow(clippy::unnecessary_literal_unwrap)]
    pub fn handle_err<E: std::error::Error>(err: &E) {
        //TODO Handle error
        eprintln!("{}", err);
    }
}

impl From<std::io::Error> for JujikError {
    fn from(value: std::io::Error) -> Self {
        JujikError::IO(value)
    }
}

impl From<std::sync::mpsc::SendError<Command>> for JujikError {
    fn from(value: std::sync::mpsc::SendError<Command>) -> Self {
        JujikError::Send(value.to_string())
    }
}

impl From<std::sync::mpsc::RecvError> for JujikError {
    fn from(value: std::sync::mpsc::RecvError) -> Self {
        JujikError::Recv(value)
    }
}

impl From<std::sync::mpsc::TryRecvError> for JujikError {
    fn from(value: std::sync::mpsc::TryRecvError) -> Self {
        JujikError::TryRecv(value)
    }
}

impl From<std::sync::mpsc::RecvTimeoutError> for JujikError {
    fn from(value: std::sync::mpsc::RecvTimeoutError) -> Self {
        JujikError::RecvTimeout(value)
    }
}

impl From<eframe::Error> for JujikError {
    fn from(value: eframe::Error) -> Self {
        JujikError::EFrame(value)
    }
}

impl From<log::SetLoggerError> for JujikError {
    fn from(value: log::SetLoggerError) -> Self {
        JujikError::Logger(value)
    }
}

impl From<Box<dyn Any + Send>> for JujikError {
    fn from(value: Box<dyn Any + Send>) -> Self {
        JujikError::Thread(value)
    }
}

impl From<nix::errno::Errno> for JujikError {
    fn from(value: nix::errno::Errno) -> Self {
        JujikError::Nix(value)
    }
}

impl std::error::Error for JujikError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            JujikError::IO(io_error) => Some(io_error),
            JujikError::Send(_send_error) => None,
            JujikError::Recv(recv_error) => Some(recv_error),
            JujikError::TryRecv(try_recv_error) => Some(try_recv_error),
            JujikError::RecvTimeout(recv_timeout_error) => Some(recv_timeout_error),
            JujikError::EFrame(eframe_error) => Some(eframe_error),
            JujikError::Logger(logger_error) => Some(logger_error),
            JujikError::Thread(_thread_error) => None,
            JujikError::Nix(nix_error) => Some(nix_error),
            JujikError::Other(_other_error) => None,
            JujikError::None => None,
        }
    }
}

unsafe impl Send for JujikError {}

impl Display for JujikError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                JujikError::IO(err) => err.to_string(),
                JujikError::Send(err) => err.to_string(),
                JujikError::Recv(err) => err.to_string(),
                JujikError::TryRecv(err) => err.to_string(),
                JujikError::RecvTimeout(err) => err.to_string(),
                JujikError::EFrame(err) => err.to_string(),
                JujikError::Logger(err) => err.to_string(),
                JujikError::Thread(err) => format!("{:?}", err),
                JujikError::Nix(err) => err.to_string(),
                JujikError::Other(err) => err.to_string(),
                JujikError::None => "".to_string(),
            }
        )
    }
}

impl Debug for JujikError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
