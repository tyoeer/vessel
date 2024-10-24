use bevy::{input::mouse::{MouseButtonInput, MouseWheel}, prelude::*};
use bevy_mod_picking::debug::DebugPickingMode;


mod editor;
mod worldplay;


fn main() {
	App::new()
	
	// Init
	.add_plugins(DefaultPlugins.set(WindowPlugin {
		primary_window: Some(Window {
			//gets rid of input lag
			present_mode: bevy::window::PresentMode::AutoNoVsync,
			..default()
		}),
		..default()
	}))
	.add_plugins(bevy_egui::EguiPlugin)
	.add_systems(
		PreUpdate,
		absorb_egui_inputs
		.after(bevy_egui::systems::process_input_system)
		.before(bevy_egui::EguiSet::BeginPass)
	)
	.add_plugins(bevy_mod_picking::DefaultPickingPlugins)
	.insert_resource(DebugPickingMode::Normal)
	
	.insert_state(GameState::EditVessel)
	.add_plugins(editor::VesselPlugin {
		state: GameState::EditVessel
	})
	.add_plugins(worldplay::GameplayPlugin {
		state: GameState::WorldPlay
	})
	
	.add_systems(Startup, setup_ui_style)
	.add_systems(Startup, editor::setup_catalogue)
	.add_systems(Update, state_ui)
	
	.run();
}


#[derive(States, Debug,Clone, PartialEq, Eq, Hash)]
pub enum GameState {
	WorldPlay,
	EditVessel,
}


fn state_ui(
	mut contexts: bevy_egui::EguiContexts,
	mut next_state: ResMut<NextState<GameState>>,
) {
	use bevy_egui::egui;
	let Some(ctx) = contexts.try_ctx_mut() else {
		// Primary window is missing, because it still is being initialized or has been closed
		// This system can still run in those conditions, so just do nothing until other systems fix it
		return;
	};

	egui::Window::new("States").resizable(true).show(ctx, |ui| {
		if ui.button("Edit").clicked() {
			next_state.set(GameState::EditVessel);
		}
		if ui.button("Play").clicked() {
			next_state.set(GameState::WorldPlay);
		}
	});
}


fn setup_ui_style(
	mut contexts: bevy_egui::EguiContexts,
) {
	use bevy_egui::egui;
	let Some(ctx) = contexts.try_ctx_mut() else {
		// Primary window is missing, because it still is being initialized or has been closed
		// This system can still run in those conditions, so just do nothing until other systems fix it
		return;
	};
	
	ctx.style_mut(|style| {
		style.visuals.window_shadow = egui::Shadow::NONE;
	});
}

///Prevents inputs that egui is using from affecting the rest of the game
// Based on https://github.com/mvlabat/bevy_egui/issues/47#issuecomment-2368811068
fn absorb_egui_inputs(
	mut contexts: bevy_egui::EguiContexts,
	mut mouse: ResMut<ButtonInput<MouseButton>>,
	mut mouse_wheel: ResMut<Events<MouseWheel>>,
	mut mouse_button_events: ResMut<Events<MouseButtonInput>>,
	mut keyboard: ResMut<ButtonInput<KeyCode>>,
	mut picking_settings: ResMut<bevy_mod_picking::input::InputPluginSettings>
) {
	//bevy_mod_picking runs too early, so we have to disable it some other way
	picking_settings.is_mouse_enabled = true;
	
	let Some(ctx) = contexts.try_ctx_mut() else {
		//Bevy is slow exiting after the window has been closed
		// So this still runs while there's no context anymore
		return;
	};
	if !ctx.wants_pointer_input() && !ctx.is_pointer_over_area() {
		return;
	}
	
	picking_settings.is_mouse_enabled = false;
	
	let modifiers = [
		KeyCode::SuperLeft,
		KeyCode::SuperRight,
		KeyCode::ControlLeft,
		KeyCode::ControlRight,
		KeyCode::AltLeft,
		KeyCode::AltRight,
		KeyCode::ShiftLeft,
		KeyCode::ShiftRight,
	];
	
	let pressed = modifiers.map(|key| keyboard.pressed(key).then_some(key));
	
	mouse.reset_all();
	mouse_wheel.clear();
	mouse_button_events.clear();
	keyboard.reset_all();
	
	for key in pressed.into_iter().flatten() {
		keyboard.press(key);
	}
}