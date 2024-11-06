use avian3d::prelude::*;
use bevy::prelude::*;
use super::*;


///Everything required for a vessel at runtime
#[derive(Resource, Clone, Default)]
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


///Physical behaviour of a vessel
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

impl Default for VesselProperties {
	fn default() -> Self {
		Self {
			control_forwards_force: 8.,
			control_torque: 6.,
			side_friction: 2.2,
			rotary_friction_hor: 3.,
			rotary_friction_ver: 6.,
		}
	}
}



#[derive(Event)]
pub struct SpawnEvent {
	pub rt_vessel_data: RtVesselData,
	///Already existing entity to spawn the player into, used for multiplayer
	pub player_entity: Option<Entity>,
}


pub fn spawn_vessels(
	mut cmds: Commands,
	root: Res<GameplayRoot>,
	mut spawn_events: EventReader<SpawnEvent>,
) {
	for event in spawn_events.read() {
		let player_data = &event.rt_vessel_data;
		
		let mut player_cmds = match event.player_entity {
			None => cmds.spawn_empty(),
			Some(entity) => cmds.entity(entity),
		};
		
		let player = player_cmds
			.insert(player_data.vessel_info.clone())
			.insert(Control::default())
			.insert(SpatialBundle::default())
			.insert(RigidBody::Dynamic)
			.insert(Friction::new(0.)) // extra friction is provided by the race track itself
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
	}
}


///The inputs to control a vessel with
#[derive(Event, Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Control(pub Vec2);


///Handles [Control] (input) and friction
pub fn move_vessel(
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
		
		force.persistent = true;
		force.clear();
		torque.persistent = true;
		torque.clear();
		
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