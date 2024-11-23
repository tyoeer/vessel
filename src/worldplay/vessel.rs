use avian3d::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::editor::element::Catalogue;

use super::*;


///Serializable form of a vessel meant to be played
#[derive(Asset, Reflect, Debug, Clone, Serialize, Deserialize)]
pub struct SimVessel {
	/// list of (element id, where to place it)
	pub graphics: Vec<(String, Transform)>,
	#[reflect(ignore)]
	pub collider: Collider,
	pub physics_properties: VesselProperties,
}


/**
Unique reference to a vessel.
When inserted on an entity, will automatically cause the corresponding vessel to be spawned/attached to the entity.
*/
#[derive(Component, From, Into, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Id(pub uuid::Uuid);


///A marker [Component] that says this entity has a vessel spawned from it's attached [Id]
#[derive(Component)]
pub struct VesselSpawned;


///Physical behaviour of a vessel
#[derive(Component, Reflect, Clone, Debug, Serialize, Deserialize)]
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


pub fn spawn_vessels(
	mut cmds: Commands,
	root: Res<GameplayRoot>,
	todo: Query<(Entity, &Id), Without<VesselSpawned>>,
	vessels: Res<Assets<SimVessel>>,
	elements: Res<Catalogue>,
) {
	for (entity, id) in &todo {
		let Some(vessel) = vessels.get(id.0) else {
			warn!("no vessel with asked for id {}", id.0);
			continue;
		};
		
		debug!(vessel=?id, ?entity, "spawned vessel");
		
		let player = cmds.entity(entity)
			.insert(VesselSpawned)
			.insert(vessel.physics_properties.clone())
			.insert(vessel.collider.clone()) // in avian3d 0.1.2 this uses an Arc under the hood so is actually rather cheap
			.insert(Control::default())
			.insert(SpatialBundle::default())
			.insert(RigidBody::Dynamic)
			.insert(Friction::new(0.)) // extra friction is provided by the race track itself
			.set_parent(root.0)
			.id();
		
		for (elem_id, transform) in &vessel.graphics {
			let elem = elements.find_by_id(elem_id);
			cmds.spawn(PbrBundle {
				mesh: elem.graphics.mesh.clone(),
				material: elem.graphics.material.clone(),
				transform: *transform,
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