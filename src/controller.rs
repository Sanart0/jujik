use crate::{commands::Command, error::CustomError};
use std::{
    path::Path,
    sync::mpsc::{Receiver, Sender},
    thread::{self, JoinHandle},
};

pub struct JujikController {
    model: Sender<Command>,
    view: Sender<Command>,
    controller: Receiver<Command>,
}

impl JujikController {
    pub fn new(
        model: Sender<Command>,
        view: Sender<Command>,
        controller: Receiver<Command>,
    ) -> Self {
        Self {
            model,
            view,
            controller,
        }
    }

    pub fn run(self) -> JoinHandle<Result<(), CustomError>> {
        thread::spawn(move || -> Result<(), CustomError> {
            'event_loop: loop {
                if let Ok(command) = self.controller.try_recv() {
                    println!("{:?}", command);

                    match command {
                        Command::NewPin(path_str) => {
                            let mut path = Path::new(&path_str);
                            if path.exists() && !path.is_dir() {
                                if let Some(parent) = path.parent() {
                                    path = parent;
                                }
                            }
                        }
                        Command::Drop => {
                            self.view.send(Command::Drop)?;
                            self.model.send(Command::Drop)?;
                            break 'event_loop;
                        }
                        _ => {}
                    }
                };

                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Ok(())
        })
    }
}
