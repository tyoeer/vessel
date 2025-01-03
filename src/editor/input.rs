use bevy::{input::mouse::MouseMotion, prelude::*};

use super::*;


const SENSITIVITY: f32 = 0.005;
const MOVE_SPEED: f32 = 12.;

pub fn move_camera(
	mut camera_transforms: Query<&mut Transform, With<Camera>>,
	mouse_buttons: Res<ButtonInput<MouseButton>>,
	key_codes: Res<ButtonInput<KeyCode>>,
	mut mouse_motion_events: EventReader<MouseMotion>,
	timer: Res<Time>,
	// mut gizmos: Gizmos,
) {
	if !mouse_buttons.pressed(MouseButton::Right) {return}
	
	for mut tf in &mut camera_transforms {		
		for ev in mouse_motion_events.read() {
			tf.rotate_y(-ev.delta.x * SENSITIVITY);
			tf.rotate_local_x(-ev.delta.y * SENSITIVITY);
		}
		//make sure Y stays up
		let forward = tf.forward();
		tf.look_to(forward, Dir3::Y);
		
		//camera: x+ = right, y+ = up, z+ = back
		let mut local_offset = Vec3::ZERO;
		if key_codes.pressed(KeyCode::KeyW) {
			local_offset.z -= 1.;
		}
		if key_codes.pressed(KeyCode::KeyS) {
			local_offset.z += 1.;
		}
		if key_codes.pressed(KeyCode::KeyE) {
			local_offset.y += 1.;
		}
		if key_codes.pressed(KeyCode::KeyQ) {
			local_offset.y -= 1.;
		}
		if key_codes.pressed(KeyCode::KeyD) {
			local_offset.x += 1.;
		}
		if key_codes.pressed(KeyCode::KeyA) {
			local_offset.x -= 1.;
		}
		
		let offset = tf.rotation.mul_vec3(local_offset * MOVE_SPEED * timer.delta_secs());
		tf.translation += offset;
	}
}


pub fn click_handler(
	mut clicks: EventReader<Pointer<Click>>,
	pos: Query<&object::Pos>,
	mut create: EventWriter<object::event::Create>,
	selem: Res<Hand>,
) {
	for click in clicks.read() {
		let ent = click.target;
		let Ok(old_pos) = pos.get(ent) else {continue};
		let Some(hit_normal) = click.hit.normal else {continue};
		let offset = hit_normal.as_ivec3();
		if offset == IVec3::ZERO {continue}
		let pos = old_pos.0 + offset;
		
		create.send(object::event::Create {
			pos: pos.into(),
			element: selem.0.clone(),
		});
	}
}


pub fn setup_camera(
	mut cmds: Commands,
	root: Res<EditorRoot>,
) {
	cmds.spawn((
		Camera3d::default(),
		PerspectiveProjection {
			fov: 80.,
			..default()
		},
		Transform::default().looking_to(Vec3::X, Vec3::Y),
	)).set_parent(root.0);
}
