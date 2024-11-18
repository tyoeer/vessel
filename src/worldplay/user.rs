/*!

Things for the local player.

*/

use bevy::prelude::*;
use super::*;


///The [vessel::Id] of the current user, as a [Resource]
#[derive(Resource, From, Into, Clone)]
pub struct UserVesselId(pub vessel::Id);


///A player thats is controlled by this client/user
#[derive(Component, Clone, Copy)]
pub struct LocallyControlled;



pub fn read_user_input(
	buttons: Res<ButtonInput<KeyCode>>,
	mut players: Query<&mut vessel::Control, With<LocallyControlled>>,
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
	
	for mut control in &mut players {
		let old = control.bypass_change_detection().0;
		if move_dir != old {
			control.0 = move_dir;
		}
	}
}



pub fn spawn_user(
	mut cmds: Commands,
	user_vessel_id: Res<UserVesselId>,
	player_data: Res<vessel::RtVesselData>,
	mut spawn_events: EventWriter<vessel::SpawnEvent>,
) {
	let id = cmds.spawn((
		LocallyControlled,
		user_vessel_id.0
	)).id();
	
	cmds.spawn(Camera3dBundle {
		transform: Transform::from_xyz(0., 0., 0.)
			.looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	}).set_parent(id);
	
	spawn_events.send(vessel::SpawnEvent {
		rt_vessel_data: player_data.clone(),
		player_entity: Some(id),
	});
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
