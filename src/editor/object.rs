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


pub fn create_event_handler(
	mut objs: EventReader<event::Create>,
	mut cmd: Commands,
	object_graphics: Res<Graphics>,
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


pub mod event {
	use bevy::prelude::*;
	use super::super::VesselPos;
	
	
	#[derive(Event)]
	pub struct Create {
		pub pos: VesselPos,
	}
}