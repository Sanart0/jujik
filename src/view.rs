use crate::{error::CustomError, pin::Pin, tab::Tab};
use eframe::{App, NativeOptions, run_native};
use egui::{CentralPanel, ScrollArea, SidePanel, TopBottomPanel, menu};
use std::thread::{self, JoinHandle};

#[derive(Default)]
pub struct JujikView {
    pins: Vec<Pin>,
    tabs: Vec<Tab>,
}

impl JujikView {
    pub fn new() -> Self {
        Self {
            pins: Vec::new(),
            tabs: Vec::new(),
        }
    }

    pub fn run(self) -> JoinHandle<Result<(), CustomError>> {
        thread::spawn(|| -> Result<(), CustomError> {
            let native_options = NativeOptions::default();
            run_native("Jujik", native_options, Box::new(|_cc| Ok(Box::new(self))))?;
            Ok(())
        })
    }
}

impl App for JujikView {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        TopBottomPanel::top("menu").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
            });
        });

        SidePanel::left("Bind")
            .width_range(100.0..=300.0)
            .show(ctx, |ui| {
                TopBottomPanel::top("line").show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        let _ = ui.button("H");
                        let _ = ui.button("J");
                        let _ = ui.button("K");
                        let _ = ui.button("L");
                    });
                });
                CentralPanel::default().show(ctx, |ui| {
                    ui.separator();
                    ui.label("Pin");
                    ui.separator();

                    // for pin in &self.pins {
                    //     if let Some(name) = pin.name() {
                    //         if ui.button(name).clicked() {
                    //             if let Some(path) = pin.path() {
                    //                 self.tabs.push(Tab::new(&path));
                    //             }
                    //         }
                    //     }
                    // }

                    ui.separator();
                    ui.label("Mount");
                    ui.separator();
                });
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
                        // for (idx, tab) in self.tabs.iter().enumerate() {
                        //     if let Some(name) = tab.name() {
                        //         if ui
                        //             .selectable_label(self.current_tab.eq(&idx), name)
                        //             .clicked()
                        //         {
                        //             self.current_tab = idx;
                        //         }
                        //     }
                        // }
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

        ctx.request_repaint();
    }
}
