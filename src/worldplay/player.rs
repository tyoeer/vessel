use bevy::prelude::*;
use super::*;


#[derive(Resource)]
pub struct RtVesselData {
	pub vessel_info: VesselProperties,
	pub graphics: Vec<VesselGraphicPart>,
}


pub struct VesselGraphicPart {
	pub mesh: Handle<Mesh>,
	pub material: Handle<StandardMaterial>,
	pub transform: Transform,
}


///Separate player stuff to put in the ECS
#[derive(Component, Clone)]
pub struct VesselProperties {
	
}


#[derive(Resource)]
pub struct CameraSettings {
	pub up: f32,
	pub back: f32,
	pub pitch: f32,
}

impl Default for CameraSettings {
	fn default() -> Self {
		Self {
			up: 2.,
			back: 4.,
			pitch: 0.,
		}
	}
}


pub fn spawn_player(
	mut cmds: Commands,
	root: Res<GameplayRoot>,
	player_data: Res<RtVesselData>,
) {
	let player = cmds.spawn(player_data.vessel_info.clone()).insert(SpatialBundle::default())
	.set_parent(root.0)
	.id();
	
	for graphic in &player_data.graphics {
		cmds.spawn(PbrBundle {
			mesh: graphic.mesh.clone(),
			material: graphic.material.clone(),
			transform: graphic.transform,
			..default()
		}).set_parent(player);
	}
	
	cmds.spawn(Camera3dBundle {
		transform: Transform::from_xyz(0., 0., 0.)
			.looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	}).set_parent(player);
}


pub fn update_camera(
	mut cams: Query<&mut Transform, With<Camera3d>>,
	camera_settings: Res<CameraSettings>,
) {
	for mut tf in &mut cams {
		tf.translation.y = camera_settings.up;
		tf.translation.x = -camera_settings.back;
		tf.look_to(Vec3::X, Vec3::Y);
		tf.rotate_z(camera_settings.pitch);
	}
}


pub fn camera_ui(
	mut contexts: bevy_egui::EguiContexts,
	mut camera_settings: ResMut<CameraSettings>,
) {
	use bevy_egui::egui;
	let Some(ctx) = contexts.try_ctx_mut() else {
		// Primary window is missing, because it still is being initialized or has been closed
		// This system can still run in those conditions, so just do nothing until other systems fix it
		return;
	};
	
	use core::f32::consts::TAU;
	
	egui::Window::new("Camera").resizable(true).show(ctx, |ui| {
		ui.add(egui::Slider::new(
			&mut camera_settings.up,
			0. ..= 30.
			).text("Up")
		);
		ui.add(egui::Slider::new(
			&mut camera_settings.back,
			0. ..= 30.
			).text("Back")
		);
		ui.add(egui::Slider::new(
			&mut camera_settings.pitch,
			-TAU/4. ..= TAU/8.
			).text("Pitch")
			.smart_aim(false)
			// .step_by((TAU/ 2_f32.powi(10)) as f64)
		);
	});
}


pub fn move_player(
	mut players: Query<(&VesselProperties, &mut Transform)>,
	buttons: Res<ButtonInput<KeyCode>>,
	timer: Res<Time>,
) {
	let mut move_dir = Vec2::ZERO;
	
	if buttons.pressed(KeyCode::KeyW) {
		move_dir += Vec2::Y;
	}
	if buttons.pressed(KeyCode::KeyS) {
		move_dir -= Vec2::Y;
	}
	if buttons.pressed(KeyCode::KeyD) {
		move_dir += Vec2::X;
	}
	if buttons.pressed(KeyCode::KeyA) {
		move_dir -= Vec2::X;
	}
	
	for (_player, mut tf) in &mut players {
		let speed = 10.;
		
		let vel = Vec3::new(move_dir.y, 0., move_dir.x) * speed * timer.delta_seconds();
		tf.translation += vel;
	}
}