/*!
Moving around in the world.

*/

use core::f32;

use bevy::prelude::*;
use derive_more::{From, Into};


pub mod vessel;
pub mod user;

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
	fn build(&self, app: &mut App) {
		app.init_state::<WorldState>();
		app.add_computed_state::<WorldLoaded>();
		app.enable_state_scoped_entities::<WorldLoaded>();
		
		app.init_resource::<user::CameraSettings>();
		
		app.init_asset::<vessel::SimVessel>();
		app.register_asset_reflect::<vessel::SimVessel>();
		app.register_type::<vessel::Id>();
		
		app.add_systems(OnEnter(WorldState::Foreground), (
			user::spawn_user,
		));
		app.add_systems(Update, (
				#[cfg(feature="user_interface")]
				user::read_user_input.before(vessel::move_vessel),
				user::update_camera,
				#[cfg(feature="user_interface")]
				user::camera_ui,
			)
			.run_if(in_state(WorldState::Foreground))
		);
		app.add_systems(Update, (
				vessel::spawn_vessels.before(avian3d::prelude::PhysicsSet::Prepare),
				vessel::move_vessel.before(avian3d::prelude::PhysicsSet::StepSimulation),
			)
			.run_if(in_state(WorldLoaded))
		);
	}
}


#[derive(States, Default,Debug, Clone,Copy, PartialEq, Eq, Hash)]
pub enum WorldState {
	///World data is loaded and actively being updated
	Foreground,
	///World data is loaded and present in the background, but not directly being interacted with
	/// Used to keep multiplayer replication up to date in an easy manner
	Background,
	///World is not loaded, no world entities present
	#[default]
	Unloaded,
}

#[derive(Debug, Clone, PartialEq,Eq, Hash)]
pub struct WorldLoaded;

impl ComputedStates for WorldLoaded {
	type SourceStates = WorldState;
	
	fn compute(source: Self::SourceStates) -> Option<Self> {
		match source {
			WorldState::Foreground | WorldState::Background => Some(Self),
			WorldState::Unloaded => None,
		}
	}
}