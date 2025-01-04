use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_egui::egui::*;

use super::*;


#[derive(Default)]
pub struct EntityInfo {
	pub replicated: bool,
	pub group: bool,
	//contains server entity id if present
	pub mapping: Option<Entity>,
}

///Collects entities related to multiplayer stuff and interesting things about them
pub fn collect_entities(world: &mut World) -> EntityHashMap::<EntityInfo> {
	let mut infos = EntityHashMap::<EntityInfo>::default();

	for replicated in world.query_filtered::<Entity, With<Replicated>>().iter(world) {
		infos.entry(replicated).or_default().replicated = true;
	}

	world.query_filtered::<Entity, (
		With<MultiPlayer>,
		With<vessel::Id>,
		With<Position>,
		With<Rotation>,
		With<LinearVelocity>,
		With<AngularVelocity>,
	)>()
		.iter(world)
		.for_each(|entity| {infos.entry(entity).or_default().group = true});

	let maybe_map = world.get_resource::<ServerEntityMap>().map(|map| map.to_server().clone());
	if let Some(map) = maybe_map {
		for (client,server) in map.iter() {
			infos.entry(*client).or_default().mapping = Some(*server);
		}
	}
	
	infos
}

pub fn multiplayer_entities(ui: &mut Ui, world: &mut World) {
	let infos = collect_entities(world);
	
	for (entity, info) in infos.iter() {
		let name = bevy_inspector_egui::bevy_inspector::guess_entity_name(world, *entity);
		let label = format!(
			"{}{}{} {}{}",
			if info.replicated {'R'} else {'_'},
			if info.group {'G'} else {'.'},
			if info.mapping.is_some() {'M'} else {'.'},
			name,
			info.mapping.map(|se| format!(" server: {se}")).unwrap_or("".into()),
		);
		ui.label(RichText::new(label).font(FontId::monospace(12.)));
		// egui::CollapsingHeader::new(name).show(ui, |ui| {
		// 	bie::bevy_inspector::ui_for_entity(world, entity, ui);
		// });
	}
}


pub fn debug_ui(
	world: &mut World,
) {
	use bevy_egui::{EguiContext, egui};
	// use bevy_inspector_egui as bie;
	// fn entity_query_ui<F: QueryFilter>(world: &mut World) {
	let egui_context = world
		.query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
		.get_single(world);

	let Ok(egui_context) = egui_context else {
		return;
	};
	let mut egui_context = egui_context.clone();
	
	
	egui::Window::new("Multiplayer entities")
		.default_size((400., 160.))
		.show(egui_context.get_mut(), |ui| {
			egui::ScrollArea::both().show(ui, |ui| {
				
				multiplayer_entities(ui, world);
				
				ui.allocate_space(ui.available_size());
			});
		})
	;
	
}