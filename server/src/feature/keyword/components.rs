use bevy::prelude::*;

/// Runtime keyword state co-located on each board unit entity.
///
/// BODYGUARD stores a Bevy entity handle because the bond is session-local and
/// must survive lane changes. Protocol snapshots must translate it to EntityId.
#[derive(Component, Clone, Debug, Default)]
pub struct UnitKeywordState {
    pub shield_active: bool,
    pub stun_active: bool,
    pub silenced_until_round: Option<u32>,
    pub leader_bonus_atk: u8,
    pub leader_bonus_hp: u8,
    pub bodyguard_protects: Option<Entity>,
    pub outnumbered_active: bool,
}
