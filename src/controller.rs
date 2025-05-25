use crate::{commands::Command, config::Config, error::JujikError};
use std::{
    sync::mpsc::{Receiver, Sender},
    thread::{self, JoinHandle},
};

pub struct JujikController {
    model: Sender<Command>,
    view: Sender<Command>,
    controller: Receiver<Command>,
    config: Config,
}
impl JujikController {
    pub fn new(
        model: Sender<Command>,
        view: Sender<Command>,
        controller: Receiver<Command>,
    ) -> Self {
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
            config: config,
        }
    }

    pub fn run(mut self) -> Result<JoinHandle<Result<(), JujikError>>, JujikError> {
        Ok(thread::Builder::new()
            .name("Controller".to_string())
            .spawn(move || -> Result<(), JujikError> {
                // self.update_tabs()?;
                self.view.send(Command::SetConfig(self.config.clone()))?;

                'event_loop: loop {
                    if let Ok(command) = self.controller.try_recv() {
                        #[cfg(feature = "print_command")]
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
                                for (idx, pin) in self.config.pins.clone().iter().enumerate() {
                                    if idx == idx_d && pin.eq(&pin_d) {
                                        self.config.pins.remove(idx_d);
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
                                if from < self.config.pins.len() && to < self.config.pins.len() {
                                    let pin_temp = self.config.pins[to].clone();
                                    self.config.pins[to] = pin.clone();
                                    self.config.pins[from] = pin_temp;
                                }

                                self.sync_view()?;
                            }
                            Command::NewPin(idx, pin) => {
                                if let Some(idx) = idx {
                                    if let Some(pin_mut) = self.config.pins.get_mut(idx) {
                                        *pin_mut = pin;
                                    }
                                } else {
                                    self.config.pins.push(pin);
                                }

                                self.sync_view()?;
                            }

                            // Tab
                            Command::CreateEntitys(pathbuf) => {
                                if pathbuf.exists() {
                                    self.model.send(Command::CreateEntitys(pathbuf))?
                                } else {
                                    self.view.send(Command::Error(Box::new(JujikError::Other(
                                        format!("Path {:?} does not exist", pathbuf),
                                    ))))?;
                                }
                            }
                            Command::CreateView(pathbuf) => {
                                if pathbuf.exists() {
                                    self.model.send(Command::CreateView(pathbuf))?
                                } else {
                                    self.view.send(Command::Error(Box::new(JujikError::Other(
                                        format!("Path {:?} does not exist", pathbuf),
                                    ))))?;
                                }
                            }
                            Command::CreateEditor(pathbuf) => {
                                if pathbuf.exists() {
                                    self.model.send(Command::CreateEditor(pathbuf))?
                                } else {
                                    self.view.send(Command::Error(Box::new(JujikError::Other(
                                        format!("Path {:?} does not exist", pathbuf),
                                    ))))?;
                                }
                            }
                            Command::CreateFinder(parameters) => {
                                if parameters.path.exists() {
                                    self.model.send(Command::CreateFinder(parameters))?
                                } else {
                                    self.view.send(Command::Error(Box::new(JujikError::Other(
                                        format!("Path {:?} does not exist", parameters.path),
                                    ))))?;
                                }
                            }
                            Command::DeleteTab(idx_d, tab_d) => {
                                for (idx, tab) in self.config.tabs.clone().iter().enumerate() {
                                    if idx == idx_d && tab.eq(&tab_d) {
                                        self.config.tabs.remove(idx_d);
                                    }
                                }

                                self.sync_view()?;
                            }
                            Command::UpdateTab(idx) => {
                                if let Some(tab) = self.config.tabs.get_mut(idx) {
                                    if let Err(_) = tab.update_entitys() {
                                        //TODO Handle error
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
                                let tab_temp = self.config.tabs[to].clone();
                                self.config.tabs[to] = tab.clone();
                                self.config.tabs[from] = tab_temp;

                                self.sync_view()?;
                            }
                            Command::NewTab(idx, tab) => {
                                if let Some(idx) = idx {
                                    if let Some(tab_mut) = self.config.tabs.get_mut(idx) {
                                        *tab_mut = tab;
                                    }
                                } else {
                                    self.config.tabs.push(tab);
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
                            Command::ChangeEntitysSortBy(idx, tab, sordby) => {
                                self.model
                                    .send(Command::ChangeEntitysSortBy(idx, tab, sordby))?;
                            }

                            // Find
                            Command::UpdateFind(idx_tab, tab, parameters) => {}

                            // Config
                            Command::SetConfig(config) => {
                                self.config = config.clone();
                                self.write_config();
                                self.view.send(Command::SetConfig(config))?;
                            }

                            // Other
                            Command::Update => {
                                self.update_tabs()?;
                                self.view.send(Command::SetConfig(self.config.clone()))?;
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

                    std::thread::sleep(std::time::Duration::from_millis(10));
                }

                Ok(self.send_drop()?)
            })?)
    }

    fn write_config(&mut self) {
        self.config.tabs.iter_mut().for_each(|t| t.clear_entitys());

        if let Err(_) = self.config.write() {
            //TODO Handle error
        }
    }

    fn update_tabs(&mut self) -> Result<(), JujikError> {
        for tab in &mut self.config.tabs {
            if let Err(err) = tab.update_entitys() {
                //TODO Handle error
            }
        }

        Ok(())
    }

    fn root_access(&self) -> Result<(), JujikError> {
        Ok(())
    }

    fn sync_view(&self) -> Result<(), JujikError> {
        Ok(self.view.send(Command::Sync(
            self.config.pins.clone(),
            self.config.tabs.clone(),
        ))?)
    }

    fn send_drop(&self) -> Result<(), JujikError> {
        let _view_drop = self.view.send(Command::Drop);
        let _model_drop = self.model.send(Command::Drop);
        Ok(())
    }
}
