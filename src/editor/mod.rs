use std::sync::Arc;

use bevy::{
	input::common_conditions::input_just_pressed,
	prelude::*
};
use bevy_mod_picking::events::{Click, Pointer};
use derive_more::{From, Into};


pub mod object;
pub mod input;
pub mod misc;



pub struct VesselPlugin<State: States> {
	pub state: State,
}

impl<State: States> Plugin for VesselPlugin<State> {
	fn build(&self, app: &mut App) {
		app
			.add_event::<object::event::Create>()
			.init_resource::<object::Catalogue>()
		;
		app.add_systems(OnEnter(self.state.clone()), (
			create_root,
			(
				misc::setup_lights,
				input::setup_camera,
			).after(create_root)
		));
		app.add_systems(OnExit(self.state.clone()), (
			(
				misc::store_objects,
			).before(cleanup_root),
			cleanup_root,
		));
		app.add_systems(Update, (
				create_test_obj
					.run_if(input_just_pressed(KeyCode::Enter)),
				input::click_handler
					.before(input::move_camera)
					.before(object::create_event_handler),
				object::create_event_handler,
				input::move_camera,
				misc::hotbar_ui,
			)
			.run_if(in_state(self.state.clone()))
		);
	}
}


pub fn setup_catalogue(
	mut catalogue: ResMut<object::Catalogue>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut cmds: Commands,
) {
	let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
	let material = materials.add(StandardMaterial {
		base_color: Color::srgb(0.9, 0.85, 0.8),
		perceptual_roughness: 0.9,
		..default()
	});
	let green = materials.add(StandardMaterial {
		base_color: Color::srgb(0.2, 0.9, 0.2),
		perceptual_roughness: 0.9,
		..default()
	});
	
	let elements = vec![
		object::Element {
			graphics: object::Graphics {
				material,
				mesh: cube.clone()
			}
		},
		object::Element {
			graphics: object::Graphics {
				material: green,
				mesh: cube
			}
		},
	];
	
	catalogue.elements.extend(elements.into_iter().map(Arc::new));
	
	cmds.insert_resource(SelectedElement(catalogue.elements.first().unwrap().clone()));
}


#[derive(Resource, From, Into, Clone)]
pub struct EditorRoot(pub Entity);

#[derive(Resource)]
pub struct SelectedElement(pub object::ElemRef);


#[derive(Component, Clone, From, Into)]
pub struct VesselPos(pub IVec3);


fn create_root(
	mut cmds: Commands
) {
	let root = cmds.spawn_empty()
		.insert(SpatialBundle::default())
		.id();
	cmds.insert_resource(EditorRoot(root));
}

fn cleanup_root(
	mut cmds: Commands,
	root: Res<EditorRoot>,
) {
	let root = root.0;
	cmds.entity(root).despawn_recursive();
	cmds.remove_resource::<EditorRoot>();
}

fn create_test_obj(
	mut oe: EventWriter<object::event::Create>,
	selem: Res<SelectedElement>,
) {
	oe.send(object::event::Create {
		pos: IVec3::new(0, 0, 0).into(),
		element: selem.0.clone(),
	});
}


