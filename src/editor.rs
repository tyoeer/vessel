use bevy::{input::common_conditions::input_just_pressed, prelude::*};

pub struct VesselPlugin;

impl Plugin for VesselPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update,
			add_objects
				.run_if(input_just_pressed(KeyCode::Enter))
		)
		;	
	}
}


fn add_objects(
	mut cmd: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	let mat = materials.add(
		StandardMaterial {
			base_color: Color::srgb(0.9, 0.85, 0.8),
			perceptual_roughness: 0.9,
			..default()
		}
	);
	let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
	
	let object_pos = IVec3::new(0, 1, 0);
	let object_size = IVec3::new(1,1,1);
	
	let pos = object_pos.as_vec3();
	let offset = object_size.as_vec3() / 2.;
	let transform = Transform::from_translation(pos + offset);
	
	let id = cmd.spawn(PbrBundle {
		mesh,
		material: mat,
		transform,
		..default()
	}).id();
	
	info!("Spawned {:?}", id);
}