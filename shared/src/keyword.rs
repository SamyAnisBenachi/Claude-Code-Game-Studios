use serde::{Deserialize, Serialize};

use crate::protocol::EntityId;
use crate::session::PlayerId;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeywordKind {
    FirstStrike,
    Haste,
    ChargeX,
    Range,
    Wall,
    Bodyguard,
    Irremovable,
    Untargetable,
    ResistanceX,
    VulnerabilityX,
    ArmorPiercing,
    Shield,
    Leader,
    Outnumbered,
    Repel,
    Attract,
    Teleport,
    ChangeLane,
    Death,
    FinalBlow,
    Counterattack,
    Injured,
    Appearance,
    StartOfTurn,
    EndOfTurn,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InjuredGrantedKeyword {
    FirstStrike,
    Counterattack,
    Range,
    Shield,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum KeywordPayload {
    ShieldConsumed,
    StunApplied {
        duration_rounds: u8,
    },
    SilenceApplied {
        duration_rounds: u8,
        stripped_keywords: Vec<KeywordKind>,
    },
    InjuredBonusActive {
        granted_keyword: InjuredGrantedKeyword,
    },
    LeaderSnapshotTaken {
        leader_unit_id: EntityId,
    },
    OutnumberedFlipped {
        player_id: PlayerId,
        active: bool,
    },
    BodyguardBondCreated {
        bodyguard_id: EntityId,
        protected_id: EntityId,
    },
    BodyguardBondBroken {
        bodyguard_id: EntityId,
    },
    CounterattackFired {
        target_id: EntityId,
    },
    HasteActivated,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DisplacementEvent {
    pub unit_id: EntityId,
    pub attacker_id: Option<EntityId>,
    pub from_lane: u8,
    pub from_cell: u8,
    pub to_lane: u8,
    pub to_cell: u8,
    pub kind: DisplacementKind,
    pub block_reason: Option<DisplacementBlockReason>,
    pub sub_step: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum DisplacementKind {
    Repel(u8),
    Attract(u8),
    Teleport { dest_lane: u8, dest_cell: u8 },
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DisplacementBlockReason {
    IrremovableKeyword,
    BoardEdgeClamped,
}
