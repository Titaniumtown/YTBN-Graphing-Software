use crate::misc::Offset;
use egui::{Id, InnerResponse};

/// Creates an area ontop of a widget with an y offset
pub fn widgets_ontop<R>(
    ui: &egui::Ui,
    id: Id,
    re: &egui::Response,
    y_offset: f32,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> InnerResponse<R> {
    let area = egui::Area::new(id)
        .fixed_pos(re.rect.min.offset_y(y_offset))
        .order(egui::Order::Foreground);

    area.show(ui.ctx(), |ui| add_contents(ui))
}
