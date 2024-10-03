use bevy::prelude::*;


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
	.add_plugins(editor::VesselPlugin)
	// Look
	.insert_resource(ClearColor(Color::srgb(0.6, 0.7, 1.)))
	.insert_resource(AmbientLight {
		color: bevy::color::palettes::css::WHITE.into(),
		brightness: 600.,
	})
	
	// Fly camera
	.add_plugins(bevy_flycam::PlayerPlugin)
	.insert_resource(bevy_flycam::MovementSettings {
		sensitivity: 0.0003, // default: 0.00012
		speed: 12.0, // default: 12.0
	})
	.insert_resource(bevy_flycam::KeyBindings {
		move_ascend: KeyCode::KeyE,
		move_descend: KeyCode::KeyQ,
		..default()
	})
	
	// Example graphics
	.add_systems(Startup, setup_example_graphics)
	
	.run();
}

fn setup_example_graphics(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	let mat = materials.add(
		StandardMaterial {
			base_color: Color::srgb(0.9, 0.85, 0.8),
			perceptual_roughness: 0.9,
			..default()
		}
	);
	// Ground
	// commands.spawn(PbrBundle {
	// 	mesh: meshes.add(Plane3d::default().mesh().size(5.0, 5.0)),
	// 	material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
	// 	..default()
	// });
	// Cube
	commands.spawn(PbrBundle {
		mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
		material: mat,
		transform: Transform::from_xyz(0.0, 0.5, 0.0),
		..default()
	});
	
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