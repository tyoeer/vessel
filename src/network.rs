use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{
	renet::{
		transport::{
			ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport,
			ServerAuthentication, ServerConfig,
		},
		ConnectionConfig, RenetClient, RenetServer,
	},
	RenetChannelsExt,
};

use std::{
	time::SystemTime,
	net::{
		IpAddr, Ipv4Addr, SocketAddr, UdpSocket,
	}
};


pub fn network_ui(
	mut contexts: bevy_egui::EguiContexts,
	mut cmds: Commands,
	channels: Res<RepliconChannels>, // used for network setup
	client: Res<RepliconClient>,
	server: Res<RepliconServer>,
	connected_clients: Res<ConnectedClients>,
) {
	use bevy_egui::egui;
	let Some(ctx) = contexts.try_ctx_mut() else {
		// Primary window is missing, because it still is being initialized or has been closed
		// This system can still run in those conditions, so just do nothing until other systems fix it
		return;
	};

	egui::Window::new("Network").resizable(true).show(ctx, |ui| {
		ui.label(format!("Client status: {:?}", client.status()));
		ui.label(format!("Server running: {}", server.is_running()));
		
		if server.is_running() {
			ui.label("Server running");
			ui.label(format!("{} clients connected", connected_clients.len()));
		} 
		
		if !server.is_running() && client.is_disconnected() {
			if ui.button("Connect as client to localhost").clicked() {
				setup_client(&mut cmds, &channels, Ipv4Addr::LOCALHOST.into());
			}
			if ui.button("Run server").clicked() {
				setup_server(&mut cmds, &channels);
			}
		}
	});
}


const PORT: u16 = 25565; //yoink
const PROTOCOL_ID: u64 = 0; //it's what the example does 🤷


pub fn setup_server(
	cmds: &mut Commands,
	channels: &RepliconChannels,
) {
	let server_channels_config = channels.get_server_configs();
	let client_channels_config = channels.get_client_configs();

	let server = RenetServer::new(ConnectionConfig {
					server_channels_config,
					client_channels_config,
					..Default::default()
	});

	let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("system time should be after the unix epoch");
	let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, PORT)).unwrap();
	let server_config = ServerConfig {
					current_time,
					max_clients: 10,
					protocol_id: PROTOCOL_ID,
					authentication: ServerAuthentication::Unsecure,
					public_addresses: Default::default(),
	};
	let transport = NetcodeServerTransport::new(server_config, socket).unwrap();

	cmds.insert_resource(server);
	cmds.insert_resource(transport);
}

pub fn setup_client(
	cmds: &mut Commands,
	channels: &RepliconChannels,
	server_ip: IpAddr,
) {
	let server_channels_config = channels.get_server_configs();
	let client_channels_config = channels.get_client_configs();

	let client = RenetClient::new(ConnectionConfig {
		server_channels_config,
		client_channels_config,
		..Default::default()
	});

	let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("system time should be after the unix epoch");
	let client_id = current_time.as_millis() as u64;
	let server_addr = SocketAddr::new(server_ip, PORT);
	let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
	let authentication = ClientAuthentication::Unsecure {
		client_id,
		protocol_id: PROTOCOL_ID,
		server_addr,
		user_data: None,
	};
	let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

	cmds.insert_resource(client);
	cmds.insert_resource(transport);
}