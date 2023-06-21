use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;
use std::borrow::Cow;

#[derive(Debug, Default, Component, Reflect, Clone, Deserialize, Serialize, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct Spawn {
    #[serde(default)]
    pub message: Cow<'static, str>,
    #[serde(default)]
    pub fail: bool,
    #[serde(default)]
    #[inspector(min = 0.0, max = f64::MAX)]
    pub duration: f64,
    #[serde(skip)]
    pub start: f64,
    #[serde(skip)]
    pub ticks: u64,
}

impl BehaviorInfo for Spawn {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Spawn";
    const DESC: &'static str = "Spawn an NPC";
}

pub fn run(
    time: Res<Time>,
    mut commands: Commands,
    mut spawns: Query<
        (Entity, &mut Spawn, Option<&Name>, Option<&BehaviorStarted>),
        BehaviorRunQuery,
    >,
) {
    for (entity, mut spawn, name, started) in &mut spawns {
        let elapsed = time.elapsed_seconds_f64();
        spawn.ticks += 1;
        if started.is_some() {
            spawn.start = elapsed;
            let name = name.map(|name| name.as_str()).unwrap_or("");
            info!("[{}:{}] {}", entity.index(), name, spawn.message);
        }
        if elapsed - spawn.start > spawn.duration - f64::EPSILON {
            if spawn.fail {
                commands.entity(entity).insert(BehaviorFailure);
            } else {
                commands.entity(entity).insert(BehaviorSuccess);
            }
        }
    }
}
