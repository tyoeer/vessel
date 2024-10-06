use bevy::prelude::*;
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
	
	.run();
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