use std::sync::Arc;

use bevy::{
	input::{
		common_conditions::input_just_pressed,
		mouse::MouseMotion,
	},
	prelude::*
};
use bevy_mod_picking::events::{Click, Pointer};
use derive_more::{From, Into};


pub mod object;


pub struct VesselPlugin;

impl Plugin for VesselPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_event::<object::event::Create>()
			.init_resource::<object::Graphics>()
			.init_resource::<object::Catalogue>()
		;
	app.add_systems(Startup, 
		setup_catalogue
	);
		app
			.add_systems(Update, (
				create_test_obj
					.run_if(input_just_pressed(KeyCode::Enter)),
				click_handler,
				object::create_event_handler,
				camera,
				hotbar_ui,
			))
		;	
	}
}


fn setup_catalogue(
	mut catalogue: ResMut<object::Catalogue>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut cmds: Commands,
) {
	let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
	let material = materials.add(StandardMaterial {
		base_color: Color::srgb(0.9, 0.85, 0.8),
		perceptual_roughness: 0.9,
		..default()
	});
	let green = materials.add(StandardMaterial {
		base_color: Color::srgb(0.2, 0.9, 0.2),
		perceptual_roughness: 0.9,
		..default()
	});
	
	let elements = vec![
		object::Element {
			graphics: object::Graphics {
				material,
				mesh: cube.clone()
			}
		},
		object::Element {
			graphics: object::Graphics {
				material: green,
				mesh: cube
			}
		},
	];
	
	catalogue.elements.extend(elements.into_iter().map(Arc::new));
	
	cmds.insert_resource(SelectedElement(catalogue.elements.first().unwrap().clone()));
}


#[derive(Resource)]
pub struct SelectedElement(pub object::ElemRef);


pub fn hotbar_ui(
	mut contexts: bevy_egui::EguiContexts,
	catalogue: Res<object::Catalogue>,
	mut selected: ResMut<SelectedElement>,
) {
	use bevy_egui::egui;
	let Some(ctx) = contexts.try_ctx_mut() else {
		// Primary window is missing, because it still is being initialized or has been closed
		// This system can still run in those conditions, so just do nothing until other systems fix it
		return;
	};
	
	egui::Window::new("Hotbar").resizable(true).default_height(50.).show(ctx, |ui| {
		ui.with_layout(egui::Layout {
			main_dir: egui::Direction::LeftToRight,
			main_wrap: false,
			main_align: egui::Align::Center,
			main_justify: false,
			cross_align: egui::Align::Center,
			cross_justify: false,
		}, |ui| {
			for (i, elem) in catalogue.elements.iter().enumerate() {
				let button = egui::Button::new(i.to_string())
					.min_size((40.,40.).into());
				let button_res = ui.add(button);
				
				if button_res.clicked() {
					*selected = SelectedElement(elem.clone());
				}
			}
		})
	});
}


const SENSITIVITY: f32 = 0.005;
const MOVE_SPEED: f32 = 12.;

pub fn camera(
	mut camera_transforms: Query<&mut Transform, With<Camera>>,
	mouse_buttons: Res<ButtonInput<MouseButton>>,
	key_codes: Res<ButtonInput<KeyCode>>,
	mut mouse_motion_events: EventReader<MouseMotion>,
	timer: Res<Time>,
	// mut gizmos: Gizmos,
) {
	if !mouse_buttons.pressed(MouseButton::Right) {return}
	
	for mut tf in &mut camera_transforms {
		
		//The camera's mapping from world space to window/viewport space is weird
		// and the documentation appears to be wrong
		// The correctness of the following code has been determined empirically by testing
		
		for ev in mouse_motion_events.read() {
			tf.rotate_y(-ev.delta.x * SENSITIVITY);
			tf.rotate_local_x(ev.delta.y * SENSITIVITY);
		}
		//make sure Y stays up
		let forward = tf.forward();
		//camera y goes down, so we need to flip it to make world Y render upwards
		tf.look_to(forward, -Dir3::Y);
		
		//camera: x+ = left, y+ = down, z+ = back
		let mut local_offset = Vec3::ZERO;
		if key_codes.pressed(KeyCode::KeyW) {
			local_offset.z -= 1.;
		}
		if key_codes.pressed(KeyCode::KeyS) {
			local_offset.z += 1.;
		}
		if key_codes.pressed(KeyCode::KeyE) {
			local_offset.y -= 1.;
		}
		if key_codes.pressed(KeyCode::KeyQ) {
			local_offset.y += 1.;
		}
		if key_codes.pressed(KeyCode::KeyD) {
			local_offset.x -= 1.;
		}
		if key_codes.pressed(KeyCode::KeyA) {
			local_offset.x += 1.;
		}
		
		let offset = tf.rotation.mul_vec3(local_offset * MOVE_SPEED * timer.delta_seconds());
		tf.translation += offset;
		
		// const SIZE: f32 = 0.3;
		// use bevy::color::palettes::css::{BLACK, BLUE, GREEN, RED};
		
		// let pos = tf.translation + tf.forward()*5.;
		// if kcs.pressed(KeyCode::KeyR) {
		// 	gizmos.line(pos, pos+tf.local_x()*SIZE, RED);
		// 	gizmos.line(pos, pos+tf.local_y()*SIZE, GREEN);
		// 	gizmos.line(pos, pos+tf.local_z()*SIZE, BLUE);
		// } else {
		// 	gizmos.line(pos, pos+Vec3::X*SIZE, RED);
		// 	gizmos.line(pos, pos+Vec3::Y*SIZE, GREEN);
		// 	gizmos.line(pos, pos+Vec3::Z*SIZE, BLUE);
		// }
		// gizmos.sphere(Vec3::ZERO, Quat::default(), 1., BLACK);
	}
}


#[derive(Component, From, Into)]
pub struct VesselPos(pub IVec3);


fn create_test_obj(
	mut oe: EventWriter<object::event::Create>,
	selem: Res<SelectedElement>,
) {
	oe.send(object::event::Create {
		pos: IVec3::new(0, 0, 0).into(),
		element: selem.0.clone(),
	});
}

fn click_handler(
	mut clicks: EventReader<Pointer<Click>>,
	pos: Query<&VesselPos>,
	mut create: EventWriter<object::event::Create>,
	selem: Res<SelectedElement>,
) {
	for click in clicks.read() {
		let ent = click.target;
		let Ok(old_pos) = pos.get(ent) else {continue};
		let Some(hit_normal) = click.hit.normal else {continue};
		let offset = hit_normal.as_ivec3();
		if offset == IVec3::ZERO {continue}
		let pos = old_pos.0 + offset;
		
		create.send(object::event::Create {
			pos: pos.into(),
			element: selem.0.clone(),
		});
	}
}

