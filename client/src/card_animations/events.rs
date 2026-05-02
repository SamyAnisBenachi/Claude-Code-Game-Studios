use bevy::prelude::*;

#[derive(Message, Clone, Debug, Default, PartialEq)]
pub struct PlacementRevealAnimReady;

#[derive(Message, Clone, Debug, Default, PartialEq)]
pub struct ObjectiveDestroyedAnimReady;

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DamageNumberSpawnRequested {
    pub target: Entity,
    pub damage_value: u32,
    pub event_id: u32,
}

impl Default for DamageNumberSpawnRequested {
    fn default() -> Self {
        Self {
            target: Entity::PLACEHOLDER,
            damage_value: 0,
            event_id: 0,
        }
    }
}

#[derive(Message, Clone, Debug, Default, PartialEq)]
pub struct BoardRebuildRequested;

#[derive(Message, Clone, Debug, Default, PartialEq)]
pub struct PlacementCancelAllAnimsRequested;

#[derive(Message, Clone, Debug, Default, PartialEq)]
pub struct CardAcquiredAnimReady;

#[derive(Message, Clone, Debug, Default, PartialEq)]
pub struct SnapBackRequested;

#[derive(Message, Clone, Debug, Default, PartialEq)]
pub struct HandHideRequested;

#[derive(Message, Clone, Debug, Default, PartialEq)]
pub struct HandShowRequested;

#[derive(Message, Clone, Debug, Default, PartialEq)]
pub struct AuctionPanelTransitionRequested;

#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub struct TimerBarEaseRequested {
    pub target_width_percent: f32,
}

impl Default for TimerBarEaseRequested {
    fn default() -> Self {
        Self {
            target_width_percent: 100.0,
        }
    }
}

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HandCardDragStarted {
    pub card: Entity,
}

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HandCardHoverEntered {
    pub card: Entity,
}

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HandCardHoverExited {
    pub card: Entity,
}

#[derive(Message, Clone, Debug, Default, PartialEq)]
pub struct GoldTickRequested;

#[derive(Message, Clone, Debug, Default, PartialEq)]
pub struct SettlementOverlayRequested;

#[derive(Message, Clone, Debug, Default, PartialEq)]
pub struct DisplacementAnimRequested;

#[derive(Message, Clone, Debug, Default, PartialEq)]
pub struct TrapFlipRequested;

#[derive(Message, Clone, Debug, Default, PartialEq)]
pub struct AuraPulseRequested;

#[derive(Message, Clone, Debug, Default, PartialEq)]
pub struct GroupDrainedSignal;
