/*!
Everything to do with elements.

Elements are the type of objects
*/


use bevy::prelude::*;
use derive_more::derive::{From, Into};
use std::sync::Arc;


pub struct Graphics {
	pub material: Handle<StandardMaterial>,
	pub mesh: Handle<Mesh>,
}


///List of all the available elements
#[derive(Resource, Default)]
pub struct Catalogue {
	pub elements: Vec<Arc<Element>>,
}

impl Catalogue {
	///Panics if the asked for element isn't in this catalogue
	pub fn find_by_id(&self, id: &str) -> Arc<Element> {
		self.elements.iter()
			.find(|elem| elem.id == id)
			.cloned()
			.unwrap()
	}
}

///Object type
pub struct Element {
	pub graphics: Graphics,
	pub id: String,
	pub collider: avian3d::collision::Collider,
}

pub type Ref = Arc<Element>;

#[derive(Component, Into, From)]
pub struct Component(pub Ref);