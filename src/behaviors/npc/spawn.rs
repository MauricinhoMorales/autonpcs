use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;
use std::borrow::Cow;

#[derive(Debug, Default, Component, Reflect, FromReflect, Clone, Deserialize, Serialize, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct Spawn {
    pub asset: Cow<'static, str>,
    #[serde(skip)]
    pub scene: Option<Entity>,
}

// "models/character/Animation_rig/Body.glb#Scene0"

impl BehaviorInfo for Spawn {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Spawn";
    const ICON: &'static str = "🏄";
    const DESC: &'static str = "Spawn an NPC";
}

pub fn run(
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut spawns: Query<
        (Entity, &mut Spawn, Option<&Name>, Option<&BehaviorStarted>),
        BehaviorRunQuery,
    >,
) {
    for (entity, mut spawn, name, started) in &mut spawns {
        let elapsed = time.elapsed_seconds_f64();
        if started.is_some() {
            let scene_id = commands.spawn(SceneBundle {
                scene: asset_server.load(spawn.asset.as_ref()),
                ..default()
            }).id();

            spawn.scene = Some(scene_id);
        }
    }
}

// Remove spawned entities when the behavior is removed
pub fn removed(mut removals: RemovedComponents<Spawn>, mut commands: Commands, spawns: Query<&Spawn>) {
    for entity in &mut removals {
        if let Ok(spawn) = spawns.get(entity) {
            if let Some(scene) = spawn.scene {
                commands.entity(scene).despawn_recursive();
            }
        }
    }
}

