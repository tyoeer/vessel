use core::f32;

use bevy::{color::palettes::css::WHITE, prelude::*};
use derive_more::{From, Into};


pub mod player;


pub struct GameplayPlugin<State: States> {
	pub state: State,
}

impl<State: States> Plugin for GameplayPlugin<State> {
	fn build(&self, app: &mut App) {
		app.init_resource::<player::CameraSettings>();
		app.add_systems(OnEnter(self.state.clone()), (
			create_root,
			(
				player::spawn_player,
			).after(create_root)
		));
		app.add_systems(OnExit(self.state.clone()), (
			cleanup_root,
		));
		app.add_systems(Update, (
				player::move_player,
				player::update_camera,
				player::camera_ui,
				demo_graphics,
			)
			.run_if(in_state(self.state.clone()))
		);
	}
}

pub fn demo_graphics(
	mut g: Gizmos,
) {
	g.grid(
		Vec3::ZERO,
		Quat::from_rotation_x(f32::consts::TAU/4.),
		UVec2::new(30,30),
		Vec2::new(1.,1.),
		WHITE
	);
}

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