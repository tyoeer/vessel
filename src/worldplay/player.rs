use avian3d::prelude::*;
use bevy::prelude::*;
use super::*;


#[derive(Resource, Clone)]
pub struct RtVesselData {
	pub vessel_info: VesselProperties,
	pub graphics: Vec<VesselGraphicPart>,
}


#[derive(Clone)]
pub struct VesselGraphicPart {
	pub mesh: Handle<Mesh>,
	pub material: Handle<StandardMaterial>,
	pub transform: Transform,
}


///Behaviour of a vessel
#[derive(Component, Clone)]
pub struct VesselProperties {
	///How much forwards force to apply when the input is fully forwards
	pub control_forwards_force: f32,
	///How much torque to apply for horizontal turning when the input is fully left or right.
	pub control_torque: f32,
	///Fraction/ratio of the sideways (left/right) speed to apply as counter-force to reduce sideways speed.
	pub side_friction: f32,
	///Fraction/ratio of the horizontal rotation speed to apply as counter-torque to reduce rotary speed.
	pub rotary_friction_hor: f32,
	///Fraction/ratio of the non-horizontal rotation speed to apply as counter-torque to reduce rotary speed.
	pub rotary_friction_ver: f32,
	
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


#[derive(Event)]
pub struct SpawnEvent {
	pub rt_vessel_data: RtVesselData,
}


pub fn spawn_players(
	mut cmds: Commands,
	root: Res<GameplayRoot>,
	mut spawn_events: EventReader<SpawnEvent>,
) {
	for event in spawn_events.read() {
		let player_data = &event.rt_vessel_data;
		let player = cmds.spawn(player_data.vessel_info.clone())
			.insert(Control::default())
			.insert(SpatialBundle::default())
			.insert(RigidBody::Dynamic)
			.insert(Collider::cuboid(1., 1., 1.))
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


#[derive(Event, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Control(pub Vec2);


pub fn read_player_input(
	buttons: Res<ButtonInput<KeyCode>>,
	mut players: Query<&mut Control>,
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


pub fn move_player(
	mut players: Query<(
		&Control,
		&VesselProperties,
		&Transform,
		&mut ExternalForce,
		&mut ExternalTorque,
		&LinearVelocity,
		&AngularVelocity
	)>,
) {
	for (control, vessel, tf, mut force, mut torque, vel, rot_vel,) in &mut players {
		let control = control.0;
		force.persistent = false;
		torque.persistent = false;
		
		// extra frictions
		
		let local_vel = tf.rotation.inverse().mul_vec3(vel.0);
		let side_friction = -local_vel.z * vessel.side_friction;
		let friction = Vec3::new(0., 0., side_friction);
		force.apply_force(tf.rotation * friction);
		
		let ver_rot = rot_vel.0.with_y(0.);
		let hor_rot = rot_vel.0.with_x(0.).with_z(0.);
		torque.apply_torque(-hor_rot * vessel.rotary_friction_hor);
		torque.apply_torque(-ver_rot * vessel.rotary_friction_ver);
		
		// player control
		
		let accel_dir_world_space = tf.rotation.mul_vec3(Vec3::X);
		//Remove flying capabilities
		let accel_dir_world_space = accel_dir_world_space.with_y(0.).normalize();
		let accel_force = vessel.control_forwards_force * control.y;
		
		force.apply_force(accel_dir_world_space * accel_force);
		
		torque.apply_torque(Quat::from_rotation_y(control.x * -vessel.control_torque).to_scaled_axis());
	}
}