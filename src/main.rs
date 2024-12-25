use avian3d::{prelude::{Gravity, Physics}, PhysicsPlugins};
use bevy::{input::mouse::{MouseButtonInput, MouseWheel}, prelude::*};

mod editor;
mod worldplay;
mod vessel_builder;
mod network;
mod multiplayer;


fn main() {
	let mut app = App::new();
	
	#[cfg(feature="user_interface")]
	app.add_plugins(DefaultPlugins.set(WindowPlugin {
		primary_window: Some(Window {
			//gets rid of input lag
			present_mode: bevy::window::PresentMode::AutoNoVsync,
			..default()
		}),
		..default()
	}));
	
	#[cfg(not(feature="user_interface"))]
	// plugin list copied from https://github.com/bevyengine/bevy/blob/a967c75e92aa08704f11459e4597f6a24bc476c3/crates/bevy_internal/src/default_plugins.rs#L81-L106
	// to replace when bevy 0.15 hits
	app.add_plugins((
		bevy::app::PanicHandlerPlugin,
		bevy::log::LogPlugin::default(),
		bevy::core::TaskPoolPlugin::default(),
		bevy::core::TypeRegistrationPlugin,
		bevy::core::FrameCountPlugin,
		bevy::time::TimePlugin,
		bevy::transform::TransformPlugin,
		bevy::hierarchy::HierarchyPlugin,
		bevy::diagnostic::DiagnosticsPlugin,
		bevy::app::ScheduleRunnerPlugin::default(),
		bevy::asset::AssetPlugin::default(),
		bevy::scene::ScenePlugin,
		bevy::animation::AnimationPlugin,
		bevy::state::app::StatesPlugin,
		bevy::gltf::GltfPlugin::default(), // used to load the track collider
	));
	#[cfg(not(feature="user_interface"))]
	app.init_asset::<Mesh>() //required by avian3d to create a collider from a mesh
		.init_asset::<bevy::pbr::StandardMaterial>() //required by the gltf loader I think?
		.register_type::<bevy::render::view::visibility::Visibility>() // required to spawn the track scene
		.register_type::<bevy::render::view::visibility::InheritedVisibility>() // required to spawn the track scene
		.register_type::<bevy::render::view::visibility::ViewVisibility>() // required to spawn the track scene
		.register_type::<bevy::render::primitives::Aabb>() // required to spawn the track scene
	;
	
	app
	
	.add_plugins(PhysicsPlugins::default())
	.insert_resource(Gravity(-Vec3::Y * 15.))
	
	.add_plugins((
		bevy_replicon::RepliconPlugins,
		bevy_replicon_renet::RepliconRenetPlugins,
	))
	;
	
	#[cfg(feature="user_interface")]
	app.add_plugins(bevy_egui::EguiPlugin)
		.add_systems(
			PreUpdate,
			absorb_egui_inputs
			.after(bevy_egui::systems::process_input_system)
			.before(bevy_egui::EguiSet::BeginPass)
		);
	
	
	#[cfg(feature="user_interface")]
	app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
	
	#[cfg(not(feature="user_interface"))]
	app.insert_state(GameState::WorldPlay)
		.add_systems(Startup, network::setup_server_system);
	#[cfg(feature="user_interface")]
	app.insert_state(GameState::EditVessel)
		//TODO redesign catalogue so a headless server can use it
		.add_systems(Startup, editor::setup_catalogue)
		.add_systems(Startup, setup_ui_style)
		.add_systems(Update, state_ui)
		.add_systems(Update, network::network_ui);

	// needed for the server
	
	#[cfg(feature="user_interface")]
	app.add_plugins(editor::EditorPlugin {
		state: GameState::EditVessel
	});
	
	app
	.add_plugins(worldplay::GameplayPlugin {
		state: GameState::WorldPlay
	});
	
	//Depends on the GameplayPlugin, so should be added later
	app.add_plugins(multiplayer::MultiplayerPlugin);
	
	app.add_systems(OnTransition {
		exited: GameState::EditVessel,
		entered: GameState::WorldPlay
	}, vessel_builder::build_vessel_system)
	
	.add_systems(Startup, setup_demo_track)
	
	.run();
}


#[derive(States, Debug,Clone, PartialEq, Eq, Hash)]
pub enum GameState {
	WorldPlay,
	EditVessel,
}


fn setup_demo_track(
	mut cmds: Commands,
	assets: ResMut<AssetServer>
) {
	use avian3d::prelude::*;
	
	let scene = assets.load("local/track.glb#Scene0");
	cmds.spawn(SceneBundle {
		scene: SceneRoot(scene),
		transform: Transform::from_xyz(0.,-10.,-5.),
		..default()
	})
	.insert(Name::new("World/Track"))
	.insert(ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMeshWithConfig(
		TrimeshFlags::MERGE_DUPLICATE_VERTICES
		| TrimeshFlags::DELETE_DUPLICATE_TRIANGLES 
		| TrimeshFlags::DELETE_DEGENERATE_TRIANGLES
		| TrimeshFlags::FIX_INTERNAL_EDGES
		| TrimeshFlags::DELETE_BAD_TOPOLOGY_TRIANGLES
	)))
	.insert(CollisionMargin(0.002)) // should help with stability around trimeshes
	.insert(RigidBody::Static);
}


fn state_ui(
	mut contexts: bevy_egui::EguiContexts,
	mut next_state: ResMut<NextState<GameState>>,
) {
	use bevy_egui::egui;
	let Some(ctx) = contexts.try_ctx_mut() else {
		// Primary window is missing, because it still is being initialized or has been closed
		// This system can still run in those conditions, so just do nothing until other systems fix it
		return;
	};

	egui::Window::new("States").resizable(true).show(ctx, |ui| {
		if ui.button("Edit").clicked() {
			next_state.set(GameState::EditVessel);
		}
		if ui.button("Play").clicked() {
			next_state.set(GameState::WorldPlay);
		}
	});
}


fn setup_ui_style(
	mut contexts: bevy_egui::EguiContexts,
) {
	use bevy_egui::egui;
	let Some(ctx) = contexts.try_ctx_mut() else {
		// Primary window is missing, because it still is being initialized or has been closed
		// This system can still run in those conditions, so just do nothing until other systems fix it
		return;
	};
	
	ctx.style_mut(|style| {
		style.visuals.window_shadow = egui::Shadow::NONE;
		style.spacing.slider_width = 300.;
	});
}

///Prevents inputs that egui is using from affecting the rest of the game
// Based on https://github.com/mvlabat/bevy_egui/issues/47#issuecomment-2368811068
fn absorb_egui_inputs(
	mut contexts: bevy_egui::EguiContexts,
	mut mouse: ResMut<ButtonInput<MouseButton>>,
	mut mouse_wheel: ResMut<Events<MouseWheel>>,
	mut mouse_button_events: ResMut<Events<MouseButtonInput>>,
	mut keyboard: ResMut<ButtonInput<KeyCode>>,
	mut picking_settings: ResMut<bevy::picking::input::PointerInputPlugin>,
) {
	//bevy_mod_picking runs too early, so we have to disable it some other way
	picking_settings.is_mouse_enabled = true;
	
	let Some(ctx) = contexts.try_ctx_mut() else {
		//Bevy is slow exiting after the window has been closed
		// So this still runs while there's no context anymore
		return;
	};
	
	if ctx.wants_pointer_input() || ctx.is_pointer_over_area() {
		picking_settings.is_mouse_enabled = false;
		mouse.reset_all();
		mouse_wheel.clear();
		mouse_button_events.clear();
	}
	
	if ctx.wants_keyboard_input() {
		let dont_absorb = [
			KeyCode::SuperLeft,
			KeyCode::SuperRight,
			KeyCode::ControlLeft,
			KeyCode::ControlRight,
			KeyCode::AltLeft,
			KeyCode::AltRight,
			KeyCode::ShiftLeft,
			KeyCode::ShiftRight,
		];
		
		let dont_absorb_pressed = dont_absorb.map(|key| keyboard.pressed(key).then_some(key));
		
		keyboard.reset_all();
		
		for key in dont_absorb_pressed.into_iter().flatten() {
			keyboard.press(key);
		}
	}
	
}