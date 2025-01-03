/*!
Everything related to the creation editor.

"Creation" is the catch-all term for whatever you can make in the editor

Coordinate system:
- X+ is forwards
- Y+ is up
- Z+ is to the right, making it a right-handed system
*/

use std::sync::Arc;

use bevy::{
	input::common_conditions::input_just_pressed,
	prelude::*
};
use derive_more::{From, Into};


pub mod object;
pub mod input;
pub mod misc;
pub mod element;


pub struct EditorPlugin<State: States> {
	pub state: State,
}

impl<State: States> Plugin for EditorPlugin<State> {
	fn build(&self, app: &mut App) {
		app
			.add_event::<object::event::Create>()
			.init_resource::<element::Catalogue>()
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
	mut catalogue: ResMut<element::Catalogue>,
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
		element::Element {
			graphics: element::Graphics {
				material,
				mesh: cube.clone()
			},
			id: "block".into(),
			collider: avian3d::collision::Collider::cuboid(1., 1., 1.),
		},
		element::Element {
			graphics: element::Graphics {
				material: green,
				mesh: cube
			},
			id: "green_block".into(),
			collider: avian3d::collision::Collider::cuboid(1., 1., 1.),
		},
	];
	
	catalogue.elements.extend(elements.into_iter().map(Arc::new));
	
	cmds.insert_resource(Hand(catalogue.elements.first().unwrap().clone()));
}


///Entity all editor entities should be (indirect) children of for state management
#[derive(Resource, From, Into, Clone)]
pub struct EditorRoot(pub Entity);

///Element to be placed by the user
#[derive(Resource)]
pub struct Hand(pub element::Ref);




fn create_root(
	mut cmds: Commands
) {
	let root = cmds.spawn_empty()
		.insert((Transform::default(), Visibility::default()))
		.insert(Name::new("Editor Root"))
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
	hand: Res<Hand>,
) {
	oe.send(object::event::Create {
		pos: IVec3::new(0, 0, 0).into(),
		element: hand.0.clone(),
	});
}


