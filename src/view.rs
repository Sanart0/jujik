use crate::config::Config;
use crate::entity::Entity;
use crate::entity::date::EntityDate;
use crate::entity::find::{EntitysFinder, FindParameters};
use crate::entity::kind::EntityKind;
use crate::entity::owner::EntityOwners;
use crate::entity::permission::{
    EntityPermissions, EntityPermissionsCategory, EntityPermissionsKind,
};
use crate::entity::size::EntitySize;
use crate::tab::{SortBy, SortDirection, SortField, TabContent};
use crate::{commands::Command, error::JujikError, pin::Pin, tab::Tab};
use chrono::NaiveDate;
use eframe::{App, EventLoopBuilderHook, NativeOptions, run_native};
use egui::{
    Align, Button, CentralPanel, Color32, ComboBox, Context, DragValue, Event, Id, Key, Label,
    Layout, Modal, Modifiers, Response, RichText, ScrollArea, Sense, SidePanel, Sides, Stroke,
    TextEdit, TextStyle, TopBottomPanel, Ui, Visuals, menu,
};
use egui_extras::{Column, DatePickerButton, TableBuilder};
use serde::{Deserialize, Serialize};
use std::f32;
use std::time::{Duration, Instant};
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
struct EntityCreate {
    show: bool,
    idx_tab: usize,
    tab: Tab,
    entity: Entity,
    path: String,
    name: String,
    extension: String,
    permissions: EntityPermissions,
    change_permissions: ChangeEntityPermissions,
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
    size: EntitySize,
    modification: EntityDate,
    creation: EntityDate,
    change_permissions: ChangeEntityPermissions,
    change_owners: ChangeEntityOwners,
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
struct ChangeEntityPermissions {
    show: bool,
    user: (bool, bool, bool),
    group: (bool, bool, bool),
    other: (bool, bool, bool),
}

#[derive(Default)]
struct ChangeEntityOwners {
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
    Copy(Vec<Entity>),
    Cut(Vec<Entity>),
}

#[derive(Default)]
struct EntitysSelection {
    move_kind: EntitysMoveKind,
    entitys: HashSet<Entity>,
    last_idx: usize,
}

#[derive(Default)]
struct EntityEdit {
    changed: bool,
    content: String,
}

#[derive(Default)]
struct FinderInfo {
    show: bool,
    idx_tab: usize,
    tab: Tab,
    parameters: FindParameters,
    regex: String,
    path: String,
    name: String,
    extension: String,
    kind: EntityKind,
    permissions: EntityPermissions,
    owners: EntityOwners,
    change_permissions: ChangeEntityPermissions,
    change_owners: ChangeEntityOwners,
    size: (EntitySize, EntitySize),
    date_modification: (EntityDate, EntityDate),
    date_creation: (EntityDate, EntityDate),
    change_size: (String, String),
    change_date_modification: (NaiveDate, NaiveDate),
    change_date_creation: (NaiveDate, NaiveDate),
}

#[derive(Default)]
struct EntitysSortByInfo {
    show: bool,
    sortby: SortBy,
    field: SortField,
    direction: SortDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitysShowColumn {
    filekind: bool,
    name: bool,
    name_with_extension: bool,
    extension: bool,
    permissions: bool,
    owners: bool,
    size: bool,
    date_modification: bool,
    date_creation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JujikColor {
    color: [u8; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JujikStyle {
    show: bool,
    primary_color: JujikColor,
    background_color: JujikColor,
    selection_color: JujikColor,
    text_color: JujikColor,
    text_size: f32,
}

pub struct JujikView {
    controller: Sender<Command>,
    view: Receiver<Command>,
    first_update: bool,
    update: Instant,
    style: JujikStyle,
    pins: Vec<Pin>,
    tabs: Vec<Tab>,
    entitys_show: EntitysShowColumn,
    entitys_sortby_info: EntitysSortByInfo,
    current_tab_idx: usize,
    entitys_selection: EntitysSelection,
    pin_info: PinInfo,
    tab_info: TabInfo,
    entity_create: EntityCreate,
    entity_info: EntityInfo,
    entitys_delete: EntitysDelete,
    entity_edit: EntityEdit,
    finder_info: FinderInfo,
}

impl App for JujikView {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.first_update {
            let _ = self
                .controller
                .send(Command::Update)
                .inspect_err(JujikError::handle_err);

            self.first_update = false;
        }

        self.style(ctx);

        TopBottomPanel::top("menu").show(ctx, |ui| {
            self.main_bar(ctx, ui);
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
        let _ = self
            .controller
            .send(Command::SetConfig(Config::new(
                self.style.clone(),
                self.pins.clone(),
                self.tabs.clone(),
                self.current_tab_idx,
                self.entitys_show.clone(),
            )))
            .inspect_err(JujikError::handle_err);

        let _ = self
            .controller
            .send(Command::Drop)
            .inspect_err(JujikError::handle_err);
    }
}

impl JujikView {
    pub fn new(controller: Sender<Command>, view: Receiver<Command>) -> Self {
        Self {
            controller,
            view,
            first_update: true,
            update: Instant::now(),
            style: JujikStyle::default(),
            pins: Vec::new(),
            tabs: Vec::new(),
            entitys_show: EntitysShowColumn::default(),
            entitys_sortby_info: EntitysSortByInfo::default(),
            current_tab_idx: 0,
            entitys_selection: EntitysSelection::default(),
            pin_info: PinInfo::default(),
            tab_info: TabInfo::default(),
            entity_create: EntityCreate::default(),
            entity_info: EntityInfo::default(),
            entitys_delete: EntitysDelete::default(),
            entity_edit: EntityEdit::default(),
            finder_info: FinderInfo::default(),
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

    fn handle_commad(&mut self, ctx: &egui::Context) -> Result<(), JujikError> {
        if let Ok(command) = self.view.try_recv() {
            #[cfg(feature = "print_command")]
            println!("View: {:?}", command);

            match command {
                // Config
                Command::GetConfig => {
                    let _ = self
                        .controller
                        .send(Command::SetConfig(Config::new(
                            self.style.clone(),
                            self.pins.clone(),
                            self.tabs.clone(),
                            self.current_tab_idx,
                            self.entitys_show.clone(),
                        )))
                        .inspect_err(JujikError::handle_err);
                }
                Command::SetConfig(config) => {
                    self.style.clone_from(&config.style);
                    self.pins.clone_from(&config.pins);
                    self.tabs.clone_from(&config.tabs);
                    self.current_tab_idx = config.current_tab_idx;
                    self.entitys_show.clone_from(&config.entitys_show);
                }

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

        if self.update.elapsed() >= Duration::from_secs(5) {
            self.controller
                .send(Command::UpdateTab(self.current_tab_idx))?;

            self.update = Instant::now();
        }

        Ok(())
    }

    fn style(&self, ctx: &egui::Context) {
        let mut visuals = Visuals::dark();

        visuals.widgets.noninteractive.bg_fill = self.style.background_color.into_color32();
        visuals.widgets.inactive.bg_fill =
            self.style.primary_color.into_color32().linear_multiply(0.7);
        visuals.widgets.hovered.bg_fill =
            self.style.primary_color.into_color32().linear_multiply(0.8);
        visuals.widgets.active.bg_fill = self.style.primary_color.into_color32();

        visuals.widgets.noninteractive.fg_stroke =
            Stroke::new(1.0, self.style.text_color.into_color32());
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.style.text_color.into_color32());
        visuals.extreme_bg_color = self.style.background_color.into_color32();
        visuals.selection.bg_fill = self.style.selection_color.into_color32();

        ctx.set_visuals(visuals);
    }

    fn main_bar(&mut self, ctx: &Context, ui: &mut Ui) {
        menu::bar(ui, |ui| {
            ui.menu_button(
                RichText::new("File")
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
                |ui| {
                    if ui
                        .button(
                            RichText::new("Reset Config")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        )
                        .clicked()
                    {
                        let _ = self
                            .controller
                            .send(Command::SetConfig(Config::default()))
                            .inspect_err(JujikError::handle_err);
                    }

                    if ui
                        .button(
                            RichText::new("Exit")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        )
                        .clicked()
                    {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                },
            );

            ui.menu_button(
                RichText::new("Show")
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
                |ui| {
                    ui.menu_button(
                        RichText::new("Entity")
                            .color(self.style.text_color.into_color32())
                            .size(self.style.text_size),
                        |ui| {
                            ui.checkbox(
                                &mut self.entitys_show.filekind,
                                RichText::new("File kind")
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            );
                            ui.checkbox(
                                &mut self.entitys_show.name,
                                RichText::new("Name")
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            );
                            ui.checkbox(
                                &mut self.entitys_show.name_with_extension,
                                RichText::new("Name with extension")
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            );
                            ui.checkbox(
                                &mut self.entitys_show.extension,
                                RichText::new("Extension")
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            );
                            ui.checkbox(
                                &mut self.entitys_show.permissions,
                                RichText::new("Permissions")
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            );
                            ui.checkbox(
                                &mut self.entitys_show.owners,
                                RichText::new("Owners")
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            );
                            ui.checkbox(
                                &mut self.entitys_show.size,
                                RichText::new("Size")
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            );
                            ui.checkbox(
                                &mut self.entitys_show.date_modification,
                                RichText::new("Date modification")
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            );
                            ui.checkbox(
                                &mut self.entitys_show.date_creation,
                                RichText::new("Date creation")
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            );
                        },
                    );

                    let style = ui.button(
                        RichText::new("Style")
                            .color(self.style.text_color.into_color32())
                            .size(self.style.text_size),
                    );

                    if style.clicked() {
                        self.style.show = true;
                    }
                },
            );

            // if ui
            //     .button(
            //         RichText::new("Find")
            //             .color(self.style.text_color.into_color32())
            //             .size(self.style.text_size),
            //     )
            //     .clicked()
            // {
            //     self.finder_info.show = true;
            // }
        });

        if self.finder_info.show {
            self.entitys_selection.entitys.clear();

            self.finder_info(ctx, true);
        }

        if self.style.show {
            self.style_info(ctx);
        }
    }

    fn message(&self, ctx: &Context) {
        let modal = Modal::new(Id::new("Message")).show(ctx, |ui| {});
    }
}

// Pin
impl JujikView {
    fn pin(&mut self, ui: &mut Ui, ctx: &Context) {
        ui.label(
            RichText::new("Pin")
                .color(self.style.text_color.into_color32())
                .size(self.style.text_size),
        );

        ui.separator();

        for (idx, pin) in self.pins.clone().iter().enumerate() {
            let response = ui.add(
                Button::new(
                    RichText::new(pin.name())
                        .color(self.style.text_color.into_color32())
                        .size(self.style.text_size),
                )
                .fill(self.style.background_color.into_color32()),
            );

            if response.clicked() {
                let _ = self
                    .controller
                    .send(Command::CreateEntitys(pin.path()))
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
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
            );

            let _position = ui.menu_button(
                RichText::new("Position")
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
                |ui| {
                    let up = ui.button(
                        RichText::new("Up")
                            .color(self.style.text_color.into_color32())
                            .size(self.style.text_size),
                    );

                    let down = ui.button(
                        RichText::new("Down")
                            .color(self.style.text_color.into_color32())
                            .size(self.style.text_size),
                    );

                    if up.clicked() {
                        if idx > 0 {
                            let _ = self
                                .controller
                                .send(Command::ChangePinPosition(idx, idx - 1, pin.clone()))
                                .inspect_err(JujikError::handle_err);
                        }

                        ui.close_menu();
                    }

                    if down.clicked() {
                        ui.close_menu();
                        if idx < self.pins.len() - 1 {
                            let _ = self
                                .controller
                                .send(Command::ChangePinPosition(idx, idx + 1, pin.clone()))
                                .inspect_err(JujikError::handle_err);
                        }

                        ui.close_menu();
                    }
                },
            );

            let delete = ui.button(
                RichText::new("Delete")
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
            );

            let info = ui.button(
                RichText::new("Info")
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
            );

            if open.clicked() {
                let _ = self
                    .controller
                    .send(Command::CreateEntitys(pin.path()))
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
        let modal = Modal::new(Id::new("Pin Info")).show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.label(
                    RichText::new(format!("Pin Info: {}", self.pin_info.name))
                        .color(self.style.text_color.into_color32())
                        .size(self.style.text_size),
                );

                ui.separator();

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Name:")
                                .color(self.style.text_color.into_color32())
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
                            RichText::new("Path:")
                                .color(self.style.text_color.into_color32())
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
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            )
                            .clicked()
                        {
                            let _ = self
                                .controller
                                .send(Command::SetConfig(Config::new(
                                    self.style.clone(),
                                    self.pins.clone(),
                                    self.tabs.clone(),
                                    self.current_tab_idx,
                                    self.entitys_show.clone(),
                                )))
                                .inspect_err(JujikError::handle_err);

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
        });

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
                            .color(self.style.text_color.into_color32())
                            .size(self.style.text_size),
                    );

                    if response.clicked() {
                        self.current_tab_idx = idx;
                        self.entitys_selection.entitys.clear();

                        let _ = self
                            .controller
                            .send(Command::UpdateTab(self.current_tab_idx))
                            .inspect_err(JujikError::handle_err);
                    }

                    self.tab_context_menu(ui, &response, idx, tab);
                }

                if self.entity_create.show {
                    self.entity_create(ctx);
                }

                if self.entity_create.change_permissions.show {
                    self.entity_create_permissions(ctx);
                }

                if self.tab_info.show {
                    self.tab_info(ctx);
                }
            });
        });
    }

    fn tab_content(&mut self, ctx: &Context, ui: &mut Ui) {
        if let Some(tab) = self.tabs.clone().get_mut(self.current_tab_idx) {
            match tab.content() {
                TabContent::Entitys(_, _, _) => {
                    self.entitys_bar(ui, tab);

                    ui.separator();

                    tab.sort();
                    self.entitys(ctx, ui, self.current_tab_idx, tab);
                }
                TabContent::View(entity) => {
                    self.view_text_bar(ui, entity);

                    ui.separator();

                    self.view_text(ui, entity);
                }
                TabContent::Editor(entity) => {
                    if !self.entity_edit.changed {
                        self.entity_edit.content = match entity.content() {
                            Ok(content) => content,
                            Err(_) => String::new(),
                        };

                        self.entity_edit.changed = true;
                    }

                    self.edit_text_bar(ui, entity);

                    ui.separator();

                    self.edit_text(ui);
                }
                TabContent::Find(finder) => {
                    self.finder_bar(ui, tab, finder);

                    ui.separator();

                    self.finder(ctx, ui, self.current_tab_idx, tab, finder);
                }
                _ => {}
            }
        }
    }

    fn entitys(&mut self, ctx: &Context, ui: &mut Ui, idx_tab: usize, tab: &Tab) {
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
                                            .color(self.style.text_color.into_color32())
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
                                            .color(self.style.text_color.into_color32())
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
                                            .color(self.style.text_color.into_color32())
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
                                            .color(self.style.text_color.into_color32())
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
                                            .color(self.style.text_color.into_color32())
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
                                            .color(self.style.text_color.into_color32())
                                            .size(self.style.text_size),
                                    );
                                });
                            });
                        }
                        if self.entitys_show.date_modification {
                            header.col(|ui| {
                                ui.centered_and_justified(|ui| {
                                    ui.label(
                                        RichText::new("Modification Date")
                                            .color(self.style.text_color.into_color32())
                                            .size(self.style.text_size),
                                    );
                                });
                            });
                        }
                        if self.entitys_show.date_creation {
                            header.col(|ui| {
                                ui.centered_and_justified(|ui| {
                                    ui.label(
                                        RichText::new("Creation Data")
                                            .color(self.style.text_color.into_color32())
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
                                                    .color(self.style.text_color.into_color32())
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
                                                    .color(self.style.text_color.into_color32())
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
                                                .color(self.style.text_color.into_color32())
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
                                                    .color(self.style.text_color.into_color32())
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
                                                    .color(self.style.text_color.into_color32())
                                                    .size(self.style.text_size),
                                            );
                                        });
                                    });
                                }
                                if self.entitys_show.size {
                                    row.col(|ui| {
                                        ui.centered_and_justified(|ui| {
                                            ui.label(
                                                RichText::new(format!("{}", entity.size()))
                                                    .color(self.style.text_color.into_color32())
                                                    .size(self.style.text_size),
                                            );
                                        });
                                    });
                                }
                                if self.entitys_show.date_modification {
                                    row.col(|ui| {
                                        ui.centered_and_justified(|ui| {
                                            ui.label(
                                                RichText::new(format!(
                                                    "{}",
                                                    entity.modification().date_str()
                                                ))
                                                .color(self.style.text_color.into_color32())
                                                .size(self.style.text_size),
                                            );
                                        });
                                    });
                                }
                                if self.entitys_show.date_creation {
                                    row.col(|ui| {
                                        ui.centered_and_justified(|ui| {
                                            ui.label(
                                                RichText::new(format!(
                                                    "{}",
                                                    entity.creation().date_str()
                                                ))
                                                .color(self.style.text_color.into_color32())
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

                        if self.entitys_sortby_info.show {
                            self.entitys_sortby_info(ctx, idx_tab, tab);
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
            // let create_entity = ui.button(
            //     RichText::new("Create Entity")
            //         .color(self.style.text_color.into_color32())
            //         .size(self.style.text_size),
            // );

            let create_pin = ui.button(
                RichText::new("Create Pin")
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
            );

            let paste = ui.button(
                RichText::new("Paste")
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
            );

            let _position = ui.menu_button(
                RichText::new("Position")
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
                |ui| {
                    let left = ui.button(
                        RichText::new("Left")
                            .color(self.style.text_color.into_color32())
                            .size(self.style.text_size),
                    );

                    let right = ui.button(
                        RichText::new("Rigth")
                            .color(self.style.text_color.into_color32())
                            .size(self.style.text_size),
                    );

                    if left.clicked() {
                        if idx > 0 {
                            let _ = self
                                .controller
                                .send(Command::ChangeTabPosition(idx, idx - 1, tab.clone()))
                                .inspect_err(JujikError::handle_err);

                            let idx_diff = self.current_tab_idx as isize - idx as isize;

                            match idx_diff {
                                0 => self.current_tab_idx -= 1,
                                -1 => self.current_tab_idx += 1,
                                _ => {}
                            }
                        }

                        ui.close_menu();
                    }

                    if right.clicked() {
                        if idx < self.tabs.len() - 1 {
                            let _ = self
                                .controller
                                .send(Command::ChangeTabPosition(idx, idx + 1, tab.clone()))
                                .inspect_err(JujikError::handle_err);

                            let idx_diff = self.current_tab_idx as isize - idx as isize;

                            match idx_diff {
                                0 => self.current_tab_idx += 1,
                                1 => self.current_tab_idx -= 1,
                                _ => {}
                            }
                        }

                        ui.close_menu();
                    }
                },
            );

            let sortby = ui.button(
                RichText::new("Sort")
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
            );

            let delete = ui.button(
                RichText::new("Delete")
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
            );

            let info = ui.button(
                RichText::new("Info")
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
            );

            // if create_entity.clicked() {
            //     self.entity_create.idx_tab = idx;
            //     self.entity_create.tab = tab.clone();
            //     self.entity_create.entity.set_path(tab.path());
            //     self.entity_create.path = tab.path_str();
            //     self.entity_create.show = true;
            //
            //     ui.close_menu();
            // }

            if create_pin.clicked() {
                let _ = self
                    .controller
                    .send(Command::CreatePin(tab.path()))
                    .inspect_err(JujikError::handle_err);

                ui.close_menu();
            }

            if paste.clicked() {
                self.entitys_selection
                    .paste(self.controller.clone(), tab.path());

                ui.close_menu();
            }

            if sortby.clicked() {
                self.entitys_sortby_info.sortby.clone_from(&tab.sortby());
                self.entitys_sortby_info
                    .field
                    .clone_from(&self.entitys_sortby_info.sortby.field);
                self.entitys_sortby_info
                    .direction
                    .clone_from(&self.entitys_sortby_info.sortby.direction);

                self.entitys_sortby_info.show = true;
            }

            if delete.clicked() {
                let _ = self
                    .controller
                    .send(Command::DeleteTab(idx, tab.clone()))
                    .inspect_err(JujikError::handle_err);

                if self.current_tab_idx != 0 && self.current_tab_idx >= idx && self.tabs.len() > 1 {
                    self.current_tab_idx -= 1;
                }

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
        let modal = Modal::new(Id::new("Tab Info")).show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.label(
                    RichText::new(format!("Tab Info: {}", self.tab_info.name))
                        .color(self.style.text_color.into_color32())
                        .size(self.style.text_size),
                );

                ui.separator();

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Name:")
                                .color(self.style.text_color.into_color32())
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
                            RichText::new("Path:")
                                .color(self.style.text_color.into_color32())
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
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            )
                            .clicked()
                        {
                            let _ = self
                                .controller
                                .send(Command::SetConfig(Config::new(
                                    self.style.clone(),
                                    self.pins.clone(),
                                    self.tabs.clone(),
                                    self.current_tab_idx,
                                    self.entitys_show.clone(),
                                )))
                                .inspect_err(JujikError::handle_err);

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
        });

        if modal.backdrop_response.clicked() {
            self.tab_info.show = false;
        }
    }

    fn entitys_bar(&self, ui: &mut Ui, tab: &Tab) {
        ui.horizontal(|ui| {
            let back = ui.add(Button::new(
                RichText::new("Back")
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
            ));

            if back.clicked() {
                let _ = self
                    .controller
                    .send(Command::ChangeTabDirectory(
                        self.current_tab_idx,
                        tab.clone(),
                        None,
                    ))
                    .inspect_err(JujikError::handle_err);
            }

            let path = ui.add(
                Label::new(
                    RichText::new(format!("{}", tab.path_str()))
                        .color(self.style.text_color.into_color32())
                        .size(self.style.text_size),
                )
                .selectable(false)
                .sense(Sense::click()),
            );

            if path.clicked() {}
        });
    }

    fn view_text_bar(&self, ui: &mut Ui, entity: &Entity) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{}", entity.path_str()))
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
            );
        });
    }

    fn view_text(&self, ui: &mut Ui, entity: &Entity) {
        match entity.content() {
            Ok(content) => {
                ScrollArea::both().show(ui, |ui| {
                    ui.label(
                        RichText::new(format!("{}", content))
                            .color(self.style.text_color.into_color32())
                            .size(self.style.text_size),
                    );
                });
            }
            Err(err) => {
                //TODO Handle error
            }
        }
    }

    fn edit_text_bar(&mut self, ui: &mut Ui, entity: &Entity) {
        ui.horizontal(|ui| {
            let save = ui.button(
                RichText::new("Save")
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
            );

            ui.label(
                RichText::new(format!("{}", entity.path_str()))
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
            );

            if save.clicked() {
                let _ = self
                    .controller
                    .send(Command::SetConfig(Config::new(
                        self.style.clone(),
                        self.pins.clone(),
                        self.tabs.clone(),
                        self.current_tab_idx,
                        self.entitys_show.clone(),
                    )))
                    .inspect_err(JujikError::handle_err);

                let _ = self
                    .controller
                    .send(Command::ChangeEntityContent(
                        0,
                        Tab::default(),
                        entity.clone(),
                        self.entity_edit.content.clone(),
                    ))
                    .inspect_err(JujikError::handle_err);

                self.entity_edit.changed = false;
            }
        });
    }

    fn edit_text(&mut self, ui: &mut Ui) {
        ScrollArea::both().show(ui, |ui| {
            ui.add(
                TextEdit::multiline(&mut self.entity_edit.content)
                    .font(TextStyle::Heading)
                    .desired_width(f32::INFINITY),
            );
        });
    }

    fn finder_bar(&mut self, ui: &mut Ui, tab: &Tab, finder: &EntitysFinder) {
        ui.horizontal(|ui| {
            let change = ui.button(
                RichText::new("Parameters")
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
            );

            ui.label(
                RichText::new(format!("{}", tab.path_str()))
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
            );

            if change.clicked() {
                self.finder_info.show = true;
                self.finder_info.idx_tab = self.current_tab_idx;
                self.finder_info.tab = tab.clone();
            }
        });
    }

    fn finder(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        idx_tab: usize,
        tab: &Tab,
        finder: &EntitysFinder,
    ) {
        let mut scroll = ui.ctx().style().spacing.scroll;
        scroll.floating = false;
        scroll.bar_width = 4.0;
        scroll.bar_inner_margin = 4.0;
        scroll.bar_outer_margin = 4.0;
        scroll.foreground_color = false;
        ui.ctx().all_styles_mut(|s| s.spacing.scroll = scroll);

        let mut responses: Vec<Option<Response>> = Vec::new();

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
                                        .color(self.style.text_color.into_color32())
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
                                        .color(self.style.text_color.into_color32())
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
                                        .color(self.style.text_color.into_color32())
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
                                        .color(self.style.text_color.into_color32())
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
                                        .color(self.style.text_color.into_color32())
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
                                        .color(self.style.text_color.into_color32())
                                        .size(self.style.text_size),
                                );
                            });
                        });
                    }
                    if self.entitys_show.date_modification {
                        header.col(|ui| {
                            ui.centered_and_justified(|ui| {
                                ui.label(
                                    RichText::new("Modification Date")
                                        .color(self.style.text_color.into_color32())
                                        .size(self.style.text_size),
                                );
                            });
                        });
                    }
                    if self.entitys_show.date_creation {
                        header.col(|ui| {
                            ui.centered_and_justified(|ui| {
                                ui.label(
                                    RichText::new("Creation Data")
                                        .color(self.style.text_color.into_color32())
                                        .size(self.style.text_size),
                                );
                            });
                        });
                    }
                })
                .body(|mut body| {
                    for (idx_entity, entity) in finder.entitys().iter().enumerate() {
                        body.row(40.0, |mut row| {
                            row.set_selected(self.entitys_selection.entitys.contains(&entity));

                            row.col(|ui| {});
                            if self.entitys_show.filekind {
                                row.col(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        ui.label(
                                            RichText::new(format!("{}", entity.kind()))
                                                .color(self.style.text_color.into_color32())
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
                                                .color(self.style.text_color.into_color32())
                                                .size(self.style.text_size),
                                        );
                                    });
                                });
                            }
                            if self.entitys_show.extension {
                                row.col(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        ui.label(
                                            RichText::new(format!("{}", entity.extension_str()))
                                                .color(self.style.text_color.into_color32())
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
                                                .color(self.style.text_color.into_color32())
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
                                                .color(self.style.text_color.into_color32())
                                                .size(self.style.text_size),
                                        );
                                    });
                                });
                            }
                            if self.entitys_show.size {
                                row.col(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        ui.label(
                                            RichText::new(format!("{}", entity.size()))
                                                .color(self.style.text_color.into_color32())
                                                .size(self.style.text_size),
                                        );
                                    });
                                });
                            }
                            if self.entitys_show.date_modification {
                                row.col(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        ui.label(
                                            RichText::new(format!(
                                                "{}",
                                                entity.modification().date_str()
                                            ))
                                            .color(self.style.text_color.into_color32())
                                            .size(self.style.text_size),
                                        );
                                    });
                                });
                            }
                            if self.entitys_show.date_creation {
                                row.col(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        ui.label(
                                            RichText::new(format!(
                                                "{}",
                                                entity.creation().date_str()
                                            ))
                                            .color(self.style.text_color.into_color32())
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

        for (idx, entity) in finder.entitys().iter().enumerate() {
            if let Some(response) = responses.get(idx) {
                if let Some(response) = response {
                    self.toogle_selection_entity(ui, &response, idx, entity, &finder.entitys());
                }
            }
        }

        self.selection_entity_move(ctx, tab.path());
    }

    fn finder_info(&mut self, ctx: &Context, new_tab: bool) {
        let modal = Modal::new(Id::new("Finder Info")).show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.label(
                    RichText::new("Finder Info")
                        .color(self.style.text_color.into_color32())
                        .size(self.style.text_size),
                );

                ui.separator();

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Regex:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.text_edit_singleline(&mut self.finder_info.regex);
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Path:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.text_edit_singleline(&mut self.finder_info.path);
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Name:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.text_edit_singleline(&mut self.finder_info.name);
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Extension:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.text_edit_singleline(&mut self.finder_info.extension);
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Kind:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ComboBox::from_id_salt("Entity Kind").show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.finder_info.kind,
                                EntityKind::File,
                                "File",
                            );
                            ui.selectable_value(
                                &mut self.finder_info.kind,
                                EntityKind::Directory,
                                "Directory",
                            );
                            ui.selectable_value(
                                &mut self.finder_info.kind,
                                EntityKind::Symlink,
                                "Symlink",
                            );
                            ui.selectable_value(
                                &mut self.finder_info.kind,
                                EntityKind::Block,
                                "Block",
                            );
                            ui.selectable_value(
                                &mut self.finder_info.kind,
                                EntityKind::Character,
                                "Character",
                            );
                            ui.selectable_value(
                                &mut self.finder_info.kind,
                                EntityKind::Pipe,
                                "Pipe",
                            );
                            ui.selectable_value(
                                &mut self.finder_info.kind,
                                EntityKind::Socket,
                                "Socket",
                            );
                            ui.selectable_value(
                                &mut self.finder_info.kind,
                                EntityKind::Unknown,
                                "Unknown",
                            );
                        });
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Permissions:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        let permissions = ui.add(
                            Label::new(
                                RichText::new(format!("{}", self.finder_info.permissions))
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            )
                            .selectable(true),
                        );

                        permissions.context_menu(|ui| {
                            let change = ui.button(
                                RichText::new("Change")
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            );

                            if change.clicked() {
                                self.finder_info.change_permissions.show = true;

                                self.finder_info.change_permissions.user = (
                                    self.finder_info.permissions.has(
                                        EntityPermissionsCategory::User,
                                        EntityPermissionsKind::Execute,
                                    ),
                                    self.finder_info.permissions.has(
                                        EntityPermissionsCategory::User,
                                        EntityPermissionsKind::Write,
                                    ),
                                    self.finder_info.permissions.has(
                                        EntityPermissionsCategory::User,
                                        EntityPermissionsKind::Read,
                                    ),
                                );
                                self.finder_info.change_permissions.group = (
                                    self.finder_info.permissions.has(
                                        EntityPermissionsCategory::Group,
                                        EntityPermissionsKind::Execute,
                                    ),
                                    self.finder_info.permissions.has(
                                        EntityPermissionsCategory::Group,
                                        EntityPermissionsKind::Write,
                                    ),
                                    self.finder_info.permissions.has(
                                        EntityPermissionsCategory::Group,
                                        EntityPermissionsKind::Read,
                                    ),
                                );
                                self.finder_info.change_permissions.other = (
                                    self.finder_info.permissions.has(
                                        EntityPermissionsCategory::Other,
                                        EntityPermissionsKind::Execute,
                                    ),
                                    self.finder_info.permissions.has(
                                        EntityPermissionsCategory::Other,
                                        EntityPermissionsKind::Write,
                                    ),
                                    self.finder_info.permissions.has(
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
                            RichText::new("Owners:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        let owners = ui.add(
                            Label::new(
                                RichText::new(format!("{}", self.finder_info.owners))
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            )
                            .selectable(true),
                        );

                        owners.context_menu(|ui| {
                            let change = ui.button(
                                RichText::new("Change")
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            );

                            if change.clicked() {
                                self.finder_info.change_owners.show = true;

                                self.finder_info.change_owners.uid = self.finder_info.owners.uid();
                                self.finder_info.change_owners.gid = self.finder_info.owners.gid();
                                self.finder_info.change_owners.username =
                                    self.finder_info.owners.username();
                                self.finder_info.change_owners.groupname =
                                    self.finder_info.owners.groupname();
                            }
                        });
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Size from:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.add(TextEdit::singleline(&mut self.finder_info.change_size.0));
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Size to:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.add(TextEdit::singleline(&mut self.finder_info.change_size.1));
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Date modification from:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.add(
                            DatePickerButton::new(&mut self.finder_info.change_date_modification.0)
                                .id_salt(
                                    format!(
                                        "from {}",
                                        self.finder_info.date_modification.0.date_str()
                                    )
                                    .as_str(),
                                ),
                        );
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Date modification to:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.add(
                            DatePickerButton::new(&mut self.finder_info.change_date_modification.1)
                                .id_salt(
                                    format!(
                                        "to {}",
                                        self.finder_info.date_modification.1.date_str()
                                    )
                                    .as_str(),
                                ),
                        );
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Date creation from:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.add(
                            DatePickerButton::new(&mut self.finder_info.change_date_creation.0)
                                .id_salt(
                                    format!("from {}", self.finder_info.date_creation.0.date_str())
                                        .as_str(),
                                ),
                        );
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Date creation to:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.add(
                            DatePickerButton::new(&mut self.finder_info.change_date_creation.1)
                                .id_salt(
                                    format!("from {}", self.finder_info.date_creation.1.date_str())
                                        .as_str(),
                                ),
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
                                RichText::new("Find")
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            )
                            .clicked()
                        {
                            //TODO DO PARAMETERS CHANGE
                            if new_tab {
                                let _ = self
                                    .controller
                                    .send(Command::CreateFinder(
                                        self.finder_info.parameters.clone(),
                                    ))
                                    .inspect_err(JujikError::handle_err);
                            } else {
                                let _ = self
                                    .controller
                                    .send(Command::UpdateFind(
                                        self.finder_info.idx_tab,
                                        self.finder_info.tab.clone(),
                                        self.finder_info.parameters.clone(),
                                    ))
                                    .inspect_err(JujikError::handle_err);
                            }

                            self.finder_info.show = false;
                        }
                    },
                );
            })
        });

        if modal.backdrop_response.clicked() {
            self.finder_info.show = false;
        }
    }

    fn finder_change_permissions(&mut self, ctx: &Context) {
        let modal = Modal::new(Id::new("Finder Permissions")).show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.label(
                    RichText::new(format!("Finder Permissions: {}", self.finder_info.path))
                        .color(self.style.text_color.into_color32())
                        .size(self.style.text_size),
                );

                ui.separator();

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("User:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.checkbox(
                            &mut self.finder_info.change_permissions.user.0,
                            RichText::new("execute")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.finder_info.change_permissions.user.1,
                            RichText::new("write")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.finder_info.change_permissions.user.2,
                            RichText::new("read")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Group:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.checkbox(
                            &mut self.finder_info.change_permissions.group.0,
                            RichText::new("execute")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.finder_info.change_permissions.group.1,
                            RichText::new("write")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.finder_info.change_permissions.group.2,
                            RichText::new("read")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Other:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.checkbox(
                            &mut self.finder_info.change_permissions.other.0,
                            RichText::new("execute")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.finder_info.change_permissions.other.1,
                            RichText::new("write")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.finder_info.change_permissions.other.2,
                            RichText::new("read")
                                .color(self.style.text_color.into_color32())
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
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            )
                            .clicked()
                        {
                            let _ = self
                                .controller
                                .send(Command::SetConfig(Config::new(
                                    self.style.clone(),
                                    self.pins.clone(),
                                    self.tabs.clone(),
                                    self.current_tab_idx,
                                    self.entitys_show.clone(),
                                )))
                                .inspect_err(JujikError::handle_err);

                            if self.finder_info.change_permissions.user.0 {
                                self.finder_info.permissions.set(
                                    EntityPermissionsCategory::User,
                                    EntityPermissionsKind::Execute,
                                );
                            } else {
                                self.finder_info.permissions.unset(
                                    EntityPermissionsCategory::User,
                                    EntityPermissionsKind::Execute,
                                );
                            }

                            if self.finder_info.change_permissions.user.1 {
                                self.finder_info.permissions.set(
                                    EntityPermissionsCategory::User,
                                    EntityPermissionsKind::Write,
                                );
                            } else {
                                self.finder_info.permissions.unset(
                                    EntityPermissionsCategory::User,
                                    EntityPermissionsKind::Write,
                                );
                            }

                            if self.finder_info.change_permissions.user.2 {
                                self.finder_info.permissions.set(
                                    EntityPermissionsCategory::User,
                                    EntityPermissionsKind::Read,
                                );
                            } else {
                                self.finder_info.permissions.unset(
                                    EntityPermissionsCategory::User,
                                    EntityPermissionsKind::Read,
                                );
                            }

                            if self.finder_info.change_permissions.group.0 {
                                self.finder_info.permissions.set(
                                    EntityPermissionsCategory::Group,
                                    EntityPermissionsKind::Execute,
                                );
                            } else {
                                self.finder_info.permissions.unset(
                                    EntityPermissionsCategory::Group,
                                    EntityPermissionsKind::Execute,
                                );
                            }

                            if self.finder_info.change_permissions.group.1 {
                                self.finder_info.permissions.set(
                                    EntityPermissionsCategory::Group,
                                    EntityPermissionsKind::Write,
                                );
                            } else {
                                self.finder_info.permissions.unset(
                                    EntityPermissionsCategory::Group,
                                    EntityPermissionsKind::Write,
                                );
                            }

                            if self.finder_info.change_permissions.group.2 {
                                self.finder_info.permissions.set(
                                    EntityPermissionsCategory::Group,
                                    EntityPermissionsKind::Read,
                                );
                            } else {
                                self.finder_info.permissions.unset(
                                    EntityPermissionsCategory::Group,
                                    EntityPermissionsKind::Read,
                                );
                            }

                            if self.finder_info.change_permissions.other.0 {
                                self.finder_info.permissions.set(
                                    EntityPermissionsCategory::Other,
                                    EntityPermissionsKind::Execute,
                                );
                            } else {
                                self.finder_info.permissions.unset(
                                    EntityPermissionsCategory::Other,
                                    EntityPermissionsKind::Execute,
                                );
                            }

                            if self.finder_info.change_permissions.other.1 {
                                self.finder_info.permissions.set(
                                    EntityPermissionsCategory::Other,
                                    EntityPermissionsKind::Write,
                                );
                            } else {
                                self.finder_info.permissions.unset(
                                    EntityPermissionsCategory::Other,
                                    EntityPermissionsKind::Write,
                                );
                            }

                            if self.finder_info.change_permissions.other.2 {
                                self.finder_info.permissions.set(
                                    EntityPermissionsCategory::Other,
                                    EntityPermissionsKind::Read,
                                );
                            } else {
                                self.finder_info.permissions.unset(
                                    EntityPermissionsCategory::Other,
                                    EntityPermissionsKind::Read,
                                );
                            }

                            self.finder_info.change_permissions.show = false;
                        }
                    },
                );
            });
        });

        if modal.backdrop_response.clicked() {
            self.finder_info.change_permissions.show = false;
        }
    }

    fn finder_change_owners(&mut self, ctx: &Context) {
        let modal =
            Modal::new(Id::new("Finder Owners")).show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.label(
                        RichText::new(format!("Finder Owners: {}", self.finder_info.path))
                            .color(self.style.text_color.into_color32())
                            .size(self.style.text_size),
                    );

                    ui.separator();

                    Sides::new().show(
                        ui,
                        |ui| {
                            ui.label(
                                RichText::new("User:")
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            );
                        },
                        |ui| {
                            ui.label(
                                RichText::new(format!("{}", self.finder_info.change_owners.uid))
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            );

                            ui.text_edit_singleline(&mut self.finder_info.change_owners.username);
                        },
                    );

                    Sides::new().show(
                        ui,
                        |ui| {
                            ui.label(
                                RichText::new("Group:")
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            );
                        },
                        |ui| {
                            ui.label(
                                RichText::new(format!("{}", self.finder_info.change_owners.gid))
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            );

                            ui.text_edit_singleline(&mut self.finder_info.change_owners.groupname);
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
                                        .color(self.style.text_color.into_color32())
                                        .size(self.style.text_size),
                                )
                                .clicked()
                            {
                                let _ = self
                                    .controller
                                    .send(Command::SetConfig(Config::new(
                                        self.style.clone(),
                                        self.pins.clone(),
                                        self.tabs.clone(),
                                        self.current_tab_idx,
                                        self.entitys_show.clone(),
                                    )))
                                    .inspect_err(JujikError::handle_err);

                                if self
                                    .finder_info
                                    .owners
                                    .username()
                                    .ne(&self.finder_info.change_owners.username)
                                {
                                    if let Err(_) = self.finder_info.owners.set_username(
                                        self.finder_info.change_owners.username.clone(),
                                    ) {
                                        //TODO handle error
                                        self.finder_info
                                            .change_owners
                                            .username
                                            .clone_from(&self.finder_info.owners.username());
                                    }
                                }

                                if self
                                    .finder_info
                                    .owners
                                    .groupname()
                                    .ne(&self.finder_info.change_owners.groupname)
                                {
                                    if let Err(_) = self.finder_info.owners.set_groupname(
                                        self.finder_info.change_owners.groupname.clone(),
                                    ) {
                                        //TODO handle error
                                        self.finder_info
                                            .change_owners
                                            .groupname
                                            .clone_from(&self.finder_info.owners.groupname());
                                    }
                                }

                                self.finder_info.change_owners.show = false;
                            }
                        },
                    );
                });
            });

        if modal.backdrop_response.clicked() {
            self.finder_info.change_owners.show = false;
        }
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
            if entity.is_dir() {
                let open = ui.button(
                    RichText::new("Open")
                        .color(self.style.text_color.into_color32())
                        .size(self.style.text_size),
                );

                let open_new_tab = ui.button(
                    RichText::new("Open in new Tab")
                        .color(self.style.text_color.into_color32())
                        .size(self.style.text_size),
                );

                if open.clicked() {
                    let _ = self
                        .controller
                        .send(Command::ChangeTabDirectory(
                            self.current_tab_idx,
                            tab.clone(),
                            Some(entity.path()),
                        ))
                        .inspect_err(JujikError::handle_err);

                    ui.close_menu();
                }

                if open_new_tab.clicked() {
                    let _ = self
                        .controller
                        .send(Command::CreateEntitys(entity.path()))
                        .inspect_err(JujikError::handle_err);

                    ui.close_menu();
                }
            } else {
                let view = ui.button(
                    RichText::new("View")
                        .color(self.style.text_color.into_color32())
                        .size(self.style.text_size),
                );

                let edit = ui.button(
                    RichText::new("Edit")
                        .color(self.style.text_color.into_color32())
                        .size(self.style.text_size),
                );

                if view.clicked() {
                    let _ = self
                        .controller
                        .send(Command::CreateView(entity.path()))
                        .inspect_err(JujikError::handle_err);

                    ui.close_menu();
                }

                if edit.clicked() {
                    if entity.is_file() {
                        let _ = self
                            .controller
                            .send(Command::CreateEditor(entity.path()))
                            .inspect_err(JujikError::handle_err);
                    }

                    ui.close_menu();
                }
            }

            let select = ui.button(
                RichText::new("Select")
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
            );

            let copy = ui.button(
                RichText::new("Copy")
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
            );

            let cut = ui.button(
                RichText::new("Cut")
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
            );

            let create_pin = ui.button(
                RichText::new("Create Pin")
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
            );

            let delete = ui.button(
                RichText::new("Delete")
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
            );

            let info = ui.button(
                RichText::new("Info")
                    .color(self.style.text_color.into_color32())
                    .size(self.style.text_size),
            );

            if select.clicked() {
                if self.entitys_selection.entitys.contains(entity) {
                    self.entitys_selection.entitys.remove(entity);
                } else {
                    self.entitys_selection.entitys.insert(entity.clone());
                }

                ui.close_menu();
            }

            if copy.clicked() {
                self.entitys_selection.copy();

                ui.close_menu();
            }

            if cut.clicked() {
                self.entitys_selection.cut();

                ui.close_menu();
            }

            if create_pin.clicked() {
                if entity.is_dir() {
                    let _ = self
                        .controller
                        .send(Command::CreatePin(entity.path()))
                        .inspect_err(JujikError::handle_err);
                } else {
                    let _ = self
                        .controller
                        .send(Command::CreatePin(entity.path_dir()))
                        .inspect_err(JujikError::handle_err);
                }

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
                self.entity_info.path = entity.path_dir_str();
                self.entity_info.name = entity.name();
                self.entity_info.extension = entity.extension_str();
                self.entity_info.kind = entity.kind().clone();
                self.entity_info.permissions = entity.permissions().clone();
                self.entity_info.owners = entity.owners().clone();
                self.entity_info.size = entity.size().clone();
                self.entity_info.modification = entity.modification().clone();
                self.entity_info.creation = entity.creation().clone();

                ui.close_menu();
            }
        });
    }

    fn entity_delete(&mut self, ctx: &Context) {
        let modal = Modal::new(Id::new("Entity Delete: {}")).show(ctx, |ui| {
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
                    .color(self.style.text_color.into_color32())
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
                                    .color(self.style.text_color.into_color32())
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
                                    .color(self.style.text_color.into_color32())
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

    fn entity_create(&mut self, ctx: &Context) {
        let modal = Modal::new(Id::new("Entity Create")).show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.label(
                    RichText::new("Entity Create")
                        .color(self.style.text_color.into_color32())
                        .size(self.style.text_size),
                );

                ui.separator();

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Path:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.text_edit_singleline(&mut self.entity_create.path);
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Name:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.text_edit_singleline(&mut self.entity_create.name);
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Extension:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.text_edit_singleline(&mut self.entity_create.extension);
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Permissions:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        let permissions = ui.add(
                            Label::new(
                                RichText::new(format!("{}", self.entity_create.permissions))
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            )
                            .selectable(true),
                        );

                        permissions.context_menu(|ui| {
                            let change = ui.button(
                                RichText::new("Change")
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            );

                            if change.clicked() {
                                self.entity_create.change_permissions.user = (
                                    self.entity_create.permissions.has(
                                        EntityPermissionsCategory::User,
                                        EntityPermissionsKind::Execute,
                                    ),
                                    self.entity_create.permissions.has(
                                        EntityPermissionsCategory::User,
                                        EntityPermissionsKind::Write,
                                    ),
                                    self.entity_create.permissions.has(
                                        EntityPermissionsCategory::User,
                                        EntityPermissionsKind::Read,
                                    ),
                                );
                                self.entity_create.change_permissions.group = (
                                    self.entity_create.permissions.has(
                                        EntityPermissionsCategory::Group,
                                        EntityPermissionsKind::Execute,
                                    ),
                                    self.entity_create.permissions.has(
                                        EntityPermissionsCategory::Group,
                                        EntityPermissionsKind::Write,
                                    ),
                                    self.entity_create.permissions.has(
                                        EntityPermissionsCategory::Group,
                                        EntityPermissionsKind::Read,
                                    ),
                                );
                                self.entity_create.change_permissions.other = (
                                    self.entity_create.permissions.has(
                                        EntityPermissionsCategory::Other,
                                        EntityPermissionsKind::Execute,
                                    ),
                                    self.entity_create.permissions.has(
                                        EntityPermissionsCategory::Other,
                                        EntityPermissionsKind::Write,
                                    ),
                                    self.entity_create.permissions.has(
                                        EntityPermissionsCategory::Other,
                                        EntityPermissionsKind::Read,
                                    ),
                                );

                                self.entity_create.change_permissions.show = true;
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
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            )
                            .clicked()
                        {
                            let _ = self
                                .controller
                                .send(Command::SetConfig(Config::new(
                                    self.style.clone(),
                                    self.pins.clone(),
                                    self.tabs.clone(),
                                    self.current_tab_idx,
                                    self.entitys_show.clone(),
                                )))
                                .inspect_err(JujikError::handle_err);

                            self.entity_create
                                .entity
                                .set_path(PathBuf::from(self.entity_create.path.clone()));

                            self.entity_create
                                .entity
                                .set_name(self.entity_create.name.clone());

                            self.entity_create
                                .entity
                                .set_extension(self.entity_create.extension.clone());

                            self.entity_create
                                .entity
                                .set_permissions(self.entity_create.permissions.clone());

                            let _ = self
                                .controller
                                .send(Command::CreateEntity(
                                    self.entity_create.idx_tab,
                                    self.entity_create.tab.clone(),
                                    self.entity_create.entity.clone(),
                                ))
                                .inspect_err(JujikError::handle_err);

                            self.entity_info.show = false;
                        }
                    },
                );
            });
        });

        if modal.backdrop_response.clicked() {
            self.entity_create.show = false;
        }
    }

    fn entity_create_permissions(&mut self, ctx: &Context) {
        let modal = Modal::new(Id::new("Entity Create Permissions")).show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.label(
                    RichText::new(format!("Create Permissions: {}", self.entity_create.name))
                        .color(self.style.text_color.into_color32())
                        .size(self.style.text_size),
                );

                ui.separator();

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("User:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.checkbox(
                            &mut self.entity_create.change_permissions.user.0,
                            RichText::new("execute")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.entity_create.change_permissions.user.1,
                            RichText::new("write")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.entity_create.change_permissions.user.2,
                            RichText::new("read")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Group:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.checkbox(
                            &mut self.entity_create.change_permissions.group.0,
                            RichText::new("execute")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.entity_create.change_permissions.group.1,
                            RichText::new("write")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.entity_create.change_permissions.group.2,
                            RichText::new("read")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Other:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.checkbox(
                            &mut self.entity_create.change_permissions.other.0,
                            RichText::new("execute")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.entity_create.change_permissions.other.1,
                            RichText::new("write")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.entity_create.change_permissions.other.2,
                            RichText::new("read")
                                .color(self.style.text_color.into_color32())
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
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            )
                            .clicked()
                        {
                            let _ = self
                                .controller
                                .send(Command::SetConfig(Config::new(
                                    self.style.clone(),
                                    self.pins.clone(),
                                    self.tabs.clone(),
                                    self.current_tab_idx,
                                    self.entitys_show.clone(),
                                )))
                                .inspect_err(JujikError::handle_err);

                            if self.entity_create.change_permissions.user.0 {
                                self.entity_create.permissions.set(
                                    EntityPermissionsCategory::User,
                                    EntityPermissionsKind::Execute,
                                );
                            } else {
                                self.entity_create.permissions.unset(
                                    EntityPermissionsCategory::User,
                                    EntityPermissionsKind::Execute,
                                );
                            }

                            if self.entity_create.change_permissions.user.1 {
                                self.entity_create.permissions.set(
                                    EntityPermissionsCategory::User,
                                    EntityPermissionsKind::Write,
                                );
                            } else {
                                self.entity_create.permissions.unset(
                                    EntityPermissionsCategory::User,
                                    EntityPermissionsKind::Write,
                                );
                            }

                            if self.entity_create.change_permissions.user.2 {
                                self.entity_create.permissions.set(
                                    EntityPermissionsCategory::User,
                                    EntityPermissionsKind::Read,
                                );
                            } else {
                                self.entity_create.permissions.unset(
                                    EntityPermissionsCategory::User,
                                    EntityPermissionsKind::Read,
                                );
                            }

                            if self.entity_create.change_permissions.group.0 {
                                self.entity_create.permissions.set(
                                    EntityPermissionsCategory::Group,
                                    EntityPermissionsKind::Execute,
                                );
                            } else {
                                self.entity_create.permissions.unset(
                                    EntityPermissionsCategory::Group,
                                    EntityPermissionsKind::Execute,
                                );
                            }

                            if self.entity_create.change_permissions.group.1 {
                                self.entity_create.permissions.set(
                                    EntityPermissionsCategory::Group,
                                    EntityPermissionsKind::Write,
                                );
                            } else {
                                self.entity_create.permissions.unset(
                                    EntityPermissionsCategory::Group,
                                    EntityPermissionsKind::Write,
                                );
                            }

                            if self.entity_create.change_permissions.group.2 {
                                self.entity_create.permissions.set(
                                    EntityPermissionsCategory::Group,
                                    EntityPermissionsKind::Read,
                                );
                            } else {
                                self.entity_create.permissions.unset(
                                    EntityPermissionsCategory::Group,
                                    EntityPermissionsKind::Read,
                                );
                            }

                            if self.entity_create.change_permissions.other.0 {
                                self.entity_create.permissions.set(
                                    EntityPermissionsCategory::Other,
                                    EntityPermissionsKind::Execute,
                                );
                            } else {
                                self.entity_create.permissions.unset(
                                    EntityPermissionsCategory::Other,
                                    EntityPermissionsKind::Execute,
                                );
                            }

                            if self.entity_create.change_permissions.other.1 {
                                self.entity_create.permissions.set(
                                    EntityPermissionsCategory::Other,
                                    EntityPermissionsKind::Write,
                                );
                            } else {
                                self.entity_create.permissions.unset(
                                    EntityPermissionsCategory::Other,
                                    EntityPermissionsKind::Write,
                                );
                            }

                            if self.entity_create.change_permissions.other.2 {
                                self.entity_create.permissions.set(
                                    EntityPermissionsCategory::Other,
                                    EntityPermissionsKind::Read,
                                );
                            } else {
                                self.entity_create.permissions.unset(
                                    EntityPermissionsCategory::Other,
                                    EntityPermissionsKind::Read,
                                );
                            }

                            self.entity_create.change_permissions.show = false;
                        }
                    },
                );
            });
        });

        if modal.backdrop_response.clicked() {
            self.entity_create.change_permissions.show = false;
        }
    }

    fn entity_info(&mut self, ctx: &Context) {
        let modal = Modal::new(Id::new("Entity Info")).show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.label(
                    RichText::new(format!("Entity Info: {}", self.entity_info.name))
                        .color(self.style.text_color.into_color32())
                        .size(self.style.text_size),
                );

                ui.separator();

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Path:")
                                .color(self.style.text_color.into_color32())
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
                            RichText::new("Name:")
                                .color(self.style.text_color.into_color32())
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
                            RichText::new("Extension:")
                                .color(self.style.text_color.into_color32())
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
                            RichText::new("Kind:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.add(
                            Label::new(
                                RichText::new(format!("{:?}", self.entity_info.kind))
                                    .color(self.style.text_color.into_color32())
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
                            RichText::new("Permissions:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        let permissions = ui.add(
                            Label::new(
                                RichText::new(format!("{}", self.entity_info.permissions))
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            )
                            .selectable(true),
                        );

                        permissions.context_menu(|ui| {
                            let change = ui.button(
                                RichText::new("Change")
                                    .color(self.style.text_color.into_color32())
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
                            RichText::new("Owners:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        let owners = ui.add(
                            Label::new(
                                RichText::new(format!("{}", self.entity_info.owners))
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            )
                            .selectable(true),
                        );

                        owners.context_menu(|ui| {
                            let change = ui.button(
                                RichText::new("Change")
                                    .color(self.style.text_color.into_color32())
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

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Size:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.label(
                            RichText::new(format!("{}", self.entity_info.size))
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Date modification:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.label(
                            RichText::new(format!("{}", self.entity_info.modification.date_str()))
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Date creation:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.label(
                            RichText::new(format!("{}", self.entity_info.creation.date_str()))
                                .color(self.style.text_color.into_color32())
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
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            )
                            .clicked()
                        {
                            let _ = self
                                .controller
                                .send(Command::SetConfig(Config::new(
                                    self.style.clone(),
                                    self.pins.clone(),
                                    self.tabs.clone(),
                                    self.current_tab_idx,
                                    self.entitys_show.clone(),
                                )))
                                .inspect_err(JujikError::handle_err);

                            if self
                                .entity_info
                                .entity
                                .path_str()
                                .ne(&self.entity_info.path)
                            {
                                let _ = self
                                    .controller
                                    .send(Command::MoveEntitys(
                                        self.entity_info.idx_tab,
                                        self.entity_info.tab.clone(),
                                        self.entity_info.idx_entity,
                                        vec![self.entity_info.entity.clone()],
                                        PathBuf::from(self.entity_info.path.clone()),
                                    ))
                                    .inspect_err(JujikError::handle_err);
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

                            self.entity_info.show = false;
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
        let modal = Modal::new(Id::new("Entity Change Permissions")).show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.label(
                    RichText::new(format!("Change Permissions: {}", self.entity_info.name))
                        .color(self.style.text_color.into_color32())
                        .size(self.style.text_size),
                );

                ui.separator();

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("User:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.checkbox(
                            &mut self.entity_info.change_permissions.user.0,
                            RichText::new("execute")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.entity_info.change_permissions.user.1,
                            RichText::new("write")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.entity_info.change_permissions.user.2,
                            RichText::new("read")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Group:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.checkbox(
                            &mut self.entity_info.change_permissions.group.0,
                            RichText::new("execute")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.entity_info.change_permissions.group.1,
                            RichText::new("write")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.entity_info.change_permissions.group.2,
                            RichText::new("read")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Other:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.checkbox(
                            &mut self.entity_info.change_permissions.other.0,
                            RichText::new("execute")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.entity_info.change_permissions.other.1,
                            RichText::new("write")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                        ui.checkbox(
                            &mut self.entity_info.change_permissions.other.2,
                            RichText::new("read")
                                .color(self.style.text_color.into_color32())
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
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            )
                            .clicked()
                        {
                            let _ = self
                                .controller
                                .send(Command::SetConfig(Config::new(
                                    self.style.clone(),
                                    self.pins.clone(),
                                    self.tabs.clone(),
                                    self.current_tab_idx,
                                    self.entitys_show.clone(),
                                )))
                                .inspect_err(JujikError::handle_err);

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
            Modal::new(Id::new("Entity Change Owners")).show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.label(
                        RichText::new(format!("Change Owners: {}", self.entity_info.name))
                            .color(self.style.text_color.into_color32())
                            .size(self.style.text_size),
                    );

                    ui.separator();

                    Sides::new().show(
                        ui,
                        |ui| {
                            ui.label(
                                RichText::new("User:")
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            );
                        },
                        |ui| {
                            ui.label(
                                RichText::new(format!("{}", self.entity_info.change_owners.uid))
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            );

                            ui.text_edit_singleline(&mut self.entity_info.change_owners.username);
                        },
                    );

                    Sides::new().show(
                        ui,
                        |ui| {
                            ui.label(
                                RichText::new("Group:")
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            );
                        },
                        |ui| {
                            ui.label(
                                RichText::new(format!("{}", self.entity_info.change_owners.gid))
                                    .color(self.style.text_color.into_color32())
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
                                        .color(self.style.text_color.into_color32())
                                        .size(self.style.text_size),
                                )
                                .clicked()
                            {
                                let _ = self
                                    .controller
                                    .send(Command::SetConfig(Config::new(
                                        self.style.clone(),
                                        self.pins.clone(),
                                        self.tabs.clone(),
                                        self.current_tab_idx,
                                        self.entitys_show.clone(),
                                    )))
                                    .inspect_err(JujikError::handle_err);

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
            self.entitys_selection.copy();
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
            self.entitys_selection.cut();
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
            self.entitys_selection
                .paste(self.controller.clone(), pathbuf);
        }
    }

    fn entitys_sortby_info(&mut self, ctx: &Context, idx_tab: usize, tab: &Tab) {
        let modal = Modal::new(Id::new("SortBy Info")).show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.label(
                    RichText::new("SortBy Into")
                        .color(self.style.text_color.into_color32())
                        .size(self.style.text_size),
                );

                ui.separator();

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Field:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ComboBox::from_id_salt("Sort Field")
                            .selected_text(format!("{:?}", self.entitys_sortby_info.field))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.entitys_sortby_info.field,
                                    SortField::Name,
                                    "Name",
                                );
                                ui.selectable_value(
                                    &mut self.entitys_sortby_info.field,
                                    SortField::Extension,
                                    "Extension",
                                );
                                ui.selectable_value(
                                    &mut self.entitys_sortby_info.field,
                                    SortField::Permissions,
                                    "Permissions",
                                );
                                ui.selectable_value(
                                    &mut self.entitys_sortby_info.field,
                                    SortField::Owners,
                                    "Owners",
                                );
                                ui.selectable_value(
                                    &mut self.entitys_sortby_info.field,
                                    SortField::Size,
                                    "Size",
                                );
                                ui.selectable_value(
                                    &mut self.entitys_sortby_info.field,
                                    SortField::Modification,
                                    "Modification",
                                );
                                ui.selectable_value(
                                    &mut self.entitys_sortby_info.field,
                                    SortField::Creation,
                                    "Creation",
                                );
                            });
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Direction:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ComboBox::from_id_salt("Sort Direction")
                            .selected_text(format!("{:?}", self.entitys_sortby_info.direction))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.entitys_sortby_info.direction,
                                    SortDirection::Ascending,
                                    "Ascending",
                                );
                                ui.selectable_value(
                                    &mut self.entitys_sortby_info.direction,
                                    SortDirection::Descending,
                                    "Descending",
                                );
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
                                RichText::new("Sort")
                                    .color(self.style.text_color.into_color32())
                                    .size(self.style.text_size),
                            )
                            .clicked()
                        {
                            if self
                                .entitys_sortby_info
                                .field
                                .ne(&self.entitys_sortby_info.sortby.field)
                                || self
                                    .entitys_sortby_info
                                    .direction
                                    .ne(&self.entitys_sortby_info.sortby.direction)
                            {
                                self.entitys_sortby_info
                                    .sortby
                                    .field
                                    .clone_from(&self.entitys_sortby_info.field);
                                self.entitys_sortby_info
                                    .sortby
                                    .direction
                                    .clone_from(&self.entitys_sortby_info.direction);

                                let _ = self
                                    .controller
                                    .send(Command::ChangeEntitysSortBy(
                                        idx_tab,
                                        tab.clone(),
                                        self.entitys_sortby_info.sortby.clone(),
                                    ))
                                    .inspect_err(JujikError::handle_err);
                            }

                            self.entitys_sortby_info.show = false;
                        }
                    },
                );
            })
        });

        if modal.backdrop_response.clicked() {
            self.entitys_sortby_info.show = false;
        }
    }
}

// Style
impl JujikView {
    fn style_info(&mut self, ctx: &Context) {
        let modal = Modal::new(Id::new("Style Info")).show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.label(
                    RichText::new("Style Info")
                        .color(self.style.text_color.into_color32())
                        .size(self.style.text_size),
                );

                ui.separator();

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Primary:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.color_edit_button_srgb(self.style.primary_color.array_mut());
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Background:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.color_edit_button_srgb(self.style.background_color.array_mut());
                    },
                );

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Selection:")
                                .color(self.style.text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.color_edit_button_srgb(self.style.selection_color.array_mut());
                    },
                );

                let text_color = self.style.text_color.clone();

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Text color:")
                                .color(text_color.into_color32())
                                .size(self.style.text_size),
                        );
                    },
                    |ui| {
                        ui.color_edit_button_srgb(self.style.text_color.array_mut());
                    },
                );

                let text_size = self.style.text_size;

                Sides::new().show(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new("Text size:")
                                .color(self.style.text_color.into_color32())
                                .size(text_size),
                        );
                    },
                    |ui| {
                        ui.add(DragValue::new(&mut self.style.text_size));
                    },
                );
            });
        });

        if modal.backdrop_response.clicked() {
            self.style.show = false;
        }
    }
}

impl EntitysSelection {
    fn entitys_vec(&self) -> Vec<Entity> {
        self.entitys.clone().into_iter().collect()
    }

    fn copy(&mut self) {
        self.move_kind = EntitysMoveKind::Copy(self.entitys_vec());
    }

    fn cut(&mut self) {
        self.move_kind = EntitysMoveKind::Cut(self.entitys_vec());
    }

    fn paste(&mut self, controller: Sender<Command>, pathbuf: PathBuf) {
        match &self.move_kind {
            EntitysMoveKind::Copy(entitys) => {
                let _ = controller
                    .send(Command::CopyEntitys(
                        0,
                        Tab::default(),
                        0,
                        entitys.clone(),
                        pathbuf.clone(),
                    ))
                    .inspect_err(JujikError::handle_err);
            }
            EntitysMoveKind::Cut(entitys) => {
                let _ = controller
                    .send(Command::MoveEntitys(
                        0,
                        Tab::default(),
                        0,
                        entitys.clone(),
                        pathbuf.clone(),
                    ))
                    .inspect_err(JujikError::handle_err);
            }
            _ => {}
        }
    }
}

impl JujikColor {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Self { color: [r, g, b] }
    }

    fn from_color32(color: Color32) -> Self {
        Self {
            color: [color.r(), color.g(), color.b()],
        }
    }

    fn into_color32(&self) -> Color32 {
        Color32::from_rgb(self.color[0], self.color[1], self.color[2])
    }

    fn array_mut(&mut self) -> &mut [u8; 3] {
        &mut self.color
    }
}

impl Default for EntitysShowColumn {
    fn default() -> Self {
        Self {
            filekind: true,
            name: true,
            name_with_extension: true,
            extension: true,
            permissions: true,
            owners: true,
            size: true,
            date_modification: true,
            date_creation: true,
        }
    }
}

impl Default for JujikStyle {
    fn default() -> Self {
        Self {
            show: false,
            primary_color: JujikColor::new(100, 100, 100),
            background_color: JujikColor::new(50, 50, 50),
            selection_color: JujikColor::new(50, 80, 50),
            text_color: JujikColor::from_color32(Color32::LIGHT_GRAY),
            text_size: 18.0,
        }
    }
}
