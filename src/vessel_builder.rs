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
	editor::{
		misc::CreationData,
		object::Catalogue,
	},
	worldplay::{user::UserVesselId, vessel::{
		RtVesselData, SimVessel, VesselGraphicPart, VesselProperties
	}}
};


pub fn build_vessel_system(
	creation: Res<CreationData>,
	catalogue: Res<Catalogue>,
	mut vessels: ResMut<Assets<SimVessel>>,
	mut cmds: Commands,
) {
	let sim = build_sim_vessel(&creation);
	let id = uuid::Uuid::new_v4();
	let rt = sim_to_rt(&sim, &catalogue);
	vessels.insert(id, sim);
	cmds.insert_resource(rt);
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


pub fn sim_to_rt(
	sim: &SimVessel,
	catalogue: &Catalogue,
) -> RtVesselData {
	let graphics = sim.graphics.iter()
		.map(|(elem_id, transform)| {
			let elem = catalogue.find_by_id(elem_id);
			VesselGraphicPart {
				mesh: elem.graphics.mesh.clone(),
				material: elem.graphics.material.clone(),
				transform: *transform,
			}
		}).collect();
		
	RtVesselData {
		graphics,
		vessel_info: sim.physics_properties.clone(),
	}
}


pub fn build_vessel(
	sv: &CreationData,
) -> RtVesselData {
	let mut graphics = Vec::new();
	
	for obj in &sv.objects {
		let pos = obj.pos.0;
		let transform = Transform::from_translation(pos.as_vec3());
		
		graphics.push(VesselGraphicPart {
			mesh: obj.element.graphics.mesh.clone(),
			material: obj.element.graphics.material.clone(),
			transform,
		});
	}
	
	RtVesselData {
		vessel_info: VesselProperties::default(),
		graphics,
	}
}

