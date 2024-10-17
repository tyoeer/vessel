use bevy::prelude::*;

use super::*;


pub fn hotbar_ui( 
	mut contexts: bevy_egui::EguiContexts,
	catalogue: Res<object::Catalogue>,
	mut selected: ResMut<SelectedElement>,
) {
	use bevy_egui::egui;
	let Some(ctx) = contexts.try_ctx_mut() else {
		// Primary window is missing, because it still is being initialized or has been closed
		// This system can still run in those conditions, so just do nothing until other systems fix it
		return;
	};
	
	egui::Window::new("Hotbar").resizable(true).default_height(50.).show(ctx, |ui| {
		ui.with_layout(egui::Layout {
			main_dir: egui::Direction::LeftToRight,
			main_wrap: false,
			main_align: egui::Align::Center,
			main_justify: false,
			cross_align: egui::Align::Center,
			cross_justify: false,
		}, |ui| {
			for (i, elem) in catalogue.elements.iter().enumerate() {
				let button = egui::Button::new(i.to_string())
					.min_size((40.,40.).into());
				let button_res = ui.add(button);
				
				if button_res.clicked() {
					*selected = SelectedElement(elem.clone());
				}
			}
		})
	});
}


pub fn setup_lights(
	mut cmds: Commands,
) {
	cmds.insert_resource(ClearColor(Color::srgb(0.6, 0.7, 1.)));
	cmds.insert_resource(AmbientLight {
		color: bevy::color::palettes::css::WHITE.into(),
		brightness: 600.,
	});
	
	cmds.spawn(DirectionalLightBundle {
		directional_light: DirectionalLight {
			illuminance: light_consts::lux::OVERCAST_DAY / 2.,
			shadows_enabled: false,
			..default()
		},
		transform: Transform {
			translation: Vec3::new(1.0, 4.0, 2.0),
			..default()
		}.looking_at(Vec3::ZERO, Dir3::Y),
		..default()
	});
	//counter light to differentiate the shadows
	cmds.spawn(DirectionalLightBundle {
		directional_light: DirectionalLight {
			illuminance: light_consts::lux::OVERCAST_DAY / 7.,
			shadows_enabled: false,
			..default()
		},
		transform: Transform {
			translation: Vec3::new(-6.0, -1.0, -3.),
			..default()
		}.looking_at(Vec3::ZERO, Dir3::Y),
		..default()
	});
}