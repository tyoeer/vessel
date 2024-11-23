/*!
Moving around in the world.

*/

use core::f32;

use bevy::prelude::*;
use derive_more::{From, Into};


pub mod vessel;
pub mod user;

pub struct GameplayPlugin<State: States> {
	pub state: State,
}

impl<State: States> Plugin for GameplayPlugin<State> {
	fn build(&self, app: &mut App) {
		app.init_resource::<user::CameraSettings>();
		app.init_asset::<vessel::SimVessel>();
		app.register_asset_reflect::<vessel::SimVessel>();
		app.add_systems(OnEnter(self.state.clone()), (
			create_root,
			(
				user::spawn_user,
			).after(create_root)
		));
		app.add_systems(OnExit(self.state.clone()), (
			cleanup_root,
		));
		app.add_systems(Update, (
				vessel::spawn_vessels.before(avian3d::prelude::PhysicsSet::Prepare),
				#[cfg(feature="user_interface")]
				user::read_user_input.before(vessel::move_vessel),
				vessel::move_vessel.before(avian3d::prelude::PhysicsSet::StepSimulation),
				user::update_camera,
				#[cfg(feature="user_interface")]
				user::camera_ui,
			)
			.run_if(in_state(self.state.clone()))
		);
	}
}





///Entity all worldplay entities should be (indirect) children of for state management
#[derive(Resource, From, Into, Clone)]
pub struct GameplayRoot(pub Entity);

pub fn create_root(
	mut cmds: Commands
) {
	let root = cmds.spawn_empty()
		.insert(SpatialBundle::default())
		.id();
	cmds.insert_resource(GameplayRoot(root));
}

pub fn cleanup_root(
	mut cmds: Commands,
	root: Res<GameplayRoot>,
) {
	let root = root.0;
	cmds.entity(root).despawn_recursive();
	cmds.remove_resource::<GameplayRoot>();
}