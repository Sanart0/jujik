use crate::{commands::Command, error::JujikError, pin::Pin, tab::Tab};
use eframe::{App, EventLoopBuilderHook, NativeOptions, run_native};
use egui::{CentralPanel, ScrollArea, SidePanel, TopBottomPanel, Ui, menu};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::{
    sync::mpsc::Receiver,
    thread::{self, JoinHandle},
};
use winit::platform::wayland::EventLoopBuilderExtWayland;

pub struct JujikView {
    controller: Sender<Command>,
    view: Receiver<Command>,
    pins: Vec<Pin>,
    tabs: Vec<Tab>,
    current_tab: String,
}

impl JujikView {
    pub fn new(controller: Sender<Command>, view: Receiver<Command>) -> Self {
        Self {
            controller,
            view,
            pins: Vec::new(),
            tabs: Vec::new(),
            current_tab: String::new(),
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
            println!("View: {:?}", command);

            match command {
                Command::ShowPin(pin) => self.pins.push(pin),
                Command::Drop => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                Command::ShowTab(tab) => self.tabs.push(tab),
                _ => {}
            }
        };
        Ok(())
    }

    fn pin(&self, ui: &mut Ui) {
        ui.separator();
        ui.label("Pin");
        ui.separator();

        for pin in &self.pins {
            if ui.button(pin.get_name()).clicked() {
                let _ = self
                    .controller
                    .send(Command::NewTab(pin.get_path()))
                    .inspect_err(JujikError::handle_err);
            }
        }
    }

    fn mount(&self, ui: &mut Ui) {
        ui.separator();
        ui.label("Mount");
        ui.separator();
    }
}

impl App for JujikView {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("menu").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                if ui.button("Create Root").clicked() {
                    let _ = self
                        .controller
                        .send(Command::NewPin(PathBuf::from("/")))
                        .inspect_err(JujikError::handle_err);
                }
                if ui.button("Create Home").clicked() {
                    let _ = self
                        .controller
                        .send(Command::NewPin(PathBuf::from("/home/sanart0/")))
                        .inspect_err(JujikError::handle_err);
                }
            });
        });

        SidePanel::left("Bind")
            .width_range(100.0..=300.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let _ = ui.button("H");
                    let _ = ui.button("J");
                    let _ = ui.button("K");
                    let _ = ui.button("L");
                });

                self.pin(ui);
                self.mount(ui);
            });

        CentralPanel::default().show(ctx, |ui| {
            TopBottomPanel::top("tab").show_inside(ui, |ui| {
                let mut scroll = ui.ctx().style().spacing.scroll;
                scroll.floating = false;
                scroll.bar_width = 4.0;
                scroll.bar_inner_margin = 4.0;
                scroll.bar_outer_margin = 4.0;
                scroll.foreground_color = false;
                ui.ctx().all_styles_mut(|s| s.spacing.scroll = scroll);

                ui.horizontal(|ui| {
                    ScrollArea::horizontal().show(ui, |ui| {
                        for tab in &self.tabs {
                            if ui
                                .selectable_label(
                                    self.current_tab.eq(&tab.get_name()),
                                    tab.get_name(),
                                )
                                .clicked()
                            {
                                self.current_tab = tab.get_name();
                            }
                        }
                    });
                });
            });
            CentralPanel::default().show_inside(ui, |ui| {
                // ui.label(format!("{:?}", self.current_tab));

                // for (idx, tab) in self.tabs.iter_mut().enumerate() {
                //     tab.filesystem.read_path();
                // }
            });
        });

        let _ = self.handle_commad(ctx).inspect_err(JujikError::handle_err);

        //TODO meybe do not need
        ctx.request_repaint();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = self
            .controller
            .send(Command::Drop)
            .inspect_err(JujikError::handle_err);
    }
}
