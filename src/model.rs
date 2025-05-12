use crate::{commands::Command, error::JujikError, pin::Pin, tab::Tab};
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

    pub fn run(self) -> Result<JoinHandle<Result<(), JujikError>>, JujikError> {
        Ok(thread::Builder::new().name("Model".to_string()).spawn(
            move || -> Result<(), JujikError> {
                'event_loop: loop {
                    if let Ok(command) = self.model.try_recv() {
                        println!("Model: {:?}", command);

                        match command {
                            // Pin
                            Command::CreatePin(pathbuf) => {
                                let new_pin = Pin::new(pathbuf);
                                match new_pin {
                                    Ok(new_pin) => {
                                        self.controller.send(Command::NewPin(new_pin))?
                                    }
                                    Err(err) => {
                                        self.controller.send(Command::Error(Box::new(err)))?
                                    }
                                }
                            }

                            // Tab
                            Command::CreateTab(tab_kind, pathbuf) => {
                                let new_tab = Tab::new(tab_kind, pathbuf);
                                match new_tab {
                                    Ok(new_tab) => {
                                        self.controller.send(Command::NewTab(None, new_tab))?
                                    }
                                    Err(err) => {
                                        self.controller.send(Command::Error(Box::new(err)))?
                                    }
                                }
                            }
                            Command::ChangeTabDirectory(idx, mut tab, pathbuf) => {
                                match tab.change_dir(pathbuf) {
                                    Ok(_) => {
                                        self.controller.send(Command::NewTab(Some(idx), tab))?
                                    }
                                    Err(err) => {
                                        self.controller.send(Command::Error(Box::new(err)))?
                                    }
                                }
                            }

                            Command::Drop => break 'event_loop,
                            _ => {}
                        }
                    };

                    std::thread::sleep(std::time::Duration::from_millis(8));
                }
                Ok(())
            },
        )?)
    }
}
