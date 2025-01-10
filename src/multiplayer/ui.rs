use bevy::{
	prelude::*,
	ecs::{
		component::ComponentId,
		entity::EntityHashMap,
	},
	window::PrimaryWindow,
};
use bevy_replicon::{
	prelude::*,
	core::{
		replication::replication_rules::ReplicationRules,
		server_entity_map::ServerEntityMap,
	},
};
use bevy_egui::egui::*;
use bevy_inspector_egui as bie;



pub const FONT_SIZE: f32 = 12.;


///Put the first part into a monospaced font
pub fn mono_start(monospaced: impl AsRef<str>, normal: impl AsRef<str>) -> text::LayoutJob {
	let mut text = text::LayoutJob::default();
	text.append(
		monospaced.as_ref(),
		0.,
		TextFormat {
			font_id: FontId::monospace(FONT_SIZE),
			..default()
		}
	);
	text.append(
		normal.as_ref(),
		10.,
		TextFormat::default(),
	);
	
	text
}

pub fn collect_replication_groups(world: &World) -> Vec<Vec<ComponentId>> {
	let rules = world.resource::<ReplicationRules>();
	rules.iter()
		.map(|rule| &rule.components)
		.map(|cs| cs.iter()
			.map(|c_f| c_f.0)
			.collect::<Vec<_>>()
		)
		.collect()
}


#[derive(Default)]
pub struct EntityInfo {
	pub replicated: bool,
	pub groups: Vec<bool>,
	//contains server entity id if present
	pub mapping: Option<Entity>,
}


///Collects entities related to multiplayer stuff and interesting things about them
pub fn collect_entities(world: &mut World) -> EntityHashMap::<EntityInfo> {
	let mut infos = EntityHashMap::<EntityInfo>::default();

	for replicated in world.query_filtered::<Entity, With<Replicated>>().iter(world) {
		infos.entry(replicated).or_default().replicated = true;
	}
	
	let groups = collect_replication_groups(world);
	let group_count = groups.len();
	
	for (i, group) in groups.into_iter().enumerate() {
		let mut query_builder = QueryBuilder::<Entity>::new(world);
		for comp_id in group {
			query_builder.with_id(comp_id);
		}
		let mut q = query_builder.build();
		
		for entity in q.iter(world) {
			let info = infos.entry(entity).or_default();
			info.groups.resize(group_count, false);
			info.groups[i] = true;
		}
	}
	

	let maybe_map = world.get_resource::<ServerEntityMap>().map(|map| map.to_server().clone());
	if let Some(map) = maybe_map {
		for (client,server) in map.iter() {
			infos.entry(*client).or_default().mapping = Some(*server);
		}
	}
	
	//Make sure all the group Vecs have false values, even the entities collected through other means
	for entity_info in infos.values_mut() {
		entity_info.groups.resize(group_count, false)
	}
	
	infos
}

pub fn multiplayer_entities_ui(ui: &mut Ui, world: &mut World) {
	let infos = collect_entities(world);
	
	let groups = collect_replication_groups(world);
	
	for (entity, info) in infos.iter() {
		let name = bie::bevy_inspector::guess_entity_name(world, *entity);
		let label = format!(
			"{}{}{} {}{}",
			if info.replicated {'R'} else {'_'},
			if info.mapping.is_some() {'M'} else {'.'},
			if info.groups.iter().any(|x| *x) {'G'} else {'.'},
			name,
			info.mapping.map(|se| format!(" server: {se}")).unwrap_or("".into()),
		);
		ui.collapsing(RichText::new(label).font(FontId::monospace(FONT_SIZE)), |ui| {
			if info.replicated {
				ui.label(mono_start("R:","Has `Replicated` component"));
			} else {
				ui.label(mono_start("_:","`Replicated` component is missing!"));
			}
			if let Some(server_id) = info.mapping {
				ui.label(mono_start("M:",format!("Is mapped to server entity {}", server_id)));
			} else {
				ui.label(mono_start(".:","No known mapping to a server entity, probably because this is the server"));
			}
			if info.groups.iter().any(|x| *x) {
				ui.label(mono_start("G:","The components for at least 1 replication group are present"));
			} else {
				ui.label(mono_start(".:","Entity matches no replication groups"));
			}
			
			ui.label("Groups:");
			for (i, group) in info.groups.iter().enumerate() {
				let number = format!("{i:2}:");
				if *group {
					ui.label(mono_start(number,"✅ All components present"));
				} else {
					let group_comps = &groups[i];
					let missing_comps = group_comps.iter()
						.filter(|comp| !world.entity(*entity).contains_id(**comp))
						.map(|comp_id| world
							.components()
							.get_name(*comp_id)
							.expect("component in replication rule should be registered with the bevy world")
						)
						.map(shorten_component_name)
						.collect::<Vec<_>>()
						.join(", ");
					ui.label(mono_start(number, format!("❌ Missing: {missing_comps}")));
				}
			}
			
			ui.separator();
			ui.collapsing("Components", |ui| {
				bie::bevy_inspector::ui_for_entity(world, *entity, ui);
			})
		});
	}
}

pub fn shorten_component_name(name: &str) -> &str {
	const SPLIT: &str = "::";
	
	let parts = name.split(SPLIT).collect::<Vec<_>>();
	let last = parts.last().expect("component name should not be empty");
	if last.len() <= 3 && parts.len() >= 2 {
		let second_to_last = parts[parts.len()-2];
		let idx = name.len() - second_to_last.len() - SPLIT.len() - last.len();
		return &name[idx..];
	}
	
	last
}

pub fn groups_ui(ui: &mut Ui, world: &mut World) {
	let rules = collect_replication_groups(world);
	for (number, rule) in rules.into_iter().enumerate() {
		let comp_names_short = rule.iter().map(
			|comp_id| world
				.components()
				.get_name(*comp_id)
				.expect("component in replication rule should be registered with the bevy world")
		).map(shorten_component_name).collect::<Vec<_>>().join(", ");
		
		let heading = mono_start(format!("{number:2}"), comp_names_short);
		ui.collapsing(heading, |ui| {
			for component_id in rule {
				let name = world.components()
					.get_name(component_id)
					.expect("component in replication rule should be registered with the bevy world");
				ui.label(name);
			}
		});
	}
}


#[derive(Default)]
pub enum Tab {
	#[default]
	Entities,
	Groups,
}


pub fn debug_ui(
	world: &mut World,
	mut tab: Local<Tab>,
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
				ui.horizontal(|ui| {
					if ui.button("Entities").clicked() {
						*tab = Tab::Entities;
					}
					if ui.button("Groups").clicked() {
						*tab = Tab::Groups;
					}
				});
				
				ui.separator();
				
				match *tab {
					Tab::Entities => multiplayer_entities_ui(ui, world),
					Tab::Groups => groups_ui(ui, world),
				}
				
				ui.allocate_space(ui.available_size());
			});
		})
	;
	
}