use crate::{commands::Command, error::JujikError, pin::Pin, tab::Tab};
use std::{
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

    pub fn run(mut self) -> Result<JoinHandle<Result<(), JujikError>>, JujikError> {
        Ok(thread::Builder::new()
            .name("Controller".to_string())
            .spawn(move || -> Result<(), JujikError> {
                'event_loop: loop {
                    if let Ok(command) = self.controller.try_recv() {
                        println!("Controller: {:?}", command);

                        match command {
                            // Pin
                            Command::CreatePin(mut pathbuf) => {
                                if pathbuf.exists() {
                                    if !pathbuf.is_dir() {
                                        if let Some(parent) = pathbuf.parent() {
                                            pathbuf = parent.to_path_buf();
                                        }
                                    }
                                    self.model.send(Command::CreatePin(pathbuf))?;
                                } else {
                                    self.view.send(Command::Error(Box::new(JujikError::Other(
                                        format!("Path {:?} does not exist", pathbuf),
                                    ))))?;
                                }
                            }
                            Command::DeletePin(idx) => {
                                if idx < self.pins.len() {
                                    self.pins.remove(idx);
                                }

                                self.sync_view()?;
                            }
                            Command::ChangePinName(idx, name) => {
                                if let Some(pin) = self.pins.get_mut(idx) {
                                    pin.set_name(name);
                                }

                                self.sync_view()?;
                            }
                            Command::ChangePinDirectory(idx, pathbuf) => {
                                if let Some(pin) = self.pins.get_mut(idx) {
                                    pin.set_path(pathbuf);
                                }

                                self.sync_view()?;
                            }
                            Command::ChangePinPosition(from, to, pin) => {
                                if from < self.pins.len() && to < self.pins.len() {
                                    let pin_temp = self.pins[to].clone();
                                    self.pins[to] = pin.clone();
                                    self.pins[from] = pin_temp;
                                }

                                self.sync_view()?;
                            }
                            Command::NewPin(idx, pin) => {
                                if let Some(idx) = idx {
                                    if let Some(pin_mut) = self.pins.get_mut(idx) {
                                        *pin_mut = pin;
                                    }
                                } else {
                                    self.pins.push(pin);
                                }

                                self.sync_view()?;
                            }

                            // Tab
                            Command::CreateTab(tab_kind, mut pathbuf) => {
                                if pathbuf.exists() {
                                    if !pathbuf.is_dir() {
                                        if let Some(parent) = pathbuf.parent() {
                                            pathbuf = parent.to_path_buf();
                                        }
                                    }
                                    self.model.send(Command::CreateTab(tab_kind, pathbuf))?
                                } else {
                                    self.view.send(Command::Error(Box::new(JujikError::Other(
                                        format!("Path {:?} does not exist", pathbuf),
                                    ))))?;
                                }
                            }
                            Command::ChangeTabDirectory(idx, tab, pathbuf) => {
                                if self.tabs.get(idx).is_some() {
                                    self.model
                                        .send(Command::ChangeTabDirectory(idx, tab, pathbuf))?;
                                }
                            }
                            Command::NewTab(idx, tab) => {
                                if let Some(idx) = idx {
                                    if let Some(tab_mut) = self.tabs.get_mut(idx) {
                                        *tab_mut = tab;
                                    }
                                } else {
                                    self.tabs.push(tab);
                                }

                                self.sync_view()?;
                            }

                            // Other
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

    fn sync_view(&self) -> Result<(), JujikError> {
        Ok(self
            .view
            .send(Command::Sync(self.pins.clone(), self.tabs.clone()))?)
    }

    fn send_drop(&self) -> Result<(), JujikError> {
        let _view_drop = self.view.send(Command::Drop);
        let _model_drop = self.model.send(Command::Drop);
        Ok(())
    }
}
