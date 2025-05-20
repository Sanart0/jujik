use crate::entity::Entity;
use crate::entity::kind::EntityKind;
use crate::entity::owner::EntityOwners;
use crate::entity::permission::{
    EntityPermissions, EntityPermissionsCategory, EntityPermissionsKind,
};
use crate::tab::{TabContent, TabKind};
use crate::{commands::Command, error::JujikError, pin::Pin, tab::Tab};
use eframe::{App, EventLoopBuilderHook, NativeOptions, run_native};
use egui::{
    Align, Button, CentralPanel, Color32, Context, Event, Id, Key, KeyboardShortcut, Label, Layout,
    Modal, Modifiers, PointerButton, Response, RichText, ScrollArea, Sense, SidePanel, Sides,
    Stroke, TopBottomPanel, Ui, Visuals, menu,
};
use egui_extras::{Column, TableBuilder};
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
    thread::{self, JoinHandle},
    usize,
};
use winit::platform::wayland::EventLoopBuilderExtWayland;

#[derive(Default)]
struct PinInfo {
    show: bool,
    idx: usize,
    pin: Pin,
    name: String,
    path: String,
}

#[derive(Default)]
struct TabInfo {
    show: bool,
    idx: usize,
    tab: Tab,
    name: String,
    path: String,
}

#[derive(Default)]
struct EntityInfo {
    show: bool,
    idx_tab: usize,
    tab: Tab,
    idx_entity: usize,
    entity: Entity,
    path: String,
    name: String,
    extension: String,
    kind: EntityKind,
    permissions: EntityPermissions,
    owners: EntityOwners,
    change_permissions: EntityChangePermissions,
    change_owners: EntityChangeOwners,
}

#[derive(Default)]
struct EntitysDelete {
    show: (bool, bool),
    idx_tab: usize,
    tab: Tab,
    idx_entity: usize,
    entitys: Vec<Entity>,
}

#[derive(Default)]
struct EntityChangePermissions {
    show: bool,
    user: (bool, bool, bool),
    group: (bool, bool, bool),
    other: (bool, bool, bool),
}

#[derive(Default)]
struct EntityChangeOwners {
    show: bool,
    uid: u32,
    gid: u32,
    username: String,
    groupname: String,
}

#[derive(Default, Debug)]
enum EntitysMoveKind {
    #[default]
    Select,
    Copy,
    Cut,
}

#[derive(Default)]
struct EntitysSelection {
    move_kind: EntitysMoveKind,
    entitys: HashSet<Entity>,
    last_idx: usize,
}

struct ShowEntitysColumn {
    filekind: bool,
    name: bool,
    name_with_extension: bool,
    extension: bool,
    permissions: bool,
    owners: bool,
    size: bool,
    modification_date: bool,
    creation_data: bool,
}

struct JujikStyle {
    primary_color: Color32,
    background_color: Color32,
    selection_color: Color32,
    text_color: Color32,
    text_size: f32,
}

pub struct JujikView {
    controller: Sender<Command>,
    view: Receiver<Command>,
    style: JujikStyle,
    pins: Vec<Pin>,
    tabs: Vec<Tab>,
    entitys_show: ShowEntitysColumn,
    current_tab_idx: usize,
    entitys_selection: EntitysSelection,
    pin_info: PinInfo,
    tab_info: TabInfo,
    entity_info: EntityInfo,
    entitys_delete: EntitysDelete,
}

impl JujikView {
    pub fn new(controller: Sender<Command>, view: Receiver<Command>) -> Self {
        Self {
            controller,
            style: JujikStyle::default(),
            view,
            pins: Vec::new(),
            tabs: Vec::new(),
            entitys_show: ShowEntitysColumn::default(),
            current_tab_idx: 0,
            entitys_selection: EntitysSelection::default(),
            pin_info: PinInfo::default(),
            tab_info: TabInfo::default(),
            entity_info: EntityInfo::default(),
            entitys_delete: EntitysDelete::default(),
        }
    }

    pub fn run(self) -> Result<JoinHandle<Result<(), JujikError>>, JujikError> {
        Ok(thread::Builder::new().name("View".to_string()).spawn(
            move || -> Result<(), JujikError> {
                let event_loop_builder: Option<EventLoopBuilderHook> =
                    Some(Box::new(|event_loop_builder| {
                        event_loop_builder.with_any_thread(true);
                    }));
                let native_options = NativeOptions {
                    event_loop_builder,
                    ..Default::default()
                };
                run_native("Jujik", native_options, Box::new(|_cc| Ok(Box::new(self))))?;

                Ok(())
            },
        )?)
    }

    fn write_save(&self) {
        //TODO write a save
    }

    fn handle_commad(&mut self, ctx: &egui::Context) -> Result<(), JujikError> {
        if let Ok(command) = self.view.try_recv() {
            println!("View: {:?}", command);

            match command {
                // Other
                Command::Sync(pins, tabs) => {
                    self.pins.clone_from(&pins);
                    self.tabs.clone_from(&tabs);
                }
                Command::Drop => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                _ => {}
            }
        };
        Ok(())
    }

    fn style(&self, ctx: &egui::Context) {
        let mut visuals = Visuals::dark();

        visuals.widgets.noninteractive.bg_fill = self.style.background_color;
        visuals.widgets.inactive.bg_fill = self.style.primary_color.linear_multiply(0.7);
        visuals.widgets.hovered.bg_fill = self.style.primary_color.linear_multiply(0.8);
        visuals.widgets.active.bg_fill = self.style.primary_color;

        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.style.text_color);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.style.text_color);
        visuals.extreme_bg_color = self.style.background_color;
        visuals.selection.bg_fill = self.style.selection_color;

        ctx.set_visuals(visuals);
    }

    fn mesaage(&self) {}
}

impl App for JujikView {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.style(ctx);

        TopBottomPanel::top("menu").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button(
                    RichText::new("File")
                        .color(self.style.text_color)
                        .size(self.style.text_size),
                    |ui| {
                        if ui
                            .button(
                                RichText::new("Exit")
                                    .color(self.style.text_color)
                                    .size(self.style.text_size),
                            )
                            .clicked()
                        {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    },
                );
                if ui
                    .button(
                        RichText::new("ROOT")
                            .color(self.style.text_color)
                            .size(self.style.text_size),
                    )
                    .clicked()
                {
                    let _ = self
                        .controller
                        .send(Command::CreatePin(PathBuf::from("/")))
                        .inspect_err(JujikError::handle_err);
                }
                if ui
                    .button(
                        RichText::new("HOME")
                            .color(self.style.text_color)
                            .size(self.style.text_size),
                    )
                    .clicked()
                {
                    let _ = self
                        .controller
                        .send(Command::CreatePin(PathBuf::from("/home/sanart0/")))
                        .inspect_err(JujikError::handle_err);
                }
                if ui
                    .button(
                        RichText::new("TEST")
                            .color(self.style.text_color)
                            .size(self.style.text_size),
                    )
                    .clicked()
                {
                    let _ = self
                        .controller
                        .send(Command::CreatePin(PathBuf::from(
                            "/home/sanart0/KPI/4/IPZ-Kursach/jujik/test/",
                        )))
                        .inspect_err(JujikError::handle_err);
                }
            });
        });

        SidePanel::left("Bind")
            .width_range(100.0..=300.0)
            .show(ctx, |ui| {
                self.pin(ui, ctx);
            });

        CentralPanel::default().show(ctx, |ui| {
            TopBottomPanel::top("tab").show_inside(ui, |ui| {
                self.tab(ui, ctx);
            });
            CentralPanel::default().show_inside(ui, |ui| {
                self.tab_content(ctx, ui);
            });
        });

        let _ = self.handle_commad(ctx).inspect_err(JujikError::handle_err);

        //TODO meybe do not need
        ctx.request_repaint();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        Self::write_save(self);
        let _ = self
            .controller
            .send(Command::Drop)
            .inspect_err(JujikError::handle_err);
    }
}

// Pin
impl JujikView {
    fn pin(&mut self, ui: &mut Ui, ctx: &Context) {
        ui.label(
            RichText::new("Pin")
                .color(self.style.text_color)
                .size(self.style.text_size),
        );

        ui.separator();

        for (idx, pin) in self.pins.clone().iter().enumerate() {
            let response = ui.add(
                Button::new(
                    RichText::new(pin.name())
                        .color(self.style.text_color)
                        .size(self.style.text_size),
                )
                .fill(self.style.background_color),
            );

            if response.clicked() {
                let _ = self
                    .controller
                    .send(Command::CreateTab(TabKind::Entitys, pin.path()))
                    .inspect_err(JujikError::handle_err);
            }

            self.pin_context_menu(&response, idx, pin);
        }

        if self.pin_info.show {
            self.pin_info(ctx);
        }
    }

    fn pin_context_menu(&mut self, response: &Response, idx: usize, pin: &Pin) {
        response.context_menu(|ui| {
            let open = ui.button(
                RichText::new("Open")
                    .color(self.style.text_color)
                    .size(self.style.text_size),
            );

            let _position = ui.menu_button(
                RichText::new("Position")
                    .color(self.style.text_color)
                    .size(self.style.text_size),
                |ui| {
                    let up = ui.button(
                        RichText::new("Up")
                            .color(self.style.text_color)
                            .size(self.style.text_size),
                    );

                    let down = ui.button(
                        RichText::new("Down")
                            .color(self.style.text_color)
                            .size(self.style.text_size),
                    );

                    if up.clicked() {
                        if idx > 0 {
                            let _ = self
                                .controller
                                .send(Command::ChangePinPosition(idx, idx - 1, pin.clone()))
                                .inspect_err(JujikError::handle_err);
                        }
                    }

                    if down.clicked() {
                        if idx < self.pins.len() - 1 {
                            let _ = self
                                .controller
                                .send(Command::ChangePinPosition(idx, idx + 1, pin.clone()))
                                .inspect_err(JujikError::handle_err);
                        }
                    }
                },
            );

            let delete = ui.button(
                RichText::new("Delete")
                    .color(self.style.text_color)
                    .size(self.style.text_size),
            );

            let info = ui.button(
                RichText::new("Info")
                    .color(self.style.text_color)
                    .size(self.style.text_size),
            );

            if open.clicked() {
                let _ = self
                    .controller
                    .send(Command::CreateTab(TabKind::Entitys, pin.path()))
                    .inspect_err(JujikError::handle_err);

                ui.close_menu();
            }

            if delete.clicked() {
                let _ = self
                    .controller
                    .send(Command::DeletePin(idx, pin.clone()))
                    .inspect_err(JujikError::handle_err);

                ui.close_menu();
            }

            if info.clicked() {
                self.pin_info.show = true;
                self.pin_info.idx = idx;
                self.pin_info.pin = pin.clone();
                self.pin_info.name = self.pin_info.pin.name();
                self.pin_info.path = self.pin_info.pin.path_str();

                ui.close_menu();
            }
        });
    }

    fn pin_info(&mut self, ctx: &Context) {
        let modal = Modal::new(Id::new(format!("Pin Info: {}", self.pin_info.pin.name()))).show(
            ctx,
            |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.label(
                        RichText::new(format!("Pin Info: {}", self.pin_info.name))
                            .color(self.style.text_color)
                            .size(self.style.text_size),
                    );

                    ui.separator();

                    Sides::new().show(
                        ui,
                        |ui| {
                            ui.label(
                                RichText::new("Name: ")
                                    .color(self.style.text_color)
                                    .size(self.style.text_size),
                            );
                        },
                        |ui| {
                            ui.text_edit_singleline(&mut self.pin_info.name);
                        },
                    );

                    Sides::new().show(
                        ui,
                        |ui| {
                            ui.label(
                                RichText::new("Path: ")
                                    .color(self.style.text_color)
                                    .size(self.style.text_size),
                            );
                        },
                        |ui| {
                            ui.text_edit_singleline(&mut self.pin_info.path);
                        },
                    );

                    ui.separator();

                    Sides::new().show(
                        ui,
                        |_ui| {},
                        |ui| {
                            if ui
                                .button(
                                    RichText::new("Save")
                                        .color(self.style.text_color)
                                        .size(self.style.text_size),
                                )
                                .clicked()
                            {
                                if self.pin_info.pin.name().ne(&self.pin_info.name) {
                                    let _ = self
                                        .controller
                                        .send(Command::ChangePinName(
                                            self.pin_info.idx,
                                            self.pin_info.pin.clone(),
                                            self.pin_info.name.clone(),
                                        ))
                                        .inspect_err(JujikError::handle_err);
                                }

                                if self.pin_info.pin.path_str().ne(&self.pin_info.path) {
                                    let _ = self
                                        .controller
                                        .send(Command::ChangePinDirectory(
                                            self.pin_info.idx,
                                            self.pin_info.pin.clone(),
                                            PathBuf::from(self.pin_info.path.clone()),
                                        ))
                                        .inspect_err(JujikError::handle_err);
                                }

                                self.pin_info.show = false;
                            }
                        },
                    );
                })
            },
        );

        if modal.backdrop_response.clicked() {
            self.pin_info.show = false;
        }
    }
}

// Tab
impl JujikView {
    fn tab(&mut self, ui: &mut Ui, ctx: &Context) {
        let mut scroll = ui.ctx().style().spacing.scroll;
        scroll.floating = false;
        scroll.bar_width = 4.0;
        scroll.bar_inner_margin = 4.0;
        scroll.bar_outer_margin = 4.0;
        scroll.foreground_color = false;
        ui.ctx().all_styles_mut(|s| s.spacing.scroll = scroll);

        ui.horizontal(|ui| {
            ScrollArea::horizontal().show(ui, |ui| {
                for (idx, tab) in self.tabs.clone().iter().enumerate() {
                    let response = ui.selectable_label(
                        self.current_tab_idx == idx,
                        RichText::new(tab.name())
                            .color(self.style.text_color)
                            .size(self.style.text_size),
                    );

                    if response.clicked() {
                        self.current_tab_idx = idx;
                        self.entitys_selection.entitys.clear();
                    }

                    self.tab_context_menu(ui, &response, idx, tab);
                }

                if self.tab_info.show {
                    self.tab_info(ctx);
                }
            });
        });
    }

    fn tab_content(&mut self, ctx: &Context, ui: &mut Ui) {
        if self.current_tab_idx < self.tabs.len() {
            if let Some(tab) = self.tabs.get(self.current_tab_idx) {
                match tab.content() {
                    TabContent::Entitys(_) => {
                        ui.horizontal(|ui| {
                            self.go_prev_directory(ui, tab.clone());
                            self.tab_path(ui, tab.clone());
                        });

                        ui.separator();

                        self.table(ctx, ui, self.current_tab_idx, tab.clone())
                    }
                    TabContent::Editor(_pathbuf) => todo!(),
                    _ => {}
                }
            }
        }
    }

    fn table(&mut self, ctx: &Context, ui: &mut Ui, idx_tab: usize, tab: Tab) {
        let mut scroll = ui.ctx().style().spacing.scroll;
        scroll.floating = false;
        scroll.bar_width = 4.0;
        scroll.bar_inner_margin = 4.0;
        scroll.bar_outer_margin = 4.0;
        scroll.foreground_color = false;
        ui.ctx().all_styles_mut(|s| s.spacing.scroll = scroll);

        let mut responses: Vec<Option<Response>> = Vec::new();

        if let Some(entitys) = tab.entitys() {
            ScrollArea::horizontal().show(ui, |ui| {
                TableBuilder::new(ui)
                    // .resizable(true)
                    .cell_layout(Layout::left_to_right(Align::Center))
                    .sense(Sense::click())
                    .striped(true)
                    .column(Column::exact(40.0))
                    .column(Column::remainder())
                    .column(Column::remainder())
                    .column(Column::remainder())
                    .column(Column::remainder())
                    .column(Column::remainder())
                    .column(Column::remainder())
                    .column(Column::remainder())
                    .column(Column::remainder())
                    .header(30.0, |mut header| {
                        header.col(|ui| {});
                        if self.entitys_show.filekind {
                            header.col(|ui| {
                                ui.centered_and_justified(|ui| {
                                    ui.label(
                                        RichText::new("Kind")
                                            .color(self.style.text_color)
                                            .size(self.style.text_size),
                                    );
                                });
                            });
                        }
                        if self.entitys_show.name {
                            header.col(|ui| {
                                ui.centered_and_justified(|ui| {
                                    ui.label(
                                        RichText::new("Name")
                                            .color(self.style.text_color)
                                            .size(self.style.text_size),
                                    );
                                });
                            });
                        }
                        if self.entitys_show.extension {
                            header.col(|ui| {
                                ui.centered_and_justified(|ui| {
                                    ui.label(
                                        RichText::new("Extension")
                                            .color(self.style.text_color)
                                            .size(self.style.text_size),
                                    );
                                });
                            });
                        }
                        if self.entitys_show.permissions {
                            header.col(|ui| {
                                ui.centered_and_justified(|ui| {
                                    ui.label(
                                        RichText::new("Permissions")
                                            .color(self.style.text_color)
                                            .size(self.style.text_size),
                                    );
                                });
                            });
                        }
                        if self.entitys_show.owners {
                            header.col(|ui| {
                                ui.centered_and_justified(|ui| {
                                    ui.label(
                                        RichText::new("Owners")
                                            .color(self.style.text_color)
                                            .size(self.style.text_size),
                                    );
                                });
                            });
                        }
                        if self.entitys_show.size {
                            header.col(|ui| {
                                ui.centered_and_justified(|ui| {
                                    ui.label(
                                        RichText::new("Size")
                                            .color(self.style.text_color)
                                            .size(self.style.text_size),
                                    );
                                });
                            });
                        }
                        if self.entitys_show.modification_date {
                            header.col(|ui| {
                                ui.centered_and_justified(|ui| {
                                    ui.label(
                                        RichText::new("Modification Date")
                                            .color(self.style.text_color)
                                            .size(self.style.text_size),
                                    );
                                });
                            });
                        }
                        if self.entitys_show.creation_data {
                            header.col(|ui| {
                                ui.centered_and_justified(|ui| {
                                    ui.label(
                                        RichText::new("Creation Data")
                                            .color(self.style.text_color)
                                            .size(self.style.text_size),
                                    );
                                });
                            });
                        }
                    })
                    .body(|mut body| {
                        for (idx_entity, entity) in entitys.iter().enumerate() {
                            body.row(40.0, |mut row| {
                                row.set_selected(self.entitys_selection.entitys.contains(&entity));

                                row.col(|ui| {});
                                if self.entitys_show.filekind {
                                    row.col(|ui| {
                                        ui.centered_and_justified(|ui| {
                                            ui.label(
                                                RichText::new(format!("{}", entity.kind()))
                                                    .color(self.style.text_color)
                                                    .size(self.style.text_size),
                                            );
                                        });
                                    });
                                }
                                if self.entitys_show.name {
                                    row.col(|ui| {
                                        ui.centered_and_justified(|ui| {
                                            let name = if self.entitys_show.name_with_extension {
                                                entity.name_with_extension()
                                            } else {
                                                entity.name()
                                            };

                                            ui.label(
                                                RichText::new(format!("{}", name))
                                                    .color(self.style.text_color)
                                                    .size(self.style.text_size),
                                            );
                                        });
                                    });
                                }
                                if self.entitys_show.extension {
                                    row.col(|ui| {
                                        ui.centered_and_justified(|ui| {
                                            ui.label(
                                                RichText::new(format!(
                                                    "{}",
                                                    entity.extension_str()
                                                ))
                                                .color(self.style.text_color)
                                                .size(self.style.text_size),
                                            );
                                        });
                                    });
                                }
                                if self.entitys_show.permissions {
                                    row.col(|ui| {
                                        ui.centered_and_justified(|ui| {
                                            ui.label(
                                                RichText::new(format!("{}", entity.permissions()))
                                                    .color(self.style.text_color)
                                                    .size(self.style.text_size),
                                            );
                                        });
                                    });
                                }
                                if self.entitys_show.owners {
                                    row.col(|ui| {
                                        ui.centered_and_justified(|ui| {
                                            ui.label(
                                                RichText::new(format!("{}", entity.owners()))
                                                    .color(self.style.text_color)
                                                    .size(self.style.text_size),
                                            );
                                        });
                                    });
                                }
                                if self.entitys_show.size {
                                    row.col(|ui| {
                                        ui.centered_and_justified(|ui| {
                                            ui.label(
                                                RichText::new(format!("{}", 0))
                                                    .color(self.style.text_color)
                                                    .size(self.style.text_size),
                                            );
                                        });
                                    });
                                }
                                if self.entitys_show.modification_date {
                                    row.col(|ui| {
                                        ui.centered_and_justified(|ui| {
                                            ui.label(
                                                RichText::new(format!("{}", 0))
                                                    .color(self.style.text_color)
                                                    .size(self.style.text_size),
                                            );
                                        });
                                    });
                                }
                                if self.entitys_show.creation_data {
                                    row.col(|ui| {
                                        ui.centered_and_justified(|ui| {
                                            ui.label(
                                                RichText::new(format!("{}", 0))
                                                    .color(self.style.text_color)
                                                    .size(self.style.text_size),
                                            );
                                        });
                                    });
                                }

                                self.entity_context_menu(
                                    &row.response(),
                                    idx_tab,
                                    &tab,
                                    idx_entity,
                                    entity,
                                );

                                responses.push(Some(row.response()));
                            });
                        }

                        if self.entitys_delete.show.0 & self.entitys_delete.show.1 {
                            self.entity_delete(ctx);
                        }

                        if self.entity_info.show {
                            self.entity_info(ctx);
                        }

                        if self.entity_info.change_permissions.show {
                            self.entity_change_permissions(ctx);
                        }

                        if self.entity_info.change_owners.show {
                            self.entity_change_owners(ctx);
                        }
                    });
            });

            for (idx, entity) in entitys.iter().enumerate() {
                if let Some(response) = responses.get(idx) {
                    if let Some(response) = response {
                        self.toogle_selection_entity(ui, &response, idx, entity, &entitys);
                    }
                }
            }

            self.selection_entity_move(ctx, tab.path());
        }
    }

    fn tab_context_menu(&mut self, ui: &mut Ui, response: &Response, idx: usize, tab: &Tab) {
        response.context_menu(|ui| {
            let _position = ui.menu_button(
                RichText::new("Position")
                    .color(self.style.text_color)
                    .size(self.style.text_size),
                |ui| {
                    let left = ui.button(
                        RichText::new("Left")
                            .color(self.style.text_color)
                            .size(self.style.text_size),
                    );

                    let right = ui.button(
                        RichText::new("Rigth")
                            .color(self.style.text_color)
                            .size(self.style.text_size),
                    );

                    if left.clicked() {
                        if self.current_tab_idx > 0 {
                            let _ = self
                                .controller
                                .send(Command::ChangeTabPosition(idx, idx - 1, tab.clone()))
                                .inspect_err(JujikError::handle_err);

                            self.current_tab_idx = idx - 1;
                        }
                    }

                    if right.clicked() {
                        if self.current_tab_idx < self.tabs.len() - 1 {
                            let _ = self
                                .controller
                                .send(Command::ChangeTabPosition(idx, idx + 1, tab.clone()))
                                .inspect_err(JujikError::handle_err);

                            self.current_tab_idx = idx + 1;
                        }
                    }
                },
            );

            let delete = ui.button(
                RichText::new("Delete")
                    .color(self.style.text_color)
                    .size(self.style.text_size),
            );

            let info = ui.button(
                RichText::new("Info")
                    .color(self.style.text_color)
                    .size(self.style.text_size),
            );

            if delete.clicked() {
                let _ = self
                    .controller
                    .send(Command::DeleteTab(idx, tab.clone()))
                    .inspect_err(JujikError::handle_err);

                ui.close_menu();
            }

            if info.clicked() {
                self.tab_info.show = true;
                self.tab_info.idx = idx;
                self.tab_info.tab = tab.clone();
                self.tab_info.name = self.tab_info.tab.name();
                self.tab_info.path = self.tab_info.tab.path_str();

                ui.close_menu();
            }
        });
    }

    fn tab_info(&mut self, ctx: &Context) {
        let modal = Modal::new(Id::new(format!("Tab Info: {}", self.tab_info.tab.name()))).show(
            ctx,
            |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.label(
                        RichText::new(format!("Tab Info: {}", self.tab_info.name))
                            .color(self.style.text_color)
                            .size(self.style.text_size),
                    );

                    ui.separator();

                    Sides::new().show(
                        ui,
                        |ui| {
                            ui.label(
                                RichText::new("Name: ")
                                    .color(self.style.text_color)
                                    .size(self.style.text_size),
                            );
                        },
                        |ui| {
                            ui.text_edit_singleline(&mut self.tab_info.name);
                        },
                    );

                    Sides::new().show(
                        ui,
                        |ui| {
                            ui.label(
                                RichText::new("Path: ")
                                    .color(self.style.text_color)
                                    .size(self.style.text_size),
                            );
                        },
                        |ui| {
                            ui.text_edit_singleline(&mut self.tab_info.path);
                        },
                    );

                    ui.separator();

                    Sides::new().show(
                        ui,
                        |_ui| {},
                        |ui| {
                            if ui
                                .button(
                                    RichText::new("Save")
                                        .color(self.style.text_color)
                                        .size(self.style.text_size),
                                )
                                .clicked()
                            {
                                if self.tab_info.tab.name().ne(&self.tab_info.name) {
                                    let _ = self
                                        .controller
                                        .send(Command::ChangeTabName(
                                            self.tab_info.idx,
                                            self.tab_info.tab.clone(),
                                            self.tab_info.name.clone(),
                                        ))
                                        .inspect_err(JujikError::handle_err);
                                }

                                if self.tab_info.tab.path_str().ne(&self.tab_info.path) {
                                    let _ = self
                                        .controller
                                        .send(Command::ChangeTabDirectory(
                                            self.tab_info.idx,
                                            self.tab_info.tab.clone(),
                                            Some(PathBuf::from(self.tab_info.path.clone())),
                                        ))
                                        .inspect_err(JujikError::handle_err);
                                }

                                self.tab_info.show = false;
                            }
                        },
                    );
                })
            },
        );

        if modal.backdrop_response.clicked() {
            self.tab_info.show = false;
        }
    }

    fn go_next_directory(&self, tab: &Tab, pathbuf: PathBuf) {
        let _ = self
            .controller
            .send(Command::ChangeTabDirectory(
                self.current_tab_idx,
                tab.clone(),
                Some(pathbuf),
            ))
            .inspect_err(JujikError::handle_err);
    }

    fn go_prev_directory(&self, ui: &mut Ui, tab: Tab) {
        let tab_back = ui.add(Button::new(
            RichText::new("Back")
                .color(self.style.text_color)
                .size(self.style.text_size),
        ));

        if tab_back.clicked() || ui.input(|i| i.pointer.button_clicked(PointerButton::Extra1)) {
            let _ = self
                .controller
                .send(Command::ChangeTabDirectory(self.current_tab_idx, tab, None))
                .inspect_err(JujikError::handle_err);
        }
    }

    fn tab_path(&self, ui: &mut Ui, tab: Tab) {
        let tab_path = ui.add(
            Label::new(
                RichText::new(format!("{}", tab.path_str()))
                    .color(self.style.text_color)
                    .size(self.style.text_size),
            )
            .selectable(false)
            .sense(Sense::click()),
        );

        if tab_path.clicked() {}
    }
}

// Entity
impl JujikView {
    fn entity_context_menu(
        &mut self,
        response: &Response,
        idx_tab: usize,
        tab: &Tab,
        idx_entity: usize,
        entity: &Entity,
    ) {
        response.context_menu(|ui| {
            let open = ui.button(
                RichText::new("Open")
                    .color(self.style.text_color)
                    .size(self.style.text_size),
            );

            let select = ui.button(
                RichText::new("Select")
                    .color(self.style.text_color)
                    .size(self.style.text_size),
            );

            let delete = ui.button(
                RichText::new("Delete")
                    .color(self.style.text_color)
                    .size(self.style.text_size),
            );

            let info = ui.button(
                RichText::new("Info")
                    .color(self.style.text_color)
                    .size(self.style.text_size),
            );

            if open.clicked() {
                self.go_next_directory(tab, entity.path());

                ui.close_menu();
            }

            if select.clicked() {
                ui.close_menu();
            }

            if delete.clicked() {
                self.entitys_delete.show = (true, true);
                self.entitys_delete.idx_tab = idx_tab;
                self.entitys_delete.tab = tab.clone();
                self.entitys_delete.idx_entity = idx_entity;
                self.entitys_delete.entitys = self.entitys_selection.entitys_vec();

                ui.close_menu();
            }

            if info.clicked() {
                self.entity_info.show = true;
                self.entity_info.idx_tab = idx_tab;
                self.entity_info.tab = tab.clone();
                self.entity_info.idx_entity = idx_entity;
                self.entity_info.entity = entity.clone();
                self.entity_info.path = self.entity_info.entity.path_dir_str();
                self.entity_info.name = self.entity_info.entity.name();
                self.entity_info.extension = self.entity_info.entity.extension_str();
                self.entity_info.kind = self.entity_info.entity.kind().clone();
                self.entity_info.permissions = self.entity_info.entity.permissions().clone();
                self.entity_info.owners = self.entity_info.entity.owners().clone();

                ui.close_menu();
            }
        });
    }

    fn entity_delete(&mut self, ctx: &Context) {
        let modal = Modal::new(Id::new(format!(
            "Entity Delete: {}",
            self.entitys_delete.entitys.len()
        )))
        .show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.label(
                    RichText::new(format!(
                        "Are you sure to delete: {}",
                        self.entitys_delete
                            .entitys
                            .clone()
                            .iter()
                            .map(|e| e.name_with_extension())
                            .collect::<Vec<String>>()
                            .join(" ")
                    ))
                    .color(self.style.text_color)
                    .size(self.style.text_size),
                );

                ui.separator();

                let (show_0, show_1) = &mut self.entitys_delete.show;

                Sides::new().show(
                    ui,
                    |ui| {
                        if ui
                            .button(
                                RichText::new("Yes")
                                    .color(self.style.text_color)
                                    .size(self.style.text_size),
                            )
                            .clicked()
                        {
                            let _ = self
                                .controller
                                .send(Command::DeleteEntitys(
                                    self.entitys_delete.idx_tab,
                                    self.entitys_delete.tab.clone(),
                                    self.entitys_delete.entitys.clone(),
                                ))
                                .inspect_err(JujikError::handle_err);

                            *show_0 = false;
                        }
                    },
                    |ui| {
                        if ui
                            .button(
                                RichText::new("No")
                                    .color(self.style.text_color)
                                    .size(self.style.text_size),
                            )
                            .clicked()
                        {
                            *show_1 = false;
                        }
                    },
                );
            })
        });

        if modal.backdrop_response.clicked() {
            self.entitys_delete.show = (false, false);
        }
    }

    fn entity_info(&mut self, ctx: &Context) {
        let modal = Modal::new(Id::new(format!(
            "Entity Info: {}",
            self.entity_info.entity.name()
        )))
        .show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.label(
                    RichText::new(format!("Entity Info: {}", self.entity_info.name))
                        .color(self.style.text_color)
                        .size(self.style.text_size),
                );

                ui.separator();

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Path: ")
                                .color(self.style.text_color)
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.text_edit_singleline(&mut self.entity_info.path);
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Name: ")
                                .color(self.style.text_color)
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.text_edit_singleline(&mut self.entity_info.name);
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Extension: ")
                                .color(self.style.text_color)
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.text_edit_singleline(&mut self.entity_info.extension);
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Kind: ")
                                .color(self.style.text_color)
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.add(
                            Label::new(
                                RichText::new(format!("{:?}", self.entity_info.kind))
                                    .color(self.style.text_color)
                                    .size(self.style.text_size),
                            )
                            .selectable(true),
                        );
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Permissions: ")
                                .color(self.style.text_color)
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        let permissions = ui.add(
                            Label::new(
                                RichText::new(format!("{}", self.entity_info.permissions))
                                    .color(self.style.text_color)
                                    .size(self.style.text_size),
                            )
                            .selectable(true),
                        );

                        permissions.context_menu(|ui| {
                            let change = ui.button(
                                RichText::new("Change")
                                    .color(self.style.text_color)
                                    .size(self.style.text_size),
                            );

                            if change.clicked() {
                                self.entity_info.change_permissions.show = true;

                                self.entity_info.change_permissions.user = (
                                    self.entity_info.permissions.has(
                                        EntityPermissionsCategory::User,
                                        EntityPermissionsKind::Execute,
                                    ),
                                    self.entity_info.permissions.has(
                                        EntityPermissionsCategory::User,
                                        EntityPermissionsKind::Write,
                                    ),
                                    self.entity_info.permissions.has(
                                        EntityPermissionsCategory::User,
                                        EntityPermissionsKind::Read,
                                    ),
                                );
                                self.entity_info.change_permissions.group = (
                                    self.entity_info.permissions.has(
                                        EntityPermissionsCategory::Group,
                                        EntityPermissionsKind::Execute,
                                    ),
                                    self.entity_info.permissions.has(
                                        EntityPermissionsCategory::Group,
                                        EntityPermissionsKind::Write,
                                    ),
                                    self.entity_info.permissions.has(
                                        EntityPermissionsCategory::Group,
                                        EntityPermissionsKind::Read,
                                    ),
                                );
                                self.entity_info.change_permissions.other = (
                                    self.entity_info.permissions.has(
                                        EntityPermissionsCategory::Other,
                                        EntityPermissionsKind::Execute,
                                    ),
                                    self.entity_info.permissions.has(
                                        EntityPermissionsCategory::Other,
                                        EntityPermissionsKind::Write,
                                    ),
                                    self.entity_info.permissions.has(
                                        EntityPermissionsCategory::Other,
                                        EntityPermissionsKind::Read,
                                    ),
                                );
                            }
                        });
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Owners: ")
                                .color(self.style.text_color)
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        let owners = ui.add(
                            Label::new(
                                RichText::new(format!("{}", self.entity_info.owners))
                                    .color(self.style.text_color)
                                    .size(self.style.text_size),
                            )
                            .selectable(true),
                        );

                        owners.context_menu(|ui| {
                            let change = ui.button(
                                RichText::new("Change")
                                    .color(self.style.text_color)
                                    .size(self.style.text_size),
                            );

                            if change.clicked() {
                                self.entity_info.change_owners.show = true;

                                self.entity_info.change_owners.uid = self.entity_info.owners.uid();
                                self.entity_info.change_owners.gid = self.entity_info.owners.gid();
                                self.entity_info.change_owners.username =
                                    self.entity_info.owners.username();
                                self.entity_info.change_owners.groupname =
                                    self.entity_info.owners.groupname();
                            }
                        });
                    },
                );

                ui.separator();

                Sides::new().show(
                    ui,
                    |_ui| {},
                    |ui| {
                        if ui
                            .button(
                                RichText::new("Save")
                                    .color(self.style.text_color)
                                    .size(self.style.text_size),
                            )
                            .clicked()
                        {
                            if self
                                .entity_info
                                .entity
                                .path_str()
                                .ne(&self.entity_info.path)
                            {
                                // let _ = self
                                //     .controller
                                //     .send(Command::ChangeEntityDirectory(
                                //         self.entity_info.idx_tab,
                                //         self.entity_info.tab.clone(),
                                //         self.entity_info.idx_entity,
                                //         self.entity_info.entity.clone(),
                                //         PathBuf::from(self.entity_info.path.clone()),
                                //     ))
                                //     .inspect_err(JujikError::handle_err);
                            }

                            if self.entity_info.entity.name().ne(&self.entity_info.name) {
                                let _ = self
                                    .controller
                                    .send(Command::ChangeEntityName(
                                        self.entity_info.idx_tab,
                                        self.entity_info.tab.clone(),
                                        self.entity_info.idx_entity,
                                        self.entity_info.entity.clone(),
                                        self.entity_info.name.clone(),
                                    ))
                                    .inspect_err(JujikError::handle_err);
                            }

                            if self
                                .entity_info
                                .entity
                                .extension_str()
                                .ne(&self.entity_info.extension)
                            {
                                let _ = self
                                    .controller
                                    .send(Command::ChangeEntityExtension(
                                        self.entity_info.idx_tab,
                                        self.entity_info.tab.clone(),
                                        self.entity_info.idx_entity,
                                        self.entity_info.entity.clone(),
                                        self.entity_info.extension.clone(),
                                    ))
                                    .inspect_err(JujikError::handle_err);
                            }

                            if self
                                .entity_info
                                .entity
                                .permissions()
                                .ne(&self.entity_info.permissions)
                            {
                                let _ = self
                                    .controller
                                    .send(Command::ChangeEntityPermissions(
                                        self.entity_info.idx_tab,
                                        self.entity_info.tab.clone(),
                                        self.entity_info.idx_entity,
                                        self.entity_info.entity.clone(),
                                        self.entity_info.permissions.clone(),
                                    ))
                                    .inspect_err(JujikError::handle_err);
                            }

                            if self
                                .entity_info
                                .entity
                                .owners()
                                .ne(&self.entity_info.owners)
                            {
                                let _ = self
                                    .controller
                                    .send(Command::ChangeEntityOwners(
                                        self.entity_info.idx_tab,
                                        self.entity_info.tab.clone(),
                                        self.entity_info.idx_entity,
                                        self.entity_info.entity.clone(),
                                        self.entity_info.owners.clone(),
                                    ))
                                    .inspect_err(JujikError::handle_err);
                            }

                            self.tab_info.show = false;
                        }
                    },
                );
            })
        });

        if modal.backdrop_response.clicked() {
            self.entity_info.show = false;
        }
    }

    fn entity_change_permissions(&mut self, ctx: &Context) {
        let modal = Modal::new(Id::new(format!(
            "Entity Change Permissions: {:?}",
            self.entity_info.permissions
        )))
        .show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("User: ")
                                .color(self.style.text_color)
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.checkbox(
                            &mut self.entity_info.change_permissions.user.0,
                            RichText::new("execute")
                                .color(self.style.text_color)
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.entity_info.change_permissions.user.1,
                            RichText::new("write")
                                .color(self.style.text_color)
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.entity_info.change_permissions.user.2,
                            RichText::new("read")
                                .color(self.style.text_color)
                                .size(self.style.text_size),
                        );
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Group: ")
                                .color(self.style.text_color)
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.checkbox(
                            &mut self.entity_info.change_permissions.group.0,
                            RichText::new("execute")
                                .color(self.style.text_color)
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.entity_info.change_permissions.group.1,
                            RichText::new("write")
                                .color(self.style.text_color)
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.entity_info.change_permissions.group.2,
                            RichText::new("read")
                                .color(self.style.text_color)
                                .size(self.style.text_size),
                        );
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Other: ")
                                .color(self.style.text_color)
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.checkbox(
                            &mut self.entity_info.change_permissions.other.0,
                            RichText::new("execute")
                                .color(self.style.text_color)
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.entity_info.change_permissions.other.1,
                            RichText::new("write")
                                .color(self.style.text_color)
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.entity_info.change_permissions.other.2,
                            RichText::new("read")
                                .color(self.style.text_color)
                                .size(self.style.text_size),
                        );
                    },
                );

                ui.separator();

                Sides::new().show(
                    ui,
                    |_ui| {},
                    |ui| {
                        if ui
                            .button(
                                RichText::new("Save")
                                    .color(self.style.text_color)
                                    .size(self.style.text_size),
                            )
                            .clicked()
                        {
                            if self.entity_info.change_permissions.user.0 {
                                self.entity_info.permissions.set(
                                    EntityPermissionsCategory::User,
                                    EntityPermissionsKind::Execute,
                                );
                            } else {
                                self.entity_info.permissions.unset(
                                    EntityPermissionsCategory::User,
                                    EntityPermissionsKind::Execute,
                                );
                            }

                            if self.entity_info.change_permissions.user.1 {
                                self.entity_info.permissions.set(
                                    EntityPermissionsCategory::User,
                                    EntityPermissionsKind::Write,
                                );
                            } else {
                                self.entity_info.permissions.unset(
                                    EntityPermissionsCategory::User,
                                    EntityPermissionsKind::Write,
                                );
                            }

                            if self.entity_info.change_permissions.user.2 {
                                self.entity_info.permissions.set(
                                    EntityPermissionsCategory::User,
                                    EntityPermissionsKind::Read,
                                );
                            } else {
                                self.entity_info.permissions.unset(
                                    EntityPermissionsCategory::User,
                                    EntityPermissionsKind::Read,
                                );
                            }

                            if self.entity_info.change_permissions.group.0 {
                                self.entity_info.permissions.set(
                                    EntityPermissionsCategory::Group,
                                    EntityPermissionsKind::Execute,
                                );
                            } else {
                                self.entity_info.permissions.unset(
                                    EntityPermissionsCategory::Group,
                                    EntityPermissionsKind::Execute,
                                );
                            }

                            if self.entity_info.change_permissions.group.1 {
                                self.entity_info.permissions.set(
                                    EntityPermissionsCategory::Group,
                                    EntityPermissionsKind::Write,
                                );
                            } else {
                                self.entity_info.permissions.unset(
                                    EntityPermissionsCategory::Group,
                                    EntityPermissionsKind::Write,
                                );
                            }

                            if self.entity_info.change_permissions.group.2 {
                                self.entity_info.permissions.set(
                                    EntityPermissionsCategory::Group,
                                    EntityPermissionsKind::Read,
                                );
                            } else {
                                self.entity_info.permissions.unset(
                                    EntityPermissionsCategory::Group,
                                    EntityPermissionsKind::Read,
                                );
                            }

                            if self.entity_info.change_permissions.other.0 {
                                self.entity_info.permissions.set(
                                    EntityPermissionsCategory::Other,
                                    EntityPermissionsKind::Execute,
                                );
                            } else {
                                self.entity_info.permissions.unset(
                                    EntityPermissionsCategory::Other,
                                    EntityPermissionsKind::Execute,
                                );
                            }

                            if self.entity_info.change_permissions.other.1 {
                                self.entity_info.permissions.set(
                                    EntityPermissionsCategory::Other,
                                    EntityPermissionsKind::Write,
                                );
                            } else {
                                self.entity_info.permissions.unset(
                                    EntityPermissionsCategory::Other,
                                    EntityPermissionsKind::Write,
                                );
                            }

                            if self.entity_info.change_permissions.other.2 {
                                self.entity_info.permissions.set(
                                    EntityPermissionsCategory::Other,
                                    EntityPermissionsKind::Read,
                                );
                            } else {
                                self.entity_info.permissions.unset(
                                    EntityPermissionsCategory::Other,
                                    EntityPermissionsKind::Read,
                                );
                            }

                            self.entity_info.change_permissions.show = false;
                        }
                    },
                );
            });
        });

        if modal.backdrop_response.clicked() {
            self.entity_info.change_permissions.show = false;
        }
    }

    fn entity_change_owners(&mut self, ctx: &Context) {
        let modal =
            Modal::new(Id::new(format!(
                "Entity Change owners: {:?}",
                self.entity_info.owners
            )))
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    Sides::new().show(
                        ui,
                        |ui| {
                            ui.label(
                                RichText::new("User: ")
                                    .color(self.style.text_color)
                                    .size(self.style.text_size),
                            );
                        },
                        |ui| {
                            ui.label(
                                RichText::new(format!("{}", self.entity_info.change_owners.uid))
                                    .color(self.style.text_color)
                                    .size(self.style.text_size),
                            );

                            ui.text_edit_singleline(&mut self.entity_info.change_owners.username);
                        },
                    );

                    Sides::new().show(
                        ui,
                        |ui| {
                            ui.label(
                                RichText::new("Group: ")
                                    .color(self.style.text_color)
                                    .size(self.style.text_size),
                            );
                        },
                        |ui| {
                            ui.label(
                                RichText::new(format!("{}", self.entity_info.change_owners.gid))
                                    .color(self.style.text_color)
                                    .size(self.style.text_size),
                            );

                            ui.text_edit_singleline(&mut self.entity_info.change_owners.groupname);
                        },
                    );

                    ui.separator();

                    Sides::new().show(
                        ui,
                        |_ui| {},
                        |ui| {
                            if ui
                                .button(
                                    RichText::new("Save")
                                        .color(self.style.text_color)
                                        .size(self.style.text_size),
                                )
                                .clicked()
                            {
                                if self
                                    .entity_info
                                    .owners
                                    .username()
                                    .ne(&self.entity_info.change_owners.username)
                                {
                                    if let Err(_) = self.entity_info.owners.set_username(
                                        self.entity_info.change_owners.username.clone(),
                                    ) {
                                        //TODO handle error
                                        self.entity_info
                                            .change_owners
                                            .username
                                            .clone_from(&self.entity_info.owners.username());
                                    }
                                }

                                if self
                                    .entity_info
                                    .owners
                                    .groupname()
                                    .ne(&self.entity_info.change_owners.groupname)
                                {
                                    if let Err(_) = self.entity_info.owners.set_groupname(
                                        self.entity_info.change_owners.groupname.clone(),
                                    ) {
                                        //TODO handle error
                                        self.entity_info
                                            .change_owners
                                            .groupname
                                            .clone_from(&self.entity_info.owners.groupname());
                                    }
                                }

                                self.entity_info.change_owners.show = false;
                            }
                        },
                    );
                });
            });

        if modal.backdrop_response.clicked() {
            self.entity_info.change_owners.show = false;
        }
    }

    fn toogle_selection_entity(
        &mut self,
        ui: &Ui,
        response: &Response,
        idx: usize,
        entity: &Entity,
        entitys: &Vec<Entity>,
    ) {
        if response.clicked() {
            if ui.input(|i| i.modifiers.ctrl) {
                if self.entitys_selection.entitys.contains(entity) {
                    self.entitys_selection.entitys.remove(&entity);
                } else {
                    self.entitys_selection.entitys.insert(entity.clone());
                }
            } else if ui.input(|i| i.modifiers.shift) {
                let (mut a, mut b) = (0, 0);
                if self.entitys_selection.last_idx < idx {
                    (a, b) = (self.entitys_selection.last_idx, idx);
                } else {
                    (a, b) = (idx, self.entitys_selection.last_idx);
                }
                for entity in entitys.iter().skip(a).take(b - a + 1) {
                    self.entitys_selection.entitys.insert(entity.clone());
                }
            } else {
                self.entitys_selection.entitys.clear();
                self.entitys_selection.entitys.insert(entity.clone());
            }

            self.entitys_selection.last_idx = idx;
        }
    }

    fn selection_entity_move(&mut self, ctx: &Context, pathbuf: PathBuf) {
        let events = ctx.input(|i| i.events.clone());

        if events.iter().any(|e| {
            matches!(
                e,
                Event::Key {
                    modifiers: Modifiers { ctrl: true, .. },
                    key: Key::C,
                    ..
                }
            )
        }) {
            self.entitys_selection.move_kind = EntitysMoveKind::Copy;
            println!("\nCopy: {:?}\n", self.entitys_selection.entitys_vec())
        }

        if events.iter().any(|e| {
            matches!(
                e,
                Event::Key {
                    modifiers: Modifiers { ctrl: true, .. },
                    key: Key::X,
                    ..
                }
            )
        }) {
            self.entitys_selection.move_kind = EntitysMoveKind::Cut;
            println!("\nCut: {:?}\n", self.entitys_selection.entitys_vec())
        }

        if events.iter().any(|e| {
            matches!(
                e,
                Event::Key {
                    modifiers: Modifiers { ctrl: true, .. },
                    key: Key::V,
                    ..
                }
            )
        }) {
            match self.entitys_selection.move_kind {
                EntitysMoveKind::Copy => {
                    let _ = self
                        .controller
                        .send(Command::CopyEntitys(
                            0,
                            Tab::default(),
                            0,
                            self.entitys_selection.entitys_vec(),
                            pathbuf.clone(),
                        ))
                        .inspect_err(JujikError::handle_err);

                    println!("\nCopy to {:?}\n", pathbuf);
                }
                EntitysMoveKind::Cut => {
                    let _ = self
                        .controller
                        .send(Command::MoveEntitys(
                            0,
                            Tab::default(),
                            0,
                            self.entitys_selection.entitys_vec(),
                            pathbuf.clone(),
                        ))
                        .inspect_err(JujikError::handle_err);

                    println!("\nCut to {:?}\n", pathbuf);
                }
                _ => {}
            }
        }
    }
}

impl EntitysSelection {
    fn entitys_vec(&self) -> Vec<Entity> {
        self.entitys.clone().into_iter().collect()
    }
}

impl Default for ShowEntitysColumn {
    fn default() -> Self {
        Self {
            filekind: true,
            name: true,
            name_with_extension: true,
            extension: true,
            permissions: true,
            owners: true,
            size: true,
            modification_date: true,
            creation_data: true,
        }
    }
}

impl Default for JujikStyle {
    fn default() -> Self {
        Self {
            primary_color: Color32::from_rgb(100, 100, 100),
            background_color: Color32::from_rgb(50, 50, 50),
            selection_color: Color32::from_rgb(50, 80, 50),
            text_color: Color32::LIGHT_GRAY,
            text_size: 18.0,
        }
    }
}
