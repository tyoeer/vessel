/*!
Turns an editor vessel into something playable.


Vessel pipeline:
- Editor edits data in the ecs
- Gets put into a [crate::editor::misc::StoredVessel]
- This module turns it into a [crate::worldplay::player::FullPlayerData]

*/

use bevy::prelude::{
	Transform,
	Res,
	Commands,
};

use crate::{
	editor::misc::CreationData,
	worldplay::vessel::{
		RtVesselData,
		VesselGraphicPart,
		VesselProperties,
	}
};


pub fn build_vessel_system(
	sv: Res<CreationData>,
	mut cmds: Commands,
) {
	cmds.insert_resource(build_vessel(&sv));
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

