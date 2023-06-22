use bevy::{ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;
use simula_behavior_macro::BehaviorFactory;
use spawn::Spawn;

mod spawn;

pub struct NPCBehaviorPlugin;

impl Plugin for NPCBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(BehaviorTreePlugin::<NPCBehavior>::default())
            .add_system(spawn::run)
            .add_system(
                spawn::removed
                    .in_base_set(CoreSet::PostUpdate)
                    .after(BehaviorSet::PostUpdate),
            )
            .add_system(subtree::run::<NPCBehavior>) // Subtrees are typed, need to register them separately
            .register_type::<Subtree<NPCBehavior>>();
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct NPCBehaviorAttributes {
    pub pos: Vec2,
}

#[derive(Serialize, Deserialize, TypeUuid, Debug, Clone, Reflect, FromReflect, BehaviorFactory)]
#[uuid = "B814382F-645F-401E-A884-E595E51E200E"]
#[BehaviorAttributes(NPCBehaviorAttributes)]
pub enum NPCBehavior {
    Debug(Debug),
    Selector(Selector),
    Sequencer(Sequencer),
    All(All),
    Any(Any),
    Repeater(Repeater),
    Inverter(Inverter),
    Succeeder(Succeeder),
    Wait(Wait),
    Delay(Delay),
    Guard(Guard),
    Timeout(Timeout),

    Spawn(Spawn),

    Subtree(Subtree<NPCBehavior>), // Substrees are typed, this loads same tree type
}

impl Default for NPCBehavior {
    fn default() -> Self {
        Self::Debug(Debug::default())
    }
}

impl BehaviorNodeInspectable<NPCBehavior> for NPCBehaviorAttributes {
    fn set_pos(&mut self, pos: Vec2) {
        self.pos = pos;
    }

    fn get_pos(&self) -> Vec2 {
        self.pos
    }
}

impl BehaviorInspectable for NPCBehavior {
    fn color(&self) -> Color {
        match self {
            NPCBehavior::Debug(_) => Color::hex("#235").unwrap(),
            NPCBehavior::Selector(_) => Color::hex("#522").unwrap(),
            NPCBehavior::Sequencer(_) => Color::hex("#252").unwrap(),
            NPCBehavior::All(_) => Color::hex("#252").unwrap(),
            NPCBehavior::Any(_) => Color::hex("#522").unwrap(),
            NPCBehavior::Repeater(_) => Color::hex("#440").unwrap(),
            NPCBehavior::Inverter(_) => Color::hex("#440").unwrap(),
            NPCBehavior::Succeeder(_) => Color::hex("#440").unwrap(),
            NPCBehavior::Wait(_) => Color::hex("#235").unwrap(),
            NPCBehavior::Delay(_) => Color::hex("#440").unwrap(),
            NPCBehavior::Guard(_) => Color::hex("#440").unwrap(),
            NPCBehavior::Timeout(_) => Color::hex("#440").unwrap(),

            NPCBehavior::Spawn(_) => Color::hex("#FFA500").unwrap(),

            NPCBehavior::Subtree(_) => Color::hex("#440").unwrap(),
        }
    }

    #[rustfmt::skip]
    fn categories(&self) -> Vec<&'static str> {
        match self {
            NPCBehavior::Debug(_) => vec![<Debug as BehaviorInfo>::TYPE.as_ref()],
            NPCBehavior::Selector(_) => vec![<Selector as BehaviorInfo>::TYPE.as_ref()],
            NPCBehavior::Sequencer(_) => vec![<Sequencer as BehaviorInfo>::TYPE.as_ref()],
            NPCBehavior::All(_) => vec![<All as BehaviorInfo>::TYPE.as_ref()],
            NPCBehavior::Any(_) => vec![<Any as BehaviorInfo>::TYPE.as_ref()],
            NPCBehavior::Repeater(_) => vec![<Repeater as BehaviorInfo>::TYPE.as_ref()],
            NPCBehavior::Inverter(_) => vec![<Inverter as BehaviorInfo>::TYPE.as_ref()],
            NPCBehavior::Succeeder(_) => vec![<Succeeder as BehaviorInfo>::TYPE.as_ref()],
            NPCBehavior::Wait(_) => vec![<Wait as BehaviorInfo>::TYPE.as_ref()],
            NPCBehavior::Delay(_) => vec![<Delay as BehaviorInfo>::TYPE.as_ref()],
            NPCBehavior::Guard(_) => vec![<Guard as BehaviorInfo>::TYPE.as_ref()],
            NPCBehavior::Timeout(_) => vec![<Timeout as BehaviorInfo>::TYPE.as_ref()],

            NPCBehavior::Spawn(_) => vec![<Spawn as BehaviorInfo>::TYPE.as_ref(), "NPC"],

            NPCBehavior::Subtree(_) => vec![<Subtree<NPCBehavior> as BehaviorInfo>::TYPE.as_ref()],
        }
    }
}
