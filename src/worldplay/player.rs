use bevy::prelude::*;
use super::*;


#[derive(Component)]
pub struct Player {
	
}


pub fn spawn_player(
	mut cmds: Commands,
	root: Res<GameplayRoot>,
) {
	let player = cmds.spawn(Player {
		
	}).insert(SpatialBundle::default())
	.set_parent(root.0)
	.id();

	cmds.spawn(Camera3dBundle {
		transform: Transform::from_xyz(0., 1., 2.)
			.looking_to(Vec3::ZERO, Vec3::Y),
		..default()
	}).set_parent(player);
}


pub fn move_player(
	mut players: Query<(&Player, &mut Transform)>,
	buttons: Res<ButtonInput<KeyCode>>,
	timer: Res<Time>,
) {
	let mut move_dir = Vec2::ZERO;
	
	if buttons.pressed(KeyCode::KeyW) {
		move_dir += Vec2::Y;
	}
	if buttons.pressed(KeyCode::KeyS) {
		move_dir -= Vec2::Y;
	}
	if buttons.pressed(KeyCode::KeyD) {
		move_dir += Vec2::X;
	}
	if buttons.pressed(KeyCode::KeyA) {
		move_dir -= Vec2::X;
	}
	
	for (player, mut tf) in &mut players {
		let speed = 10.;
		
		let vel = Vec3::new(move_dir.x, 0., -move_dir.y) * speed * timer.delta_seconds();
		tf.translation += vel;
	}
}