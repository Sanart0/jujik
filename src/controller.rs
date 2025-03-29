use crate::{commands::Command, error::CustomError};
use std::{
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
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
                let mut command = Command::Empty;
                if let Ok(value) = self.controller.recv() {
                    command = value
                };

                println!("{:?}", command);

                match command {
                    Command::NewPin(path_str) => {
                        let mut path = Path::new(&path_str);
                        if path.exists() && !path.is_dir() {
                            if let Some(parent) = path.parent() {
                                path = parent;
                            }
                        }
                        println!("{:?}", path);
                    }
                    Command::CreatePin(path_buf) => {}
                    Command::ShowPin(pin) => {}
                    Command::ErrorPin(err) => {}
                    Command::Drop => {
                        self.view.send(Command::Drop)?;
                        self.model.send(Command::Drop)?;
                        break 'event_loop;
                    }
                    Command::Empty => {}
                }

                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Ok(())
        })
    }
}
