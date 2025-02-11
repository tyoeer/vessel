use std::collections::HashMap;

use bevy::{
	color::palettes::css, prelude::*, window::PrimaryWindow
};
use bevy_replicon::prelude::*;
use avian3d::prelude::{AngularVelocity, LinearVelocity, Position, Rotation};
use serde::{Serialize, Deserialize};

use crate::worldplay::{
	self, user, vessel, WorldState
};


pub mod ui;
pub mod network;

pub struct MultiplayerPlugin;

impl Plugin for MultiplayerPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_observer(network::setup_client)
			.add_observer(network::setup_server)
			.init_resource::<ClientOwnedEntities>()
			
			.replicate_group::<(MultiPlayer, vessel::Id, Position, Rotation, LinearVelocity, AngularVelocity)>()
			.add_client_event::<vessel::Control>(ChannelKind::Ordered)
			.add_client_event::<NewUserVessel>(ChannelKind::Unordered)
			.add_server_event::<AddVessel>(ChannelKind::Unordered)
			
			.add_systems(OnEnter(WorldState::Foreground), send_user_vessel.after(user::spawn_user).run_if(client_connected))
		;
		
		#[cfg(feature="user_interface")]
		app
			.add_systems(Update, mark_players.after(vessel::move_vessel))
			.add_systems(Update, ui::debug_ui)
			.add_systems(Update, (
				set_server_window_title.run_if(server_just_started),
				set_client_window_title.run_if(client_just_connected),
			))
			.add_systems(
				OnEnter(WorldState::Foreground),
				mark_server_user.after(user::spawn_user).run_if(server_running)
			);
		
		app
			.add_systems(Update, apply_client_movement
				.after(vessel::spawn_vessels)
				.before(vessel::move_vessel)
				.run_if(server_running)
			)
			.add_systems(PreUpdate,
				(
					spawn_player,
					connection_handler,
				)
				.after(ServerSet::Receive)
				.run_if(server_running)
			)
			.add_systems(PreUpdate, setup_player.after(ClientSet::Receive).run_if(client_connected))
			.add_systems(PreUpdate, receive_server_vessels.after(ClientSet::Receive).run_if(client_connected))
			.add_systems(PostUpdate, send_movement.before(ClientSet::Send).run_if(client_connected))
		;
	}
}

pub fn server_just_started(
    mut last_running: Local<bool>,
    server: Option<Res<RepliconServer>>,
) -> bool {
    let running = server.filter(|server| server.is_running()).is_some();

    let just_stopped = !*last_running && running;
    *last_running = running;
    just_stopped
}

fn set_server_window_title(
	window: Option<Single<&mut Window, With<PrimaryWindow>>>,
) {
	if let Some(mut window) = window {
		window.title = "Server".into();
	}
}
fn set_client_window_title(
	window: Option<Single<&mut Window, With<PrimaryWindow>>>,
	client: Res<RepliconClient>,
) {
	if let Some(mut window) = window {
		window.title = format!("Client {}", client.id().expect("system `set_client_window_title` should only be called when the client is connected").get());
	}
}


///Maps [ClientId]s to the [Entity] they're controlling
#[derive(Resource, Default)]
pub struct ClientOwnedEntities {
	pub map: HashMap<ClientId, Entity>,
}


#[derive(Component, Serialize, Deserialize)]
pub struct MultiPlayer;


pub fn send_movement(
	query: Query<&vessel::Control, (With<worldplay::user::LocallyControlled>, Changed<vessel::Control>)>,
	mut events: EventWriter<vessel::Control>,
) {
	if let Some(control) = query.iter().next() {
		events.send(control.clone());
	}
}

pub fn apply_client_movement(
	mut query: Query<&mut vessel::Control>,
	mut events: EventReader<FromClient<vessel::Control>>,
	client_entities: Res<ClientOwnedEntities>,
) {
	for event in events.read() {
		let target = client_entities.map.get(&event.client_id).expect("client sending us events should have a related entity");
		let mut control = query.get_mut(*target).expect("target should exist and have a Control component");
		control.0 = event.event.0;
	}
}


#[derive(Event, serde::Serialize, serde::Deserialize)]
pub struct AddVessel {
	///The uuid of the sim_vessel asset
	vessel_id: vessel::Id,
	///The data of the vessel
	sim_vessel: vessel::SimVessel,
}


pub fn receive_server_vessels(
	mut add_vessel_events: EventReader<AddVessel>,
	mut vessels: ResMut<Assets<vessel::SimVessel>>,
) {
	for event in add_vessel_events.read() {
		vessels.insert(event.vessel_id.0, event.sim_vessel.clone());
	}
}


#[derive(Event, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NewUserVessel {
	///The uuid of the sim_vessel asset
	vessel_id: vessel::Id,
	///The data of the vessel
	sim_vessel: vessel::SimVessel,
	///The entity id of the prespawned id of the entity
	client_entity: Entity,
}


pub fn send_user_vessel(
	local_vessel_query: Query<(Entity, &vessel::Id), With<user::LocallyControlled>>,
	vessels: Res<Assets<vessel::SimVessel>>,
	mut events: EventWriter<NewUserVessel>,
) {
	let (id, vessel_id) = local_vessel_query.single();
	events.send(NewUserVessel {
		vessel_id: *vessel_id,
		sim_vessel: vessels.get(vessel_id.0).expect("user vessel id should point to existing vessel").clone(),
		client_entity: id,
	});
}


pub fn mark_server_user(
	local: Query<Entity, (With<user::LocallyControlled>, With<vessel::Id>, Without<MultiPlayer>)>,
	mut cmds: Commands,
) {
	for entity in &local {
		cmds.entity(entity)
			.insert(MultiPlayer)
			.insert(Replicated)
		;
	}
}


pub fn spawn_player(
	mut cmds: Commands,
	mut new_user_vessel_events: EventReader<FromClient<NewUserVessel>>,
	mut new_vessel_send: EventWriter<ToClients<AddVessel>>,
	mut client_owned_entities: ResMut<ClientOwnedEntities>,
	mut vessels: ResMut<Assets<vessel::SimVessel>>,
	mut client_entity_map: ResMut<ClientEntityMap>,
) {
	for client_event in new_user_vessel_events.read() {
		vessels.insert(client_event.event.vessel_id.0, client_event.event.sim_vessel.clone());
		
		new_vessel_send.send(ToClients {
			mode: SendMode::BroadcastExcept(client_event.client_id),
			event: AddVessel {
				vessel_id: client_event.event.vessel_id,
				sim_vessel: client_event.event.sim_vessel.clone(),
			}
		});
		
		let id = cmds.spawn(MultiPlayer)
			.insert(Replicated)
			.insert(client_event.event.vessel_id)
			.id();
		
		client_entity_map.insert(client_event.client_id, ClientMapping {
			server_entity: id,
			client_entity: client_event.event.client_entity,
		});
		
		client_owned_entities.map.insert(client_event.client_id, id);
	}
}


pub fn connection_handler(
	mut events: EventReader<ServerEvent>,
	active_vessels: Query<&vessel::Id, With<MultiPlayer>>,
	vessels: Res<Assets<vessel::SimVessel>>,
	mut new_vessel_send: EventWriter<ToClients<AddVessel>>,
	mut cmds: Commands,
	mut client_owned_entities: ResMut<ClientOwnedEntities>,
) {
	for event in events.read() {
		match event {
			ServerEvent::ClientConnected { client_id } => {
				info!(?client_id, "client connected");
				
				for id in &active_vessels {
					let Some(vessel) = vessels.get(id.0)  else {
						// 🤷
						warn!(vessel_id=?id.0, ?client_id, "trying to share vessel \\w client, but we don't have it");
						continue;
					};
					new_vessel_send.send(ToClients {
						mode: SendMode::Direct(*client_id),
						event: AddVessel {
							vessel_id: *id,
							sim_vessel: vessel.clone(),
						},
					});
				}
			},
			ServerEvent::ClientDisconnected { client_id, reason } => {
				info!(?client_id, reason, "client disconnected");
				let maybe_entity = client_owned_entities.map.remove(client_id);
				if let Some(entity) = maybe_entity {
					cmds.entity(entity).despawn_recursive();
				}
			}
		}
	}
}


pub fn setup_player(
	todo: Query<Entity, (With<MultiPlayer>, Without<GlobalTransform>)>,
	mut cmds: Commands,
) {
	for entity in &todo {
		cmds.entity(entity).insert((
			Transform::default(),
			Visibility::default(),
		));
	}
}

pub fn mark_players(
	query: Query<(
		&Transform,
		Option<&vessel::Control>,
		Option<&avian3d::prelude::ExternalForce>
		),
		With<MultiPlayer>
	>,
	mut gizmos: Gizmos,
) {
	for (tf, maybe_control, maybe_force) in &query {
		gizmos.sphere(Isometry3d::new(tf.translation, tf.rotation), 0.5, css::WHITE);
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