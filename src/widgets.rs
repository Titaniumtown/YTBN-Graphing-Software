pub fn widgets_ontop<R>(
	ui: &egui::Ui, id: String, re: &egui::Response, y_offset: f32,
	add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
	let area = egui::Area::new(id)
		.fixed_pos(re.rect.min.offset_y(y_offset))
		.order(egui::Order::Foreground);

	area.show(ui.ctx(), |ui| add_contents(ui)).inner
}
