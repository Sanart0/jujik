use std::thread::{self, JoinHandle};

use crate::error::CustomError;

pub struct JujikModel {}

impl JujikModel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(self) -> JoinHandle<Result<(), CustomError>> {
        thread::spawn(|| -> Result<(), CustomError> { Ok(()) })
    }
}
