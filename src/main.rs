use bevy::{input::mouse::{MouseButtonInput, MouseWheel}, prelude::*};
use bevy_mod_picking::debug::DebugPickingMode;


mod editor;


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
	.add_plugins(editor::VesselPlugin)
	// Look
	.insert_resource(ClearColor(Color::srgb(0.6, 0.7, 1.)))
	.insert_resource(AmbientLight {
		color: bevy::color::palettes::css::WHITE.into(),
		brightness: 600.,
	})
	
	// Example graphics
	.add_systems(Startup, setup_example_graphics)
	.add_systems(Startup, add_camera)
	.add_systems(Startup, setup_ui_style)
	
	.run();
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

fn add_camera(
	mut cmds: Commands,
) {
	cmds.spawn(Camera3dBundle {
		projection: PerspectiveProjection {
			fov: 80.,
			..default()
		}.into(),
		transform: Transform::default().looking_to(Vec3::X, Vec3::Y),
		..default()
	});
}

fn setup_example_graphics(
	mut commands: Commands,
) {
	let cam_t = Vec3::new(1.0, 4.0, 2.0);
	commands.spawn(DirectionalLightBundle {
		directional_light: DirectionalLight {
			illuminance: light_consts::lux::OVERCAST_DAY / 2.,
			shadows_enabled: false,
			..default()
		},
		transform: Transform {
			translation: cam_t,
			..default()
		}.looking_at(Vec3::ZERO, Dir3::Y),
		..default()
	});
	//counter light to differentiate the shadows
	commands.spawn(DirectionalLightBundle {
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