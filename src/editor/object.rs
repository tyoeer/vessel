/*!
Everything to do with objects and elements.

A creation consists of objects that are placed by the user.
THe type of an object is it's element.
*/

use std::sync::Arc;

use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;
use derive_more::derive::{From, Into};

use super::EditorRoot;


///Position of an object within a creation
#[derive(Component, Clone, From, Into)]
pub struct Pos(pub IVec3);


///Object info separate from ECS
pub struct Object {
	pub element: ElemRef,
	pub pos: Pos,
}


pub struct Graphics {
	pub material: Handle<StandardMaterial>,
	pub mesh: Handle<Mesh>,
}


///List of all the available elements
#[derive(Resource, Default)]
pub struct Catalogue {
	pub elements: Vec<Arc<Element>>,
}


///Object type
pub struct Element {
	pub graphics: Graphics,
}

pub type ElemRef = Arc<Element>;

#[derive(Component, Into, From)]
pub struct ElementComponent(pub ElemRef);

///Creates objects when [event::Create] happen
pub fn create_event_handler(
	mut objs: EventReader<event::Create>,
	root: Res<EditorRoot>,
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
		.set_parent(root.0)
		.insert(Pos::from(object_pos))
		.insert(ElementComponent::from(element.clone()))
		.insert(PickableBundle::default());
	}
}


pub mod event {
	use bevy::prelude::*;
	use super::*;
	
	
	///CReates a new object
	#[derive(Event)]
	pub struct Create {
		pub pos: Pos,
		pub element: ElemRef,
	}
}