/*!
Turns an editor vessel into something playable.


Vessel pipeline:
- Editor edits data in the ecs
- Gets put into a [crate::editor::misc::StoredVessel]
- This module turns it into a [crate::worldplay::player::FullPlayerData]

*/

use avian3d::prelude::Collider;
use bevy::{asset::Assets, math::{Quat, Vec3}, prelude::{
	Commands, Res, ResMut, Transform
}};

use crate::{
	editor::misc::CreationData,
	worldplay::{
		user::UserVesselId,
		vessel::{
			SimVessel, VesselProperties
		}
	}
};


pub fn build_vessel_system(
	creation: Res<CreationData>,
	mut vessels: ResMut<Assets<SimVessel>>,
	mut cmds: Commands,
) {
	let sim = build_sim_vessel(&creation);
	let id = uuid::Uuid::new_v4();
	vessels.insert(id, sim);
	cmds.insert_resource(UserVesselId(id.into()));
}


pub fn build_sim_vessel(
	creation: &CreationData,
) -> SimVessel {
	let mut graphics = Vec::new();
	let mut collider_parts: Vec<(Vec3, Quat, Collider)> = Vec::new();
	
	for object in &creation.objects {
		let pos = object.pos.0;
		let transform = Transform::from_translation(pos.as_vec3());
		
		collider_parts.push((
			pos.as_vec3(),
			Quat::default(),
			object.element.collider.clone(),
		));
		graphics.push((object.element.id.clone(), transform));
	}
	
	SimVessel {
		graphics,
		collider: Collider::compound(collider_parts),
		physics_properties: VesselProperties::default(),
	}
}
