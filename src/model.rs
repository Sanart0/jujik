use crate::{commands::Command, error::CustomError};
use std::{
    sync::mpsc::{Receiver, Sender},
    thread::{self, JoinHandle},
};

pub struct JujikModel {
    controller: Sender<Command>,
    model: Receiver<Command>,
}

impl JujikModel {
    pub fn new(controller: Sender<Command>, model: Receiver<Command>) -> Self {
        Self { controller, model }
    }

    pub fn run(self) -> JoinHandle<Result<(), CustomError>> {
        thread::spawn(move || -> Result<(), CustomError> {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Ok(())
        })
    }
}
