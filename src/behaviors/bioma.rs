use bevy::{ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;
use simula_behavior_macro::BehaviorFactory;

pub struct BiomaBehaviorPlugin;

impl Plugin for BiomaBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(BehaviorTreePlugin::<BiomaBehavior>::default())
            .add_system(subtree::run::<BiomaBehavior>) // Subtrees are typed, need to register them separately
            .register_type::<Subtree<BiomaBehavior>>();
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct BiomaBehaviorAttributes {
    pub pos: Vec2,
}

#[derive(Serialize, Deserialize, TypeUuid, Debug, Clone, Reflect, FromReflect, BehaviorFactory)]
#[uuid = "57178605-8BDA-48D9-B1B9-414E6D142663"]
#[BehaviorAttributes(BiomaBehaviorAttributes)]
pub enum BiomaBehavior {
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

    Subtree(Subtree<BiomaBehavior>), // Substrees are typed, this loads same tree type
}

impl Default for BiomaBehavior {
    fn default() -> Self {
        Self::Debug(Debug::default())
    }
}

impl BehaviorNodeInspectable<BiomaBehavior> for BiomaBehaviorAttributes {
    fn set_pos(&mut self, pos: Vec2) {
        self.pos = pos;
    }

    fn get_pos(&self) -> Vec2 {
        self.pos
    }
}

impl BehaviorInspectable for BiomaBehavior {
    fn color(&self) -> Color {
        match self {
            BiomaBehavior::Debug(_) => Color::hex("#235").unwrap(),
            BiomaBehavior::Selector(_) => Color::hex("#522").unwrap(),
            BiomaBehavior::Sequencer(_) => Color::hex("#252").unwrap(),
            BiomaBehavior::All(_) => Color::hex("#252").unwrap(),
            BiomaBehavior::Any(_) => Color::hex("#522").unwrap(),
            BiomaBehavior::Repeater(_) => Color::hex("#440").unwrap(),
            BiomaBehavior::Inverter(_) => Color::hex("#440").unwrap(),
            BiomaBehavior::Succeeder(_) => Color::hex("#440").unwrap(),
            BiomaBehavior::Wait(_) => Color::hex("#235").unwrap(),
            BiomaBehavior::Delay(_) => Color::hex("#440").unwrap(),
            BiomaBehavior::Guard(_) => Color::hex("#440").unwrap(),
            BiomaBehavior::Timeout(_) => Color::hex("#440").unwrap(),
            BiomaBehavior::Subtree(_) => Color::hex("#440").unwrap(),
        }
    }

    #[rustfmt::skip]
    fn categories(&self) -> Vec<&'static str> {
        match self {
            BiomaBehavior::Debug(_) => vec![<Debug as BehaviorInfo>::TYPE.as_ref()],
            BiomaBehavior::Selector(_) => vec![<Selector as BehaviorInfo>::TYPE.as_ref()],
            BiomaBehavior::Sequencer(_) => vec![<Sequencer as BehaviorInfo>::TYPE.as_ref()],
            BiomaBehavior::All(_) => vec![<All as BehaviorInfo>::TYPE.as_ref()],
            BiomaBehavior::Any(_) => vec![<Any as BehaviorInfo>::TYPE.as_ref()],
            BiomaBehavior::Repeater(_) => vec![<Repeater as BehaviorInfo>::TYPE.as_ref()],
            BiomaBehavior::Inverter(_) => vec![<Inverter as BehaviorInfo>::TYPE.as_ref()],
            BiomaBehavior::Succeeder(_) => vec![<Succeeder as BehaviorInfo>::TYPE.as_ref()],
            BiomaBehavior::Wait(_) => vec![<Wait as BehaviorInfo>::TYPE.as_ref()],
            BiomaBehavior::Delay(_) => vec![<Delay as BehaviorInfo>::TYPE.as_ref()],
            BiomaBehavior::Guard(_) => vec![<Guard as BehaviorInfo>::TYPE.as_ref()],
            BiomaBehavior::Timeout(_) => vec![<Timeout as BehaviorInfo>::TYPE.as_ref()],
            BiomaBehavior::Subtree(_) => vec![<Subtree<BiomaBehavior> as BehaviorInfo>::TYPE.as_ref()],
        }
    }
}
