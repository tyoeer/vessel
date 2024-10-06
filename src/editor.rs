use bevy::{
	input::{
		common_conditions::input_just_pressed,
		mouse::MouseMotion,
	},
	prelude::*
};
use bevy_mod_picking::{
	events::{Click, Pointer},
	PickableBundle,
};
use derive_more::{From, Into};

pub struct VesselPlugin;

impl Plugin for VesselPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_event::<CreateObjectEvent>()
			.init_resource::<ObjectGraphics>()
		;
		app
			.add_systems(Update, (
				create_test_obj
					.run_if(input_just_pressed(KeyCode::Enter)),
				click_handler,
				add_objects,
				
				camera,
			))
		;	
	}
}

const SENSITIVITY: f32 = 0.005;
const MOVE_SPEED: f32 = 12.;

pub fn camera(
	mut camera_transforms: Query<&mut Transform, With<Camera>>,
	mouse_buttons: Res<ButtonInput<MouseButton>>,
	key_codes: Res<ButtonInput<KeyCode>>,
	mut mouse_motion_events: EventReader<MouseMotion>,
	timer: Res<Time>,
	// mut gizmos: Gizmos,
) {
	if !mouse_buttons.pressed(MouseButton::Right) {return}
	
	for mut tf in &mut camera_transforms {
		
		//The camera's mapping from world space to window/viewport space is weird
		// and the documentation appears to be wrong
		// The correctness of the following code has been determined empirically by testing
		
		for ev in mouse_motion_events.read() {
			tf.rotate_y(-ev.delta.x * SENSITIVITY);
			tf.rotate_local_x(ev.delta.y * SENSITIVITY);
		}
		//make sure Y stays up
		let forward = tf.forward();
		//camera y goes down, so we need to flip it to make world Y render upwards
		tf.look_to(forward, -Dir3::Y);
		
		//camera: x+ = left, y+ = down, z+ = back
		let mut local_offset = Vec3::ZERO;
		if key_codes.pressed(KeyCode::KeyW) {
			local_offset.z -= 1.;
		}
		if key_codes.pressed(KeyCode::KeyS) {
			local_offset.z += 1.;
		}
		if key_codes.pressed(KeyCode::KeyE) {
			local_offset.y -= 1.;
		}
		if key_codes.pressed(KeyCode::KeyQ) {
			local_offset.y += 1.;
		}
		if key_codes.pressed(KeyCode::KeyD) {
			local_offset.x -= 1.;
		}
		if key_codes.pressed(KeyCode::KeyA) {
			local_offset.x += 1.;
		}
		
		let offset = tf.rotation.mul_vec3(local_offset * MOVE_SPEED * timer.delta_seconds());
		tf.translation += offset;
		
		// const SIZE: f32 = 0.3;
		// use bevy::color::palettes::css::{BLACK, BLUE, GREEN, RED};
		
		// let pos = tf.translation + tf.forward()*5.;
		// if kcs.pressed(KeyCode::KeyR) {
		// 	gizmos.line(pos, pos+tf.local_x()*SIZE, RED);
		// 	gizmos.line(pos, pos+tf.local_y()*SIZE, GREEN);
		// 	gizmos.line(pos, pos+tf.local_z()*SIZE, BLUE);
		// } else {
		// 	gizmos.line(pos, pos+Vec3::X*SIZE, RED);
		// 	gizmos.line(pos, pos+Vec3::Y*SIZE, GREEN);
		// 	gizmos.line(pos, pos+Vec3::Z*SIZE, BLUE);
		// }
		// gizmos.sphere(Vec3::ZERO, Quat::default(), 1., BLACK);
	}
}

#[derive(Resource)]
pub struct ObjectGraphics {
	pub material: Handle<StandardMaterial>,
	pub mesh: Handle<Mesh>,
}

impl FromWorld for ObjectGraphics {
	fn from_world(world: &mut World) -> Self {
		let mut meshes = world.get_resource_mut::<Assets<Mesh>>().expect("bevy world should have Assets<Mesh>");
		let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
		
		let mut materials = world.get_resource_mut::<Assets<StandardMaterial>>().expect("bevy world should have Assets<StandardMaterial>");
		let material = materials.add(StandardMaterial {
			base_color: Color::srgb(0.9, 0.85, 0.8),
			perceptual_roughness: 0.9,
			..default()
		});
		
		Self {
			mesh,
			material,
		}
	}
}

#[derive(Component, From, Into)]
pub struct VesselPos(pub IVec3);

#[derive(Event)]
pub struct CreateObjectEvent {
	pub pos: VesselPos,
}

fn create_test_obj(
	mut oe: EventWriter<CreateObjectEvent>,
) {
	oe.send(CreateObjectEvent {
		pos: IVec3::new(0, 0, 0).into()
	});
}

fn click_handler(
	mut clicks: EventReader<Pointer<Click>>,
	pos: Query<&VesselPos>,
	mut create: EventWriter<CreateObjectEvent>,
) {
	for click in clicks.read() {
		let ent = click.target;
		let Ok(old_pos) = pos.get(ent) else {continue};
		let Some(hit_normal) = click.hit.normal else {continue};
		let offset = hit_normal.as_ivec3();
		if offset == IVec3::ZERO {continue}
		let pos = old_pos.0 + offset;
		
		create.send(CreateObjectEvent {
			pos: pos.into()
		});
	}
}

fn add_objects(
	mut objs: EventReader<CreateObjectEvent>,
	mut cmd: Commands,
	object_graphics: Res<ObjectGraphics>,
) {
	for obj_ev in objs.read() {
		let object_pos = obj_ev.pos.0;
		let object_size = IVec3::new(1,1,1);
		
		let pos = object_pos.as_vec3();
		let offset = object_size.as_vec3() / 2.;
		let transform = Transform::from_translation(pos + offset);
		
		cmd.spawn(PbrBundle {
			mesh: object_graphics.mesh.clone(),
			material: object_graphics.material.clone(),
			transform,
			..default()
		})
		.insert(VesselPos::from(object_pos))
		.insert(PickableBundle::default());
	}
}