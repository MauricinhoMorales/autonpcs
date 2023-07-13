use bevy::{prelude::*, reflect::TypeRegistry};
use bevy_inspector_egui::{egui, prelude::*};
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;
use simula_core::epath::{self, EPathQueries};

#[derive(
    Debug, Component, Reflect, FromReflect, Clone, Deserialize, Serialize, InspectorOptions, Default,
)]
#[reflect(InspectorOptions)]
pub struct Anim {
    pub asset: BehaviorPropStr,
    pub target: BehaviorPropEPath,
    #[serde(default)]
    pub repeat: BehaviorPropGeneric<bool>,
    #[serde(skip)]
    #[reflect(ignore)]
    pub clip: Option<Handle<AnimationClip>>,
}

impl BehaviorSpec for Anim {
    const TYPE: BehaviorType = BehaviorType::Action;
    const NAME: &'static str = "Anim";
    const ICON: &'static str = "🏋";
    const DESC: &'static str = "Play animation on NPC";
}

impl BehaviorUI for Anim {
    fn ui(
        &mut self,
        _label: Option<&str>,
        state: Option<protocol::BehaviorState>,
        ui: &mut bevy_inspector_egui::egui::Ui,
        type_registry: &TypeRegistry,
    ) -> bool {
        let mut changed = false;
        changed |= behavior_ui!(self, asset, state, ui, type_registry);
        changed |= behavior_ui!(self, target, state, ui, type_registry);
        changed |= behavior_ui!(self, repeat, state, ui, type_registry);
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
        behavior_ui_readonly!(self, target, state, ui, type_registry);
        behavior_ui_readonly!(self, repeat, state, ui, type_registry);

        // show if we have a clip
        if let Some(clip) = &self.clip {
            ui.label(egui::RichText::new(format!("clip: {:?}", clip)).small());
        }
    }
}

#[derive(Component, Debug, Deref)]
pub struct SpawnOwned(Entity);

pub fn run(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut anims: Query<
        (Entity, &mut Anim, &BehaviorNode, Option<&BehaviorStarted>),
        BehaviorRunQuery,
    >,
    mut anim_players: Query<(Entity, &Name, Option<&mut AnimationPlayer>)>,
    mut scripts: ScriptQueries,
    equeries: EPathQueries,
) {
    for (entity, mut anim, node, started) in &mut anims {
        if started.is_some() {
            // reset eval properties
            anim.asset.value = BehaviorPropValue::None;
            anim.target.value = BehaviorPropValue::None;

            // remove previous clip
            anim.clip = None;
        }
        // keep working on eval properties
        else if anim.clip.is_none() {
            if let BehaviorPropValue::None = anim.asset.value {
                let result = anim.asset.fetch(node, &mut scripts);
                if let Some(Err(err)) = result {
                    error!("Script errored: {:?}", err);
                    commands.entity(entity).insert(BehaviorFailure);
                    continue;
                }
            }
            if let BehaviorPropValue::None = anim.target.value {
                let result = anim.target.fetch(node, &mut scripts);
                if let Some(Err(err)) = result {
                    error!("Script errored: {:?}", err);
                    commands.entity(entity).insert(BehaviorFailure);
                    continue;
                }
            }

            if let BehaviorPropValue::None = anim.repeat.value {
                let result = anim.repeat.fetch(node, &mut scripts);
                if let Some(Err(err)) = result {
                    error!("Script errored: {:?}", err);
                    commands.entity(entity).insert(BehaviorFailure);
                    continue;
                }
            }

            // if all eval properties are ready, assign anim clip to target
            if let (
                BehaviorPropValue::Some(anim_asset),
                BehaviorPropValue::Some(anim_target),
                BehaviorPropValue::Some(anim_repeat),
            ) = (&anim.asset.value, &anim.target.value, &anim.repeat.value)
            {
                let anim_target = anim_target.clone();
                let clip = asset_server.load(anim_asset.as_ref());

                let mut successes = 0;

                let targets = epath::select(None, &anim_target, &equeries);
                for target in &targets {
                    if let Ok((entity, _name, anim_player)) = anim_players.get_mut(target.entity) {
                        successes += 1;
                        if let Some(mut anim_player) = anim_player {
                            anim_player.start(clip.clone());
                            if *anim_repeat {
                                anim_player.repeat();
                            } else {
                                anim_player.stop_repeating();
                            }
                        } else {
                            let mut anim_player = AnimationPlayer::default();
                            anim_player.start(clip.clone());
                            if *anim_repeat {
                                anim_player.repeat();
                            }
                            commands.entity(entity).insert(anim_player);
                        }
                    } else {
                        warn!("Invalid anim target: {:?}", target);
                    }
                }

                anim.clip = Some(clip.clone());

                if successes == targets.len() {
                    commands.entity(entity).insert(BehaviorSuccess);
                } else {
                    commands.entity(entity).insert(BehaviorFailure);
                }
            }
        }
    }
}
