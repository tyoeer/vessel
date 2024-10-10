use std::sync::Arc;

use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;

use super::VesselPos;



#[derive(Resource)]
pub struct Graphics {
	pub material: Handle<StandardMaterial>,
	pub mesh: Handle<Mesh>,
}

impl FromWorld for Graphics {
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


#[derive(Resource, Default)]
pub struct Catalogue {
	pub elements: Vec<Arc<Element>>,
}


///Object type
pub struct Element {
	pub graphics: Graphics,
}

pub type ElemRef = Arc<Element>;


pub fn create_event_handler(
	mut objs: EventReader<event::Create>,
	mut cmd: Commands,
) {
	for obj_ev in objs.read() {
		let event::Create {
			pos,
			element,
		} = obj_ev;
		let object_pos = pos.0;
		let object_size = IVec3::new(1,1,1);
		
		let pos = object_pos.as_vec3();
		let offset = object_size.as_vec3() / 2.;
		let transform = Transform::from_translation(pos + offset);
		
		cmd.spawn(PbrBundle {
			mesh: element.graphics.mesh.clone(),
			material: element.graphics.material.clone(),
			transform,
			..default()
		})
		.insert(VesselPos::from(object_pos))
		.insert(PickableBundle::default());
	}
}


pub mod event {
	use bevy::prelude::*;
	use super::super::VesselPos;
	use super::*;
	
	
	#[derive(Event)]
	pub struct Create {
		pub pos: VesselPos,
		pub element: Arc<Element>,
	}
}