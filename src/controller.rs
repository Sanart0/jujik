use crate::error::CustomError;
use std::thread::{self, JoinHandle};

pub struct Jujik {}

impl Jujik {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(self) -> JoinHandle<Result<(), CustomError>> {
        thread::spawn(|| -> Result<(), CustomError> { Ok(()) })
    }
}
