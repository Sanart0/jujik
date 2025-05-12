use crate::entity::Entity;
use crate::tab::{TabContent, TabKind};
use crate::{commands::Command, error::JujikError, pin::Pin, tab::Tab};
use eframe::{App, EventLoopBuilderHook, NativeOptions, run_native};
use egui::{
    Align, Button, CentralPanel, Color32, Label, Layout, Response, RichText, ScrollArea, Sense,
    SidePanel, Stroke, TopBottomPanel, Ui, Visuals, menu,
};
use egui_extras::{Column, TableBuilder};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::{
    sync::mpsc::Receiver,
    thread::{self, JoinHandle},
};
use winit::platform::wayland::EventLoopBuilderExtWayland;

struct ShowEntitysColumn {
    filekind: bool,
    name: bool,
    extansion: bool,
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
    current_tab: usize,
    style: JujikStyle,
}

impl JujikView {
    pub fn new(controller: Sender<Command>, view: Receiver<Command>) -> Self {
        Self {
            controller,
            view,
            pins: Vec::new(),
            tabs: Vec::new(),
            entitys_show: ShowEntitysColumn::default(),
            current_tab: 0,
            style: JujikStyle::default(),
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

    fn pin(&self, ui: &mut Ui) {
        ui.separator();
        ui.label(
            RichText::new("Pin")
                .color(self.style.text_color)
                .size(self.style.text_size),
        );
        ui.separator();

        for pin in &self.pins {
            let pin_btn = Button::new(
                RichText::new(pin.name())
                    .color(self.style.text_color)
                    .size(self.style.text_size),
            )
            .fill(self.style.background_color);

            if ui.add(pin_btn).clicked() {
                let _ = self
                    .controller
                    .send(Command::CreateTab(TabKind::Entitys, pin.path()))
                    .inspect_err(JujikError::handle_err);
            }
        }
    }

    fn tabs(&mut self, ui: &mut Ui) {
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
                    if ui
                        .selectable_label(
                            self.current_tab == idx,
                            RichText::new(tab.name())
                                .color(self.style.text_color)
                                .size(self.style.text_size),
                        )
                        .clicked()
                    {
                        self.current_tab = idx;
                    }
                }
            });
        });
    }

    fn table(&self, ui: &mut Ui, tab: Tab, entitys: &Vec<Entity>) {
        let mut scroll = ui.ctx().style().spacing.scroll;
        scroll.floating = false;
        scroll.bar_width = 4.0;
        scroll.bar_inner_margin = 4.0;
        scroll.bar_outer_margin = 4.0;
        scroll.foreground_color = false;
        ui.ctx().all_styles_mut(|s| s.spacing.scroll = scroll);

        ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                TableBuilder::new(ui)
                    // .resizable(true)
                    .cell_layout(Layout::left_to_right(Align::Center))
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
                                ui.vertical_centered(|ui| {
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
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        RichText::new("Name")
                                            .color(self.style.text_color)
                                            .size(self.style.text_size),
                                    );
                                });
                            });
                        }
                        if self.entitys_show.extansion {
                            header.col(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        RichText::new("Extansion")
                                            .color(self.style.text_color)
                                            .size(self.style.text_size),
                                    );
                                });
                            });
                        }
                        if self.entitys_show.permissions {
                            header.col(|ui| {
                                ui.vertical_centered(|ui| {
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
                                ui.vertical_centered(|ui| {
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
                                ui.vertical_centered(|ui| {
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
                                ui.vertical_centered(|ui| {
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
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        RichText::new("Creation Data")
                                            .color(self.style.text_color)
                                            .size(self.style.text_size),
                                    );
                                });
                            });
                        }
                    })
                    .body(|body| {
                        body.rows(10.0, entitys.len(), |mut row| {
                            let entity = entitys.get(row.index());

                            if let Some(entity) = entity {
                                if self.entitys_show.filekind {
                                    row.col(|ui| {
                                        ui.vertical_centered(|ui| {
                                            let _response = ui.add(
                                                Label::new(
                                                    RichText::new(format!("{}", entity.kind()))
                                                        .color(self.style.text_color)
                                                        .size(self.style.text_size),
                                                )
                                                .selectable(false)
                                                .sense(Sense::click()),
                                            );
                                        });
                                    });
                                }
                                if self.entitys_show.name {
                                    row.col(|ui| {
                                        ui.vertical_centered(|ui| {
                                            let response = ui.add(
                                                Label::new(
                                                    RichText::new(format!("{}", entity.name()))
                                                        .color(self.style.text_color)
                                                        .size(self.style.text_size),
                                                )
                                                .selectable(false)
                                                .sense(Sense::click()),
                                            );

                                            self.handle_table_click(&response, &tab, &entity);
                                        });
                                    });
                                }
                                if self.entitys_show.extansion {
                                    row.col(|ui| {
                                        if let Some(extansion) = entity.extansion() {
                                            ui.vertical_centered(|ui| {
                                                let _response = ui.add(
                                                    Label::new(
                                                        RichText::new(format!("{}", extansion))
                                                            .color(self.style.text_color)
                                                            .size(self.style.text_size),
                                                    )
                                                    .selectable(false)
                                                    .sense(Sense::click()),
                                                );
                                            });
                                        } else {
                                            ui.vertical_centered(|ui| {
                                                let _response = ui.add(
                                                    Label::new(
                                                        RichText::new(format!(
                                                            "{:?}",
                                                            None::<String>
                                                        ))
                                                        .color(self.style.text_color)
                                                        .size(self.style.text_size),
                                                    )
                                                    .selectable(false)
                                                    .sense(Sense::click()),
                                                );
                                            });
                                        }
                                    });
                                }
                                if self.entitys_show.permissions {
                                    row.col(|ui| {
                                        ui.vertical_centered(|ui| {
                                            let _response = ui.add(
                                                Label::new(
                                                    RichText::new(format!(
                                                        "{}",
                                                        entity.permissions()
                                                    ))
                                                    .color(self.style.text_color)
                                                    .size(self.style.text_size),
                                                )
                                                .selectable(false)
                                                .sense(Sense::click()),
                                            );
                                        });
                                    });
                                }
                                if self.entitys_show.owners {
                                    row.col(|ui| {
                                        ui.vertical_centered(|ui| {
                                            let _response = ui.add(
                                                Label::new(
                                                    RichText::new(format!("{}", entity.owners()))
                                                        .color(self.style.text_color)
                                                        .size(self.style.text_size),
                                                )
                                                .selectable(false)
                                                .sense(Sense::click()),
                                            );
                                        });
                                    });
                                }
                                if self.entitys_show.size {
                                    row.col(|ui| {
                                        ui.vertical_centered(|ui| {
                                            let _response = ui.add(
                                                Label::new(
                                                    RichText::new(format!("{}", 0))
                                                        .color(self.style.text_color)
                                                        .size(self.style.text_size),
                                                )
                                                .selectable(false)
                                                .sense(Sense::click()),
                                            );
                                        });
                                    });
                                }
                                if self.entitys_show.modification_date {
                                    row.col(|ui| {
                                        ui.vertical_centered(|ui| {
                                            let _response = ui.add(
                                                Label::new(
                                                    RichText::new(format!("{}", 0))
                                                        .color(self.style.text_color)
                                                        .size(self.style.text_size),
                                                )
                                                .selectable(false)
                                                .sense(Sense::click()),
                                            );
                                        });
                                    });
                                }
                                if self.entitys_show.creation_data {
                                    row.col(|ui| {
                                        ui.vertical_centered(|ui| {
                                            let _response = ui.add(
                                                Label::new(
                                                    RichText::new(format!("{}", 0))
                                                        .color(self.style.text_color)
                                                        .size(self.style.text_size),
                                                )
                                                .selectable(false)
                                                .sense(Sense::click()),
                                            );
                                        });
                                    });
                                }
                            }
                        });
                    });
            });
    }

    fn handle_table_click(&self, response: &Response, tab: &Tab, entity: &Entity) {
        if response.double_clicked() {
            if self.tabs.get(self.current_tab).is_some() {
                let _ = self.controller.send(Command::ChangeTabDirectory(
                    self.current_tab,
                    tab.clone(),
                    entity.path(),
                ));
            }
        }
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
            });
        });

        SidePanel::left("Bind")
            .width_range(100.0..=300.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let _ = ui.button(
                        RichText::new("⇦")
                            .color(self.style.text_color)
                            .size(self.style.text_size),
                    );
                    let _ = ui.button(
                        RichText::new("⇧")
                            .color(self.style.text_color)
                            .size(self.style.text_size),
                    );
                    let _ = ui.button(
                        RichText::new("⇨")
                            .color(self.style.text_color)
                            .size(self.style.text_size),
                    );
                    let _ = ui.button(
                        RichText::new("⇩")
                            .color(self.style.text_color)
                            .size(self.style.text_size),
                    );
                });

                self.pin(ui);
            });

        CentralPanel::default().show(ctx, |ui| {
            TopBottomPanel::top("tab").show_inside(ui, |ui| {
                self.tabs(ui);
            });
            CentralPanel::default().show_inside(ui, |ui| {
                if let Some(tab) = self.tabs.get(self.current_tab) {
                    let tab_path = ui.add(
                        Label::new(
                            RichText::new(format!("{}", tab.path_str()))
                                .color(self.style.text_color)
                                .size(self.style.text_size),
                        )
                        .selectable(false)
                        .sense(Sense::click()),
                    );

                    match tab.content() {
                        TabContent::Entitys(entitys) => self.table(ui, tab.clone(), entitys),
                        TabContent::Editor(_pathbuf) => todo!(),
                        _ => {}
                    }
                }
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

impl Default for ShowEntitysColumn {
    fn default() -> Self {
        Self {
            filekind: true,
            name: true,
            extansion: true,
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
