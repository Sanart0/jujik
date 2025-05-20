use crate::{commands::Command, error::JujikError, pin::Pin, tab::Tab};
use std::{
    fs,
    os::unix,
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

    pub fn run(mut self) -> Result<JoinHandle<Result<(), JujikError>>, JujikError> {
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
                                        self.controller.send(Command::NewPin(None, new_pin))?;
                                    }
                                    Err(err) => {
                                        self.controller.send(Command::Error(Box::new(err)))?;
                                    }
                                }
                            }
                            Command::ChangePinName(idx, mut pin, name) => {
                                pin.set_name(name);

                                self.controller.send(Command::NewPin(Some(idx), pin))?;
                            }
                            Command::ChangePinDirectory(idx, mut pin, pathbuf) => {
                                pin.set_path(pathbuf);

                                self.controller.send(Command::NewPin(Some(idx), pin))?;
                            }

                            // Tab
                            Command::CreateTab(tab_kind, pathbuf) => {
                                let new_tab = Tab::new(tab_kind, pathbuf);
                                match new_tab {
                                    Ok(new_tab) => {
                                        self.controller.send(Command::NewTab(None, new_tab))?;
                                    }
                                    Err(err) => {
                                        self.controller.send(Command::Error(Box::new(err)))?;
                                    }
                                }
                            }
                            Command::ChangeTabName(idx, mut tab, name) => {
                                tab.set_name(name);

                                self.controller.send(Command::NewTab(Some(idx), tab))?;
                            }
                            Command::ChangeTabDirectory(idx, mut tab, pathbuf) => {
                                let res = if let Some(pathbuf) = pathbuf {
                                    tab.change_dir(pathbuf)
                                } else {
                                    tab.change_dir_back()
                                };

                                match res {
                                    Ok(_) => {
                                        self.controller.send(Command::NewTab(Some(idx), tab))?;
                                    }
                                    Err(err) => {
                                        self.controller.send(Command::Error(Box::new(err)))?;
                                    }
                                };
                            }

                            // Entity
                            Command::CreateEntity(idx, tab, entity_ghost) => {}
                            Command::DeleteEntitys(idx, tab, entitys) => {
                                for entity in entitys {
                                    let res = if entity.is_dir() {
                                        fs::remove_file(entity.path())
                                    } else {
                                        fs::remove_dir_all(entity.path())
                                    };

                                    if let Err(_) = res {
                                        //TODO Handle error
                                    }
                                }

                                self.controller.send(Command::Uptade)?;
                            }
                            Command::CopyEntitys(idx_tab, tab, idx_entity, entitys, pathbuf) => {
                                for entity in entitys {
                                    let mut pathbuf = pathbuf.clone();
                                    pathbuf.push(entity.name_with_extension());

                                    let res = fs::copy(entity.path(), pathbuf);

                                    if let Err(_) = res {
                                        //TODO Handle error
                                    }
                                }

                                self.controller.send(Command::Uptade)?;
                            }
                            Command::MoveEntitys(idx_tab, tab, idx_entity, entitys, pathbuf) => {
                                for entity in entitys {
                                    let mut pathbuf = pathbuf.clone();
                                    pathbuf.push(entity.name_with_extension());

                                    let res = fs::rename(entity.path(), pathbuf.clone());

                                    if let Err(_) = res {
                                        //TODO Handle error
                                    }
                                }

                                self.controller.send(Command::Uptade)?;
                            }
                            Command::ChangeEntityName(idx_tab, tab, idx_entity, entity, name) => {
                                let mut path = entity.path_dir();
                                path.push(name);

                                let res = fs::rename(entity.path(), path);

                                if let Err(_) = res {
                                    //TODO Handle error
                                } else {
                                    self.controller.send(Command::Uptade)?;
                                }
                            }
                            Command::ChangeEntityExtension(
                                idx_tab,
                                tab,
                                idx_entity,
                                entity,
                                extension,
                            ) => {
                                let mut path = entity.path_dir();
                                path.push(entity.name_with_extension() + extension.as_str());

                                let res = fs::rename(entity.path(), path);

                                if let Err(_) = res {
                                    //TODO Handle error
                                } else {
                                    self.controller.send(Command::Uptade)?;
                                }
                            }
                            Command::ChangeEntityPermissions(
                                idx_tab,
                                tab,
                                idx_entity,
                                entity,
                                permissions,
                            ) => {
                                let res = fs::set_permissions(entity.path(), permissions.into());

                                if let Err(_) = res {
                                    //TODO Handle error
                                } else {
                                    self.controller.send(Command::Uptade)?;
                                }
                            }
                            Command::ChangeEntityOwners(
                                idx_tab,
                                tab,
                                idx_entity,
                                entity,
                                owners,
                            ) => {
                                let res = unix::fs::chown(
                                    entity.path(),
                                    Some(owners.uid()),
                                    Some(owners.gid()),
                                );

                                if let Err(_) = res {
                                    //TODO Handle error
                                } else {
                                    self.controller.send(Command::Uptade)?;
                                }
                            }

                            // Other
                            Command::Drop => break 'event_loop,
                            _ => {}
                        }
                    };

                    std::thread::sleep(std::time::Duration::from_millis(8));
                }

                Ok(self.send_drop()?)
            },
        )?)
    }

    fn send_drop(&self) -> Result<(), JujikError> {
        let _controller_drop = self.controller.send(Command::Drop);
        Ok(())
    }
}
