use crate::{commands::Command, error::JujikError};
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

    pub fn run(self) -> Result<JoinHandle<Result<(), JujikError>>, JujikError> {
        Ok(thread::Builder::new()
            .name("Controller".to_string())
            .spawn(move || -> Result<(), JujikError> {
                'event_loop: loop {
                    if let Ok(command) = self.controller.try_recv() {
                        println!("Controller: {:?}", command);

                        match command {
                            Command::NewPin(pathbuf) => {
                                let mut path = pathbuf.as_path();
                                if path.exists() {
                                    if !path.is_dir() {
                                        if let Some(parent) = path.parent() {
                                            path = parent;
                                        }
                                    }
                                    self.model.send(Command::NewPin(path.to_path_buf()))?;
                                } else {
                                    self.view.send(Command::Error(Box::new(JujikError::Other(
                                        format!("Path {:?} does not exist", pathbuf),
                                    ))))?;
                                }
                            }
                            Command::ShowPin(pin) => self.view.send(Command::ShowPin(pin))?,
                            Command::NewTab(pin) => self.model.send(Command::CreateTab(pin))?,
                            Command::Error(err) => self.view.send(Command::Error(err))?,
                            Command::Drop => {
                                self.send_drop()?;
                                break 'event_loop;
                            }
                            _ => {}
                        }
                    };

                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Ok(())
            })?)
    }

    fn send_drop(&self) -> Result<(), JujikError> {
        let _view_drop = self.view.send(Command::Drop);
        let _model_drop = self.model.send(Command::Drop);
        Ok(())
    }
}
