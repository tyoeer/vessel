use std::collections::HashMap;

use bevy::{color::palettes::css, prelude::*};
use bevy_replicon::prelude::*;
use serde::{Serialize, Deserialize};
use crate::worldplay;
use worldplay::player::Control as PlayerControl;

pub struct MultiplayerPlugin;

impl Plugin for MultiplayerPlugin {
	fn build(&self, app: &mut App) {
		app
			.init_resource::<ClientEntities>()
			
			.replicate_group::<(MultiPlayer, Transform)>()
			.add_client_event::<PlayerControl>(ChannelKind::Ordered)
			
			.add_systems(Update, mark_players.after(worldplay::player::move_player))
			.add_systems(Update, apply_client_movement
				.after(worldplay::player::spawn_players)
				.before(worldplay::player::move_player)
				.run_if(server_running)
			)
			.add_systems(PreUpdate,
				(
					spawn_player,
				)
				.after(ServerSet::Receive)
				.run_if(server_running)
			)
			.add_systems(PreUpdate, setup_player.after(ClientSet::Receive).run_if(client_connected))
			.add_systems(PostUpdate, send_movement.before(ClientSet::Send).run_if(client_connected))
		;
	}
}


///Maps [ClientId]s to the [Entity] they're controlling
#[derive(Resource, Default)]
pub struct ClientEntities {
	pub map: HashMap<ClientId, Entity>,
}


#[derive(Component, Serialize, Deserialize)]
pub struct MultiPlayer;


pub fn send_movement(
	query: Query<&PlayerControl, (With<worldplay::player::LocallyControlled>, Changed<PlayerControl>)>,
	mut events: EventWriter<PlayerControl>,
) {
	if let Some(control) = query.iter().next() {
		events.send(control.clone());
	}
}

pub fn apply_client_movement(
	mut query: Query<&mut PlayerControl>,
	mut events: EventReader<FromClient<PlayerControl>>,
	client_entities: Res<ClientEntities>,
) {
	for event in events.read() {
		let target = client_entities.map.get(&event.client_id).expect("client sending us events should have a related entity");
		let mut control = query.get_mut(*target).expect("target should exist and have a Control component");
		control.0 = event.event.0;
	}
}


pub fn spawn_player(
	mut cmds: Commands,
	mut ev: EventReader<ServerEvent>,
	mut client_entities: ResMut<ClientEntities>,
	rt_vessel_data: Res<worldplay::player::RtVesselData>,
	mut spawn_events: EventWriter<worldplay::player::SpawnEvent>,
) {
	for event in ev.read() {
		if let ServerEvent::ClientConnected { client_id } = event {
			let id = cmds.spawn(MultiPlayer)
				.insert(Replicated)
				.id();
			client_entities.map.insert(*client_id, id);
			spawn_events.send(worldplay::player::SpawnEvent {
				rt_vessel_data: rt_vessel_data.clone(),
				player_entity: Some(id),
			});
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
	query: Query<(
		&Transform,
		Option<&PlayerControl>,
		Option<&avian3d::prelude::ExternalForce>
		),
		With<MultiPlayer>
	>,
	mut gizmos: Gizmos,
) {
	for (tf, maybe_control, maybe_force) in &query {
		gizmos.sphere(tf.translation, tf.rotation, 0.5, css::WHITE);
		if let Some(control) = maybe_control {
			let offset =  control.0.extend(0.).xzy()*2.;
			gizmos.line(tf.translation, tf.translation + tf.rotation * offset, css::BLUE);
		}
		if let Some(force) = maybe_force {
			let offset = force.force() * 10.;
			gizmos.line(tf.translation, tf.translation + offset, css::RED);
		}
	}
}