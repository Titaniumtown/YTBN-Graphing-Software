use crate::function_entry::{FunctionEntry, DEFAULT_FUNCTION_ENTRY};
use uuid::Uuid;

pub struct Manager {
	functions: Vec<(Uuid, FunctionEntry)>,
}

impl Default for Manager {
	fn default() -> Self {
		Self {
			functions: vec![(Uuid::new_v4(), DEFAULT_FUNCTION_ENTRY.clone())],
		}
	}
}

impl Manager {
	pub fn display_entries(&mut self, ui: &mut egui::Ui) {
		// ui.label("Functions:");
		let can_remove = self.functions.len() > 1;

		let mut remove_i: Option<usize> = None;
		for (i, (uuid, function)) in self.functions.iter_mut().enumerate() {
			// Entry for a function
			if function.function_entry(ui, can_remove, i) {
				remove_i = Some(i);
			}

			function.settings_window(ui.ctx());
		}

		// Remove function if the user requests it
		if let Some(remove_i_unwrap) = remove_i {
			self.functions.remove(remove_i_unwrap);
		}
	}

	pub fn new_function(&mut self) {
		self.functions
			.push((Uuid::new_v4(), DEFAULT_FUNCTION_ENTRY.clone()));
	}

	pub fn any_using_integral(&self) -> bool {
		self.functions
			.iter()
			.filter(|(_, func)| func.integral)
			.count() > 0
	}

	#[inline]
	pub fn len(&self) -> usize { self.functions.len() }

	pub fn get_entries_mut(&mut self) -> &mut Vec<(Uuid, FunctionEntry)> { &mut self.functions }

	pub fn get_entries(&self) -> &Vec<(Uuid, FunctionEntry)> { &self.functions }
}
