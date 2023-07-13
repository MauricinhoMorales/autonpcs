use super::SpawnOwned;
use bevy::{prelude::*, reflect::TypeRegistry, scene::SceneInstance};
use bevy_inspector_egui::{egui, prelude::*};
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;
use simula_core::epath::{self, EPathQueries};

#[derive(
    Debug, Component, Reflect, FromReflect, Clone, Deserialize, Serialize, InspectorOptions, Default,
)]
#[reflect(InspectorOptions)]
pub struct Spawn {
    pub asset: BehaviorPropStr,
    pub name: BehaviorPropStr,
    #[serde(default)]
    pub target: BehaviorPropOption<BehaviorPropEPath>,

    #[serde(skip)]
    #[reflect(ignore)]
    pub scenes: Vec<Entity>,
}

impl BehaviorSpec for Spawn {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Spawn";
    const ICON: &'static str = "🏄";
    const DESC: &'static str = "Spawn an NPC";
}

impl BehaviorUI for Spawn {
    fn ui(
        &mut self,
        _label: Option<&str>,
        state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        type_registry: &TypeRegistry,
    ) -> bool {
        let mut changed = false;
        changed |= behavior_ui!(self, asset, state, ui, type_registry);
        changed |= behavior_ui!(self, name, state, ui, type_registry);
        changed |= behavior_ui!(self, target, state, ui, type_registry);
        changed
    }

    fn ui_readonly(
        &self,
        _label: Option<&str>,
        state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        type_registry: &TypeRegistry,
    ) {
        behavior_ui_readonly!(self, asset, state, ui, type_registry);
        behavior_ui_readonly!(self, name, state, ui, type_registry);
        behavior_ui_readonly!(self, target, state, ui, type_registry);

        // show if we have scenes
        for scene in &self.scenes {
            ui.label(egui::RichText::new(format!("scene: {:?}", scene)).small());
        }
    }
}

pub fn run(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut spawns: Query<
        (Entity, &mut Spawn, &BehaviorNode, Option<&BehaviorStarted>),
        BehaviorRunQuery,
    >,
    owned_spawns: Query<(Entity, Option<&Children>), (With<SpawnOwned>, With<SceneInstance>)>,
    mut scripts: ScriptQueries,
    equeries: EPathQueries,
) {
    for (entity, mut spawn, node, started) in &mut spawns {
        if started.is_some() {
            // reset eval properties
            spawn.asset.value = BehaviorPropValue::None;
            spawn.name.value = BehaviorPropValue::None;
            if let Some(target) = &mut *spawn.target {
                target.value = BehaviorPropValue::None;
            }

            // despawn scenes if they already exists
            for scene in &spawn.scenes {
                info!("despawning scene: {:?}", scene);
                commands.entity(*scene).despawn_recursive();
            }
            spawn.scenes.clear();
        } else {
            // if NPC has been spawned
            if spawn.scenes.len() > 0 {
                let mut successes = 0;
                for scene in &spawn.scenes {
                    if let Ok((_owned, children)) = owned_spawns.get(*scene) {
                        // if has children, complete with success
                        if let Some(children) = children {
                            if !children.is_empty() {
                                successes += 1;
                            }
                        }
                    }
                }
                if successes == spawn.scenes.len() {
                    commands.entity(entity).insert(BehaviorSuccess);
                }
            }
            // else still working on eval properties and spawning
            else {
                // keep working on eval properties
                if let BehaviorPropValue::None = spawn.asset.value {
                    let result = spawn.asset.fetch(node, &mut scripts);
                    if let Some(Err(err)) = result {
                        error!("Script errored: {:?}", err);
                        commands.entity(entity).insert(BehaviorFailure);
                        continue;
                    }
                }
                if let BehaviorPropValue::None = spawn.name.value {
                    let result = spawn.name.fetch(node, &mut scripts);
                    if let Some(Err(err)) = result {
                        error!("Script errored: {:?}", err);
                        commands.entity(entity).insert(BehaviorFailure);
                        continue;
                    }
                }
                if let Some(prop) = &mut spawn.target.as_mut() {
                    if let BehaviorPropValue::None = prop.value {
                        let result = prop.fetch(node, &mut scripts);
                        if let Some(Err(err)) = result {
                            error!("Script errored: {:?}", err);
                            commands.entity(entity).insert(BehaviorFailure);
                            continue;
                        }
                    }
                }

                // if we have a spawn target, check if ready
                let spawn_target = if let Some(prop) = &*spawn.target {
                    if let BehaviorPropValue::Some(value) = &prop.value {
                        Some(Some(value))
                    } else {
                        None
                    }
                } else {
                    Some(None)
                };

                // if all eval properties are ready, spawn the NPC
                if let (
                    BehaviorPropValue::Some(spawn_asset),
                    BehaviorPropValue::Some(spawn_name),
                    Some(spawn_target),
                ) = (&spawn.asset.value, &spawn.name.value, &spawn_target)
                {
                    let mut scenes = vec![];

                    let targets = if let Some(spawn_target) = spawn_target {
                        epath::select(None, spawn_target, &equeries)
                            .into_iter()
                            .map(Some)
                            .collect()
                    } else {
                        vec![None]
                    };

                    for target in &targets {
                        // spawn the scene
                        let scene_id = commands
                            .spawn(SceneBundle {
                                scene: asset_server.load(spawn_asset.as_ref()),
                                ..default()
                            })
                            .insert(Name::new(spawn_name.to_owned()))
                            .insert(SpawnOwned(entity))
                            .id();

                        if let Some(target) = target {
                            info!(
                                "spawning scene: {:?} for target: {:?}",
                                spawn_name, target.name
                            );
                            commands.entity(target.entity).add_child(scene_id);
                        } else {
                            info!("spawning scene: {:?}", scene_id);
                        }

                        // keep track of the spawned scene
                        scenes.push(scene_id);
                    }

                    // if no scenes were spawned, fail
                    if scenes.len() == 0 {
                        warn!("No scenes spawned for: {:?}", spawn_name);
                        commands.entity(entity).insert(BehaviorFailure);
                    }

                    for scene in &scenes {
                        spawn.scenes.push(*scene);
                    }
                }
            }
        }
    }
}

// Remove spawned entities when the behavior is removed
pub fn removed(
    mut removals: RemovedComponents<Spawn>,
    mut commands: Commands,
    owned_spawns: Query<(Entity, &SpawnOwned)>,
) {
    // Iterate over all removed Spawns
    for entity in &mut removals {
        // Remove all SpawnOwned by this entity
        for (owned_entity, spawn) in &owned_spawns {
            if **spawn == entity {
                info!("Despawning scene: {:?}", owned_entity);
                commands.entity(owned_entity).despawn_recursive();
            }
        }
    }
}
