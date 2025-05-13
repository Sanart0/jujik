use crate::entity::Entity;
use crate::tab::{TabContent, TabKind};
use crate::{commands::Command, error::JujikError, pin::Pin, tab::Tab};
use eframe::{App, EventLoopBuilderHook, NativeOptions, run_native};
use egui::{
    Align, Button, CentralPanel, Color32, Context, Id, Label, Layout, Modal, PointerButton,
    Response, RichText, ScrollArea, Sense, SidePanel, Sides, Stroke, TopBottomPanel, Ui, Visuals,
    menu,
};
use egui_extras::{Column, TableBuilder};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::usize;
use std::{
    sync::mpsc::Receiver,
    thread::{self, JoinHandle},
};
use winit::platform::wayland::EventLoopBuilderExtWayland;

struct PinInfo {
    show: bool,
    idx: usize,
    pin: Pin,
    name: String,
    path: String,
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
    text_color: Color32,
    text_size: f32,
}

pub struct JujikView {
    controller: Sender<Command>,
    view: Receiver<Command>,
    pins: Vec<Pin>,
    tabs: Vec<Tab>,
    entitys_show: ShowEntitysColumn,
    current_tab_idx: usize,
    style: JujikStyle,
    pin_info: PinInfo,
}

impl JujikView {
    pub fn new(controller: Sender<Command>, view: Receiver<Command>) -> Self {
        Self {
            controller,
            view,
            pins: Vec::new(),
            tabs: Vec::new(),
            entitys_show: ShowEntitysColumn::default(),
            current_tab_idx: 0,
            style: JujikStyle::default(),
            pin_info: PinInfo {
                show: false,
                idx: 0,
                pin: Pin::empty(),
                name: String::new(),
                path: String::new(),
            },
        }
    }

    pub fn run(mut self) -> Result<JoinHandle<Result<(), JujikError>>, JujikError> {
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
                Self::read_save(&mut self);
                run_native("Jujik", native_options, Box::new(|_cc| Ok(Box::new(self))))?;
                Ok(())
            },
        )?)
    }

    fn read_save(&mut self) {
        //TODO read a save
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
        visuals.selection.bg_fill = self.style.background_color;

        ctx.set_visuals(visuals);
    }
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
                self.tab(ui);
            });
            CentralPanel::default().show_inside(ui, |ui| {
                self.tab_content(ui);
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
                        let _ = self
                            .controller
                            .send(Command::ChangePinPosition(idx, idx - 1, pin.clone()))
                            .inspect_err(JujikError::handle_err);
                    }

                    if down.clicked() {
                        let _ = self
                            .controller
                            .send(Command::ChangePinPosition(idx, idx + 1, pin.clone()))
                            .inspect_err(JujikError::handle_err);
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
                    .send(Command::DeletePin(idx))
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
        let modal =
            Modal::new(Id::new(format!("Pin Info: {}", self.pin_info.name))).show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
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
                                            self.pin_info.name.clone(),
                                        ))
                                        .inspect_err(JujikError::handle_err);
                                }

                                if self.pin_info.pin.path_str().ne(&self.pin_info.path) {
                                    let _ = self
                                        .controller
                                        .send(Command::ChangePinDirectory(
                                            self.pin_info.idx,
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
    fn tab(&mut self, ui: &mut Ui) {
        let mut scroll = ui.ctx().style().spacing.scroll;
        scroll.floating = false;
        scroll.bar_width = 4.0;
        scroll.bar_inner_margin = 4.0;
        scroll.bar_outer_margin = 4.0;
        scroll.foreground_color = false;
        ui.ctx().all_styles_mut(|s| s.spacing.scroll = scroll);

        ui.horizontal(|ui| {
            ScrollArea::horizontal().show(ui, |ui| {
                for (idx, tab) in self.tabs.iter().enumerate() {
                    let response = ui.selectable_label(
                        self.current_tab_idx == idx,
                        RichText::new(tab.name())
                            .color(self.style.text_color)
                            .size(self.style.text_size),
                    );

                    if response.clicked() {
                        self.current_tab_idx = idx;
                    }

                    self.tab_context_menu(ui, &response, idx, tab);
                }
            });
        });
    }

    fn tab_content(&mut self, ui: &mut Ui) {
        if self.current_tab_idx < self.tabs.len() {
            if let Some(tab) = self.tabs.get(self.current_tab_idx) {
                match tab.content() {
                    TabContent::Entitys(_) => {
                        ui.horizontal(|ui| {
                            self.go_prev_directory(ui, tab.clone());
                            self.tab_path(ui, tab.clone());
                        });

                        ui.separator();

                        self.table(ui, tab.clone())
                    }
                    TabContent::Editor(_pathbuf) => todo!(),
                    _ => {}
                }
            }
        }
    }

    fn table(&self, ui: &mut Ui, mut tab: Tab) {
        let tab_clone = tab.clone();
        let entitys = if let Some(entitys) = tab.entitys_mut() {
            entitys
        } else {
            &mut Vec::new()
        };

        let mut scroll = ui.ctx().style().spacing.scroll;
        scroll.floating = false;
        scroll.bar_width = 4.0;
        scroll.bar_inner_margin = 4.0;
        scroll.bar_outer_margin = 4.0;
        scroll.foreground_color = false;
        ui.ctx().all_styles_mut(|s| s.spacing.scroll = scroll);

        ScrollArea::horizontal().show(ui, |ui| {
            TableBuilder::new(ui)
                // .resizable(true)
                .cell_layout(Layout::left_to_right(Align::Center))
                .sense(Sense::click())
                .column(Column::remainder())
                .column(Column::remainder())
                .column(Column::remainder())
                .column(Column::remainder())
                .column(Column::remainder())
                .column(Column::remainder())
                .column(Column::remainder())
                .column(Column::remainder())
                .header(10.0, |mut header| {
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
                    for entity in entitys {
                        body.row(10.0, |mut row| {
                            if self.entitys_show.filekind {
                                row.col(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        let response = ui.add(
                                            Label::new(
                                                RichText::new(format!("{}", entity.kind()))
                                                    .color(self.style.text_color)
                                                    .size(self.style.text_size),
                                            )
                                            .selectable(false)
                                            .sense(Sense::click()),
                                        );

                                        self.table_entity_context_menu(
                                            &response, &tab_clone, entity,
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

                                        let response = ui.add(
                                            Label::new(
                                                RichText::new(format!("{}", name))
                                                    .color(self.style.text_color)
                                                    .size(self.style.text_size),
                                            )
                                            .selectable(false)
                                            .sense(Sense::click()),
                                        );

                                        if response.double_clicked() {
                                            self.go_next_directory(&tab_clone, entity.path());
                                        }

                                        self.table_entity_context_menu(
                                            &response, &tab_clone, entity,
                                        );
                                    });
                                });
                            }
                            if self.entitys_show.extension {
                                row.col(|ui| {
                                    let extension = if let Some(extension) = entity.extension() {
                                        extension.to_string()
                                    } else {
                                        String::new()
                                    };

                                    ui.centered_and_justified(|ui| {
                                        let response = ui.add(
                                            Label::new(
                                                RichText::new(format!("{}", extension))
                                                    .color(self.style.text_color)
                                                    .size(self.style.text_size),
                                            )
                                            .selectable(false)
                                            .sense(Sense::click()),
                                        );

                                        self.table_entity_context_menu(
                                            &response, &tab_clone, entity,
                                        );
                                    });
                                });
                            }
                            if self.entitys_show.permissions {
                                row.col(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        let response = ui.add(
                                            Label::new(
                                                RichText::new(format!("{}", entity.permissions()))
                                                    .color(self.style.text_color)
                                                    .size(self.style.text_size),
                                            )
                                            .selectable(false)
                                            .sense(Sense::click()),
                                        );

                                        self.table_entity_context_menu(
                                            &response, &tab_clone, entity,
                                        );
                                    });
                                });
                            }
                            if self.entitys_show.owners {
                                row.col(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        let response = ui.add(
                                            Label::new(
                                                RichText::new(format!("{}", entity.owners()))
                                                    .color(self.style.text_color)
                                                    .size(self.style.text_size),
                                            )
                                            .selectable(false)
                                            .sense(Sense::click()),
                                        );

                                        self.table_entity_context_menu(
                                            &response, &tab_clone, entity,
                                        );
                                    });
                                });
                            }
                            if self.entitys_show.size {
                                row.col(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        let response = ui.add(
                                            Label::new(
                                                RichText::new(format!("{}", 0))
                                                    .color(self.style.text_color)
                                                    .size(self.style.text_size),
                                            )
                                            .selectable(false)
                                            .sense(Sense::click()),
                                        );

                                        self.table_entity_context_menu(
                                            &response, &tab_clone, entity,
                                        );
                                    });
                                });
                            }
                            if self.entitys_show.modification_date {
                                row.col(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        let response = ui.add(
                                            Label::new(
                                                RichText::new(format!("{}", 0))
                                                    .color(self.style.text_color)
                                                    .size(self.style.text_size),
                                            )
                                            .selectable(false)
                                            .sense(Sense::click()),
                                        );

                                        self.table_entity_context_menu(
                                            &response, &tab_clone, entity,
                                        );
                                    });
                                });
                            }
                            if self.entitys_show.creation_data {
                                row.col(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        let response = ui.add(
                                            Label::new(
                                                RichText::new(format!("{}", 0))
                                                    .color(self.style.text_color)
                                                    .size(self.style.text_size),
                                            )
                                            .selectable(false)
                                            .sense(Sense::click()),
                                        );

                                        self.table_entity_context_menu(
                                            &response, &tab_clone, entity,
                                        );
                                    });
                                });
                            }
                        });
                    }
                });
        });
    }

    fn tab_context_menu(&self, ui: &mut Ui, response: &Response, idx: usize, tab: &Tab) {
        response.context_menu(|ui| {});
    }

    fn table_entity_context_menu(&self, response: &Response, tab: &Tab, entity: &mut Entity) {
        response.context_menu(|ui| {
            let open = ui.button(
                RichText::new("Open")
                    .color(self.style.text_color)
                    .size(self.style.text_size),
            );

            if open.clicked() {
                self.go_next_directory(tab, entity.path());
                ui.close_menu();
            }
        });
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
            text_color: Color32::LIGHT_GRAY,
            text_size: 18.0,
        }
    }
}
