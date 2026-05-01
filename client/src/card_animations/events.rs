use bevy::prelude::*;

#[derive(Message, Clone, Debug, Default, PartialEq)]
pub struct PlacementRevealAnimReady;

#[derive(Message, Clone, Debug, Default, PartialEq)]
pub struct ObjectiveDestroyedAnimReady;

#[derive(Message, Clone, Debug, Default, PartialEq)]
pub struct DamageNumberSpawnRequested;

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

#[derive(Message, Clone, Debug, Default, PartialEq)]
pub struct TimerBarEaseRequested;

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
