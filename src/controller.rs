use crate::{commands::Command, error::JujikError, pin::Pin, tab::Tab};
use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
    thread::{self, JoinHandle},
};

pub struct JujikController {
    model: Sender<Command>,
    view: Sender<Command>,
    controller: Receiver<Command>,
    pins: Vec<Pin>,
    tabs: Vec<Tab>,
}

impl JujikController {
    pub fn new(
        model: Sender<Command>,
        view: Sender<Command>,
        controller: Receiver<Command>,
    ) -> Self {
        //TODO load a pins from cache toml file, and sync saves with view

        Self {
            model,
            view,
            controller,
            pins: Vec::new(),
            tabs: Vec::new(),
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
                            // Pin
                            Command::NewPin(mut pathbuf) => {
                                if pathbuf.exists() {
                                    if !pathbuf.is_dir() {
                                        if let Some(parent) = pathbuf.parent() {
                                            pathbuf = parent.to_path_buf();
                                        }
                                    }
                                    self.model.send(Command::NewPin(pathbuf))?;
                                } else {
                                    self.view.send(Command::Error(Box::new(JujikError::Other(
                                        format!("Path {:?} does not exist", pathbuf),
                                    ))))?;
                                }
                            }
                            Command::ShowPin(pin) => self.view.send(Command::ShowPin(pin))?,

                            // Tab
                            Command::NewTab(pin) => {
                                let tab_exist = self
                                    .tabs
                                    .iter()
                                    .filter(|tab| PathBuf::from(tab.get_name())== pin)
                                    .collect::<Vec<_>>()
                                    .is_empty();

                                if tab_exist {
                                    self.model.send(Command::NewTab(pin))?
                                }
                            }
                            Command::ShowTab(tab) => self.view.send(Command::ShowTab(tab))?,

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
