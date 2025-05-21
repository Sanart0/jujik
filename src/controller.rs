use crate::{
    commands::Command,
    config::{self, Config},
    error::JujikError,
    pin::Pin,
    tab::Tab,
};
use std::{
    fs,
    sync::mpsc::{Receiver, Sender},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

pub struct JujikController {
    model: Sender<Command>,
    view: Sender<Command>,
    controller: Receiver<Command>,
    config: Config,
    update: Instant,
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

        let config = match Config::load() {
            Ok(config) => config,
            Err(_) => {
                //TODO Handle error
                Config::default()
            }
        };

        Self {
            model,
            view,
            controller,
            config: config.clone(),
            update: Instant::now(),
            pins: config.pins(),
            tabs: config.tabs(),
        }
    }

    pub fn run(mut self) -> Result<JoinHandle<Result<(), JujikError>>, JujikError> {
        Ok(thread::Builder::new()
            .name("Controller".to_string())
            .spawn(move || -> Result<(), JujikError> {
                self.send_config()?;

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
                            Command::DeletePin(idx_d, pin_d) => {
                                for (idx, pin) in self.pins.clone().iter().enumerate() {
                                    if idx == idx_d && pin.eq(&pin_d) {
                                        self.pins.remove(idx_d);
                                    }
                                }

                                self.sync_view()?;
                            }
                            Command::ChangePinName(idx, pin, name) => {
                                self.model.send(Command::ChangePinName(idx, pin, name))?;
                            }
                            Command::ChangePinDirectory(idx, pin, pathbuf) => {
                                self.model
                                    .send(Command::ChangePinDirectory(idx, pin, pathbuf))?;
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
                            Command::CreateTab(tab_kind, pathbuf) => {
                                if pathbuf.exists() {
                                    self.model.send(Command::CreateTab(tab_kind, pathbuf))?
                                } else {
                                    self.view.send(Command::Error(Box::new(JujikError::Other(
                                        format!("Path {:?} does not exist", pathbuf),
                                    ))))?;
                                }
                            }
                            Command::DeleteTab(idx_d, tab_d) => {
                                for (idx, tab) in self.tabs.clone().iter().enumerate() {
                                    if idx == idx_d && tab.eq(&tab_d) {
                                        self.tabs.remove(idx_d);
                                    }
                                }

                                self.sync_view()?;
                            }
                            Command::ChangeTabName(idx, tab, name) => {
                                self.model.send(Command::ChangeTabName(idx, tab, name))?;
                            }
                            Command::ChangeTabDirectory(idx, tab, pathbuf) => {
                                self.model
                                    .send(Command::ChangeTabDirectory(idx, tab, pathbuf))?;
                            }
                            Command::ChangeTabPosition(from, to, tab) => {
                                let tab_temp = self.tabs[to].clone();
                                self.tabs[to] = tab.clone();
                                self.tabs[from] = tab_temp;

                                self.sync_view()?;
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

                            // Entity
                            Command::CreateEntity(idx, tab, entity_ghost) => {
                                self.model
                                    .send(Command::CreateEntity(idx, tab, entity_ghost))?;
                            }
                            Command::DeleteEntitys(idx, tab, entitys) => {
                                self.model.send(Command::DeleteEntitys(idx, tab, entitys))?;
                            }
                            Command::CopyEntitys(idx_tab, tab, idx_entity, entitys, pathbuf) => {
                                if pathbuf.exists() {
                                    if pathbuf.is_dir() {
                                        self.model.send(Command::CopyEntitys(
                                            idx_tab, tab, idx_entity, entitys, pathbuf,
                                        ))?;
                                    } else {
                                        //TODO hande error path does not a directory
                                    }
                                } else {
                                    //TODO hande error path does not exist
                                }
                            }
                            Command::MoveEntitys(idx_tab, tab, idx_entity, entitys, pathbuf) => {
                                if pathbuf.exists() {
                                    if pathbuf.is_dir() {
                                        self.model.send(Command::MoveEntitys(
                                            idx_tab, tab, idx_entity, entitys, pathbuf,
                                        ))?;
                                    } else {
                                        //TODO hande error path does not a directory
                                    }
                                } else {
                                    //TODO hande error path does not exist
                                }
                            }
                            Command::ChangeEntityName(idx_tab, tab, idx_entity, entity, name) => {
                                if entity.exists() {
                                    self.model.send(Command::ChangeEntityName(
                                        idx_tab, tab, idx_entity, entity, name,
                                    ))?;
                                }
                            }
                            Command::ChangeEntityExtension(
                                idx_tab,
                                tab,
                                idx_entity,
                                entity,
                                extension,
                            ) => {
                                if entity.exists() {
                                    self.model.send(Command::ChangeEntityExtension(
                                        idx_tab, tab, idx_entity, entity, extension,
                                    ))?;
                                }
                            }
                            Command::ChangeEntityPermissions(
                                idx_tab,
                                tab,
                                idx_entity,
                                entity,
                                permissions,
                            ) => {
                                if entity.exists() {
                                    self.model.send(Command::ChangeEntityPermissions(
                                        idx_tab,
                                        tab,
                                        idx_entity,
                                        entity,
                                        permissions,
                                    ))?;
                                }
                            }
                            Command::ChangeEntityOwners(
                                idx_tab,
                                tab,
                                idx_entity,
                                entity,
                                owners,
                            ) => {
                                if entity.exists() {
                                    self.model.send(Command::ChangeEntityOwners(
                                        idx_tab, tab, idx_entity, entity, owners,
                                    ))?;
                                }
                            }
                            Command::ChangeEntityContent(idx, tab, entity, content) => {
                                if entity.exists() {
                                    self.model.send(Command::ChangeEntityContent(
                                        idx, tab, entity, content,
                                    ))?;
                                }
                            }

                            // Config
                            Command::SetConfig(config) => {
                                self.config = config;
                                self.write_config();
                            }

                            // Other
                            Command::Uptade => {
                                self.update_tabs()?;
                                self.sync_view()?;
                            }
                            Command::Error(err) => self.view.send(Command::Error(err))?,
                            Command::Drop => {
                                self.write_config();
                                self.send_drop()?;
                                break 'event_loop;
                            }
                            _ => {}
                        }
                    };

                    if self.update.elapsed() >= Duration::from_secs(5) {
                        self.update_tabs()?;
                        self.sync_view()?;
                        self.view.send(Command::GetConfig)?;

                        self.update = Instant::now();
                    }

                    std::thread::sleep(std::time::Duration::from_millis(10));
                }

                Ok(self.send_drop()?)
            })?)
    }

    fn write_config(&mut self) {
        self.config.set_tabs(
            self.tabs
                .clone()
                .into_iter()
                .map(|mut t| {
                    t.clear_entitys();
                    t
                })
                .collect(),
        );

        if let Err(_) = self.config.write() {
            //TODO Handle error
        }
    }

    fn send_config(&mut self) -> Result<(), JujikError> {
        self.update_tabs()?;
        self.sync_view()?;

        Ok(self.view.send(Command::SetConfig(self.config.clone()))?)
    }

    fn update_tabs(&mut self) -> Result<(), JujikError> {
        for tab in &mut self.tabs {
            if let Err(_) = tab.update_entitys() {
                //TODO Handle error
            }
        }

        Ok(())
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
