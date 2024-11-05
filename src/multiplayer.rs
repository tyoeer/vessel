use bevy::{color::palettes::css, prelude::*};
use bevy_replicon::prelude::*;
use serde::{Serialize, Deserialize};

pub struct MultiplayerPlugin;

impl Plugin for MultiplayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, mark_players);
		app.add_systems(PreUpdate, spawn_player.after(ServerSet::Receive).run_if(server_running));
		app.add_systems(PreUpdate, setup_player.after(ClientSet::Receive).run_if(client_connected));
		app.replicate_group::<(MultiPlayer, Transform)>();
	}
}


#[derive(Component, Serialize, Deserialize)]
pub struct MultiPlayer;


pub fn spawn_player(
	mut cmds: Commands,
	mut ev: EventReader<ServerEvent>,
) {
	for event in ev.read() {
		if let ServerEvent::ClientConnected { client_id: _ } = event {
			cmds.spawn(MultiPlayer)
				.insert(SpatialBundle::default())
				.insert(Replicated)
			;
		}
	}
}

pub fn setup_player(
	todo: Query<Entity, (With<MultiPlayer>, Without<GlobalTransform>)>,
	mut cmds: Commands,
) {
	for entity in &todo {
		cmds.entity(entity).insert((
			GlobalTransform::default(),
			VisibilityBundle::default(),
		));
	}
}

pub fn mark_players(
	query: Query<&Transform, With<MultiPlayer>>,
	mut gizmos: Gizmos,
) {
	for tf in &query {
		gizmos.sphere(tf.translation, tf.rotation, 0.5, css::WHITE);
	}
}