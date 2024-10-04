use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_mod_picking::{events::{Click, Pointer}, PickableBundle};
use derive_more::{From, Into};

pub struct VesselPlugin;

impl Plugin for VesselPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<CreateObjectEvent>();
		app.add_systems(Update, (
			create_test_obj
				.run_if(input_just_pressed(KeyCode::Enter)),
			click_handler,
			add_objects,
		))
		;	
	}
}

#[derive(Component, From, Into)]
pub struct VesselPos(pub IVec3);

#[derive(Event)]
pub struct CreateObjectEvent {
	pub pos: VesselPos,
}

fn create_test_obj(
	mut oe: EventWriter<CreateObjectEvent>,
) {
	oe.send(CreateObjectEvent {
		pos: IVec3::new(0, 0, 0).into()
	});
}

fn click_handler(
	mut clicks: EventReader<Pointer<Click>>,
	pos: Query<&VesselPos>,
	mut create: EventWriter<CreateObjectEvent>,
) {
	for click in clicks.read() {
		let ent = click.target;
		let Ok(old_pos) = pos.get(ent) else {continue};
		let Some(hit_normal) = click.hit.normal else {continue};
		let offset = hit_normal.as_ivec3();
		if offset == IVec3::ZERO {continue}
		let pos = old_pos.0 + offset;
		
		create.send(CreateObjectEvent {
			pos: pos.into()
		});
	}
}

fn add_objects(
	mut objs: EventReader<CreateObjectEvent>,
	mut cmd: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	for obj_ev in objs.read() {
		let mat = materials.add(
			StandardMaterial {
				base_color: Color::srgb(0.9, 0.85, 0.8),
				perceptual_roughness: 0.9,
				..default()
			}
		);
		let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
		
		let object_pos = obj_ev.pos.0;
		let object_size = IVec3::new(1,1,1);
		
		let pos = object_pos.as_vec3();
		let offset = object_size.as_vec3() / 2.;
		let transform = Transform::from_translation(pos + offset);
		
		cmd.spawn(PbrBundle {
			mesh,
			material: mat,
			transform,
			..default()
		})
		.insert(VesselPos::from(object_pos))
		.insert(PickableBundle::default());
	}
}