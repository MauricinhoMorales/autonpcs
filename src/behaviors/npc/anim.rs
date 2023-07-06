use bevy::{prelude::*, reflect::TypeRegistry};
use bevy_inspector_egui::{egui, prelude::*};
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;
use simula_core::epath::{self};
use simula_script::{Script, ScriptContext};

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
        changed |= self.asset.ui(Some("asset"), state, ui, type_registry);
        changed |= self.target.ui(Some("target"), state, ui, type_registry);
        changed |= self.repeat.ui(Some("repeat"), state, ui, type_registry);
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
        self.target
            .ui_readonly(Some("target"), state, ui, type_registry);
        self.repeat
            .ui_readonly(Some("repeat"), state, ui, type_registry);

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
    // for handling scripts
    mut scripts: ResMut<Assets<Script>>,
    script_ctx_handles: Query<&Handle<ScriptContext>>,
    mut script_ctxs: ResMut<Assets<ScriptContext>>,
    // for handling epaths
    names: Query<&Name>,
    parents: Query<&Parent>,
    children: Query<&Children>,
    roots: Query<Entity, Without<Parent>>,
) {
    for (entity, mut anim, node, started) in &mut anims {
        if started.is_some() {
            // reset eval properties
            anim.asset.value = BehaviorPropValue::None;
            anim.target.value = BehaviorPropValue::None;

            // remove previous clip
            anim.clip = None;
        } else {
            // if we have an anim clip, we're done
            if anim.clip.is_some() {
                commands.entity(entity).insert(BehaviorSuccess);
            }
            // else still working on eval properties and assigning clip
            else {
                // keep working on eval properties
                if let BehaviorPropValue::None = anim.asset.value {
                    let result =
                        anim.asset
                            .fetch(node, &mut scripts, &script_ctx_handles, &mut script_ctxs);
                    if let Some(Err(err)) = result {
                        error!("Script errored: {:?}", err);
                        commands.entity(entity).insert(BehaviorFailure);
                        continue;
                    }
                }
                if let BehaviorPropValue::None = anim.target.value {
                    let result = anim.target.fetch(
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

                if let BehaviorPropValue::None = anim.repeat.value {
                    let result = anim.repeat.fetch(
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

                // if all eval properties are ready, assign anim clip to target
                if let (
                    BehaviorPropValue::Some(anim_asset),
                    BehaviorPropValue::Some(anim_target),
                    BehaviorPropValue::Some(anim_repeat),
                ) = (&anim.asset.value, &anim.target.value, &anim.repeat.value)
                {
                    let mut success = false;
                    let clip = asset_server.load(anim_asset.as_ref());
                    if let Some(anim_target) =
                        epath::select(None, anim_target, &names, &parents, &children, &roots)
                            .first()
                    {
                        if let Ok((entity, _name, anim_player)) =
                            anim_players.get_mut(anim_target.entity)
                        {
                            success = true;
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
                        }
                    }

                    if success {
                        anim.clip = Some(clip.clone());
                        commands.entity(entity).insert(BehaviorSuccess);
                    } else {
                        error!("Anim target not found: {:?}", anim_target);
                        commands.entity(entity).insert(BehaviorFailure);
                    }
                }
            }
        }
    }
}
