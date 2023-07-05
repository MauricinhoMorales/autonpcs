use super::SpawnOwned;
use bevy::{prelude::*, reflect::TypeRegistry, scene::SceneInstance};
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;
use simula_core::epath::EPath;
use simula_script::{Script, ScriptContext};
use std::borrow::Cow;
#[derive(
    Debug, Component, Reflect, FromReflect, Clone, Deserialize, Serialize, InspectorOptions, Default,
)]
#[reflect(InspectorOptions)]
pub struct Spawn {
    pub asset: BehaviorProp<Cow<'static, str>, Cow<'static, str>, String>,
    pub name: BehaviorProp<Cow<'static, str>, Cow<'static, str>, String>,
    pub target: BehaviorProp<EPath, Cow<'static, str>, String>,

    #[serde(skip)]
    #[reflect(ignore)]
    pub scene: Option<Entity>,
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
        changed |= self.asset.ui(Some("asset"), state, ui, type_registry);
        changed |= self.name.ui(Some("name"), state, ui, type_registry);
        changed |= self.target.ui(Some("target"), state, ui, type_registry);
        changed
    }

    fn ui_readonly(
        &self,
        _label: Option<&str>,
        state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        type_registry: &TypeRegistry,
    ) {
        self.asset
            .ui_readonly(Some("asset"), state, ui, type_registry);
        self.name
            .ui_readonly(Some("name"), state, ui, type_registry);
        self.target
            .ui_readonly(Some("target"), state, ui, type_registry);
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
    mut scripts: ResMut<Assets<Script>>,
    script_ctx_handles: Query<&Handle<ScriptContext>>,
    mut script_ctxs: ResMut<Assets<ScriptContext>>,
) {
    for (entity, mut spawn, node, started) in &mut spawns {
        if started.is_some() {
            // reset eval properties
            spawn.asset.value = BehaviorPropValue::None;
            spawn.name.value = BehaviorPropValue::None;
            spawn.target.value = BehaviorPropValue::None;

            // despawn the scene if one already exists
            if let Some(scene) = spawn.scene {
                info!("despawning scene: {:?}", scene);
                commands.entity(scene).despawn_recursive();
            }
            spawn.scene = None;
        } else {
            // if NPC has been spawned
            if let Some(scene) = spawn.scene {
                if let Ok((_owned, children)) = owned_spawns.get(scene) {
                    // if has children, complete with success
                    if let Some(children) = children {
                        if !children.is_empty() {
                            commands.entity(entity).insert(BehaviorSuccess);
                        }
                    }
                }
            }
            // else still working on eval properties and spawning
            else {
                // keep working on eval properties
                if let BehaviorPropValue::None = spawn.asset.value {
                    let result = spawn.asset.fetch(
                        node,
                        &mut scripts,
                        &script_ctx_handles,
                        &mut script_ctxs,
                    );
                    if let Some(Err(err)) = result {
                        error!("Script errored: {:?}", err);
                        commands.entity(entity).insert(BehaviorFailure);
                        continue;
                    }
                }
                if let BehaviorPropValue::None = spawn.name.value {
                    let result =
                        spawn
                            .name
                            .fetch(node, &mut scripts, &script_ctx_handles, &mut script_ctxs);
                    if let Some(Err(err)) = result {
                        error!("Script errored: {:?}", err);
                        commands.entity(entity).insert(BehaviorFailure);
                        continue;
                    }
                }
                if let BehaviorPropValue::None = spawn.target.value {
                    let result = spawn.target.fetch(
                        node,
                        &mut scripts,
                        &script_ctx_handles,
                        &mut script_ctxs,
                    );
                    if let Some(Err(err)) = result {
                        error!("Script errored: {:?}", err);
                        commands.entity(entity).insert(BehaviorFailure);
                        continue;
                    }
                }

                // if all eval properties are ready, spawn the NPC
                if let (
                    BehaviorPropValue::Some(spawn_asset),
                    BehaviorPropValue::Some(spawn_name),
                    BehaviorPropValue::Some(_spawn_target),
                ) = (&spawn.asset.value, &spawn.name.value, &spawn.target.value)
                {
                    // spawn the scene
                    let scene_id = commands
                        .spawn(SceneBundle {
                            scene: asset_server.load(spawn_asset.as_ref()),
                            ..default()
                        })
                        .insert(Name::new(spawn_name.to_owned()))
                        .insert(SpawnOwned(entity))
                        .id();
                    info!("spawning scene: {:?}", scene_id);

                    // keep track of the spawned scene
                    spawn.scene = Some(scene_id);
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
