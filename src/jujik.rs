pub mod model;
pub mod view;

use eframe::App;
use view::JujikView;

pub struct Jujik {
    view: JujikView,
}

impl Jujik {
    pub fn new() -> Self {
        Self {
            view: JujikView::default(),
        }
    }
}

impl App for Jujik {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.view.update(ctx, frame);
    }
}
