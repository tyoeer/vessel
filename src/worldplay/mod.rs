/*!
Moving around in the world.

*/

use core::f32;

use bevy::prelude::*;
use derive_more::{From, Into};


pub mod player;


pub struct GameplayPlugin<State: States> {
	pub state: State,
}

impl<State: States> Plugin for GameplayPlugin<State> {
	fn build(&self, app: &mut App) {
		app.init_resource::<player::CameraSettings>();
		app.add_event::<player::SpawnEvent>();
		app.add_systems(OnEnter(self.state.clone()), (
			create_root,
			(
				spawn_player,
			).after(create_root)
		));
		app.add_systems(OnExit(self.state.clone()), (
			cleanup_root,
		));
		app.add_systems(Update, (
				player::spawn_players.before(avian3d::prelude::PhysicsSet::Prepare),
				player::read_player_input.before(player::move_player),
				player::move_player.before(avian3d::prelude::PhysicsSet::StepSimulation),
				player::update_camera,
				player::camera_ui,
			)
			.run_if(in_state(self.state.clone()))
		);
	}
}


pub fn spawn_player(
	mut cmds: Commands,
	player_data: Res<player::RtVesselData>,
	mut spawn_events: EventWriter<player::SpawnEvent>,
) {
	let id = cmds.spawn(player::LocallyControlled).id();
	
	cmds.spawn(Camera3dBundle {
		transform: Transform::from_xyz(0., 0., 0.)
			.looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	}).set_parent(id);
	
	spawn_events.send(player::SpawnEvent {
		rt_vessel_data: player_data.clone(),
		player_entity: Some(id),
	});
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