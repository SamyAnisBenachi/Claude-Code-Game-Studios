use std::{collections::HashMap, time::Duration};

use bevy::prelude::*;
use bevy_tweening::TweenAnim;
use lightyear::prelude::{MessageReceiver, MessageSender};
use shared::card::{CardId, ClassId};
use shared::keyword::InjuredGrantedKeyword;
use shared::protocol::{
    C2SRequestSnapshot, EntityId, ObjectiveSnapshot, PlacedCardReveal, PlayTarget, ReliableChannel,
    RoundPhase, S2CGameSnapshot, S2CJoinAck, S2CPlacementReveal, S2CResolutionEvent,
    S2CRoomCreated, S2CSlotUpdated, SessionSlot, UnitBoardLocation, UnitBoardState,
    UnitStatsSnapshot,
};
use shared::session::PlayerId;

use super::PresentationSet;
use crate::card_animations::{
    cancel_tween_anim_in_place, AnimQueue, BoardRebuildRequested, PendingObjectiveDestroyedEvents,
    PendingPhaseChange, PlacementRevealAnimReady, PlacementRevealEntry, StagedObjectiveRevealQueue,
};
use crate::state::{ClientGameSnapshotMessage, ClientState, CurrentClientPhase};
use crate::ui::hand::{
    GhostClickedEvent, GhostDragStartEvent, GhostPlacementChanged, ObjectiveCell,
    PlacementTargetUnit,
};
use crate::ui::shared::{BoardLayout, LaneCell, BOARD_CELL_COUNT, BOARD_LANE_COUNT};

pub mod rendering_constants;

pub const UNIT_PLACEHOLDER_FRAME_INDEX: usize = 0;
pub const HP_BAR_WHITE_PIXEL_FRAME_INDEX: usize = 1;
pub const OBJECTIVE_UNKNOWN_FRAME_INDEX: usize = 0;
pub const STATUS_ICON_SHIELD_FRAME_INDEX: usize = 10;
pub const STATUS_ICON_STUN_FRAME_INDEX: usize = 11;
pub const STATUS_ICON_SILENCE_FRAME_INDEX: usize = 12;
pub const STATUS_ICON_INJURED_FRAME_INDEX: usize = 13;
pub const STATUS_ICON_LEADER_FRAME_INDEX: usize = 14;
pub const STATUS_ICON_HASTE_FRAME_INDEX: usize = 15;
pub const STATUS_ICON_BODYGUARD_FRAME_INDEX: usize = 16;
pub const STATUS_ICON_OUTNUMBERED_FRAME_INDEX: usize = 17;
pub const STATUS_ICON_INJURED_GRANTED_FRAME_INDEX: usize = 18;
pub const STATUS_OVERFLOW_BADGE_FRAME_INDEX: usize = 19;
pub const HP_THRESHOLD_EPSILON: f32 = 1e-4;
pub const DEFAULT_CO_OCCUPANCY_SIDE_OFFSET: f32 = 8.0;
pub const MIN_CO_OCCUPANCY_SIDE_OFFSET: f32 = 4.0;
pub const MAX_CO_OCCUPANCY_SIDE_OFFSET: f32 = 16.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitAtlasFrame {
    pub frame_index: usize,
    pub max_hp: u8,
}

#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct CardAtlas {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub board_elements_image: Handle<Image>,
    pub board_elements_layout: Handle<TextureAtlasLayout>,
    pub unit_frames: HashMap<CardId, UnitAtlasFrame>,
}

impl CardAtlas {
    pub fn with_unit_frame(mut self, card_id: CardId, frame_index: usize, max_hp: u8) -> Self {
        self.unit_frames.insert(
            card_id,
            UnitAtlasFrame {
                frame_index,
                max_hp,
            },
        );
        self
    }

    pub fn unit_frame(&self, card_id: CardId) -> Option<UnitAtlasFrame> {
        self.unit_frames.get(&card_id).copied()
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardRenderingEntity;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardCamera;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardCellNode;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardSnapshotEntity;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardUnit {
    pub unit_id: EntityId,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardUnitOwner(pub PlayerId);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardUnitCard {
    pub card_id: Option<CardId>,
    pub frame_index: usize,
    pub used_missing_art_fallback: bool,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardUnitStats {
    pub hp_current: u8,
    pub hp_max: u8,
    pub atk: u8,
    pub mp: u8,
    pub ar: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardUnitSourceClass(pub ClassId);

#[derive(Component, Debug, Clone, PartialEq, Eq, Default)]
pub struct StatusEffectsList {
    effects: Vec<StatusEffectVisual>,
}

impl StatusEffectsList {
    pub fn new(effects: Vec<StatusEffectVisual>) -> Self {
        Self { effects }
    }

    pub fn effects(&self) -> &[StatusEffectVisual] {
        &self.effects
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusEffectVisual {
    pub key: StatusEffectKey,
    pub remaining_duration_sort_key: u16,
}

impl StatusEffectVisual {
    pub fn untimed(key: StatusEffectKey) -> Self {
        Self {
            key,
            remaining_duration_sort_key: 0,
        }
    }

    pub fn timed(key: StatusEffectKey, remaining_duration_sort_key: u16) -> Self {
        Self {
            key,
            remaining_duration_sort_key,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusEffectKey {
    Shield,
    Stun,
    Silence,
    Injured,
    Leader,
    Haste,
    Bodyguard,
    Outnumbered,
    InjuredGranted(InjuredGrantedKeyword),
}

impl StatusEffectKey {
    fn deterministic_sort_key(self) -> u16 {
        match self {
            Self::Shield => 0,
            Self::Stun => 1,
            Self::Silence => 2,
            Self::Injured => 3,
            Self::Leader => 4,
            Self::Haste => 5,
            Self::Bodyguard => 6,
            Self::Outnumbered => 7,
            Self::InjuredGranted(keyword) => 100 + injured_granted_sort_key(keyword),
        }
    }
}

fn injured_granted_sort_key(keyword: InjuredGrantedKeyword) -> u16 {
    match keyword {
        InjuredGrantedKeyword::FirstStrike => 0,
        InjuredGrantedKeyword::Counterattack => 1,
        InjuredGrantedKeyword::Range => 2,
        InjuredGrantedKeyword::Shield => 3,
    }
}

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct StatusDisplayDefinitions {
    definitions: HashMap<StatusEffectKey, StatusDisplayDefinition>,
}

impl Default for StatusDisplayDefinitions {
    fn default() -> Self {
        let mut definitions = HashMap::new();
        definitions.insert(
            StatusEffectKey::Shield,
            StatusDisplayDefinition::new(
                1,
                STATUS_ICON_SHIELD_FRAME_INDEX,
                Color::srgba(0.42, 0.78, 1.0, 1.0),
            ),
        );
        definitions.insert(
            StatusEffectKey::Stun,
            StatusDisplayDefinition::new(
                2,
                STATUS_ICON_STUN_FRAME_INDEX,
                Color::srgba(1.0, 0.84, 0.22, 1.0),
            ),
        );
        definitions.insert(
            StatusEffectKey::Silence,
            StatusDisplayDefinition::new(
                2,
                STATUS_ICON_SILENCE_FRAME_INDEX,
                Color::srgba(0.72, 0.72, 0.82, 1.0),
            ),
        );
        definitions.insert(
            StatusEffectKey::Injured,
            StatusDisplayDefinition::new(
                2,
                STATUS_ICON_INJURED_FRAME_INDEX,
                Color::srgba(1.0, 0.28, 0.28, 1.0),
            ),
        );
        definitions.insert(
            StatusEffectKey::Leader,
            StatusDisplayDefinition::new(
                2,
                STATUS_ICON_LEADER_FRAME_INDEX,
                Color::srgba(0.85, 0.68, 1.0, 1.0),
            ),
        );
        definitions.insert(
            StatusEffectKey::Haste,
            StatusDisplayDefinition::new(
                2,
                STATUS_ICON_HASTE_FRAME_INDEX,
                Color::srgba(0.44, 1.0, 0.62, 1.0),
            ),
        );
        definitions.insert(
            StatusEffectKey::Bodyguard,
            StatusDisplayDefinition::new(
                2,
                STATUS_ICON_BODYGUARD_FRAME_INDEX,
                Color::srgba(0.66, 0.88, 1.0, 1.0),
            ),
        );
        definitions.insert(
            StatusEffectKey::Outnumbered,
            StatusDisplayDefinition::new(
                2,
                STATUS_ICON_OUTNUMBERED_FRAME_INDEX,
                Color::srgba(1.0, 0.52, 0.28, 1.0),
            ),
        );

        for keyword in [
            InjuredGrantedKeyword::FirstStrike,
            InjuredGrantedKeyword::Counterattack,
            InjuredGrantedKeyword::Range,
            InjuredGrantedKeyword::Shield,
        ] {
            definitions.insert(
                StatusEffectKey::InjuredGranted(keyword),
                StatusDisplayDefinition::new(
                    2,
                    STATUS_ICON_INJURED_GRANTED_FRAME_INDEX,
                    Color::srgba(1.0, 0.36, 0.46, 1.0),
                ),
            );
        }

        Self { definitions }
    }
}

impl StatusDisplayDefinitions {
    pub fn definition(&self, key: StatusEffectKey) -> Option<&StatusDisplayDefinition> {
        self.definitions.get(&key)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatusDisplayDefinition {
    pub display_tier: u8,
    pub icon_frame_index: usize,
    pub tint: Color,
}

impl StatusDisplayDefinition {
    pub fn new(display_tier: u8, icon_frame_index: usize, tint: Color) -> Self {
        Self {
            display_tier,
            icon_frame_index,
            tint,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusIcon {
    pub key: StatusEffectKey,
    pub slot: u8,
    pub display_tier: u8,
    pub remaining_duration_sort_key: u16,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusOverflowBadge {
    pub slot: u8,
    pub hidden_count: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandingObjective {
    pub owner_id: PlayerId,
    pub lane: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandingObjectiveHp {
    pub hp_current: u8,
    pub hp_max: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HpBarBackground;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HpBarFill;

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct ObjectiveIdentityCache {
    identities: HashMap<(PlayerId, u8), bool>,
}

impl ObjectiveIdentityCache {
    pub fn insert(&mut self, player_id: PlayerId, lane: u8, is_fake: bool) {
        self.identities.insert((player_id, lane), is_fake);
    }

    pub fn clear(&mut self) {
        self.identities.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.identities.is_empty()
    }
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct PlayerTeamMap {
    teams: HashMap<PlayerId, u8>,
}

impl PlayerTeamMap {
    pub fn apply_slots(&mut self, slots: &[SessionSlot]) {
        self.teams.clear();
        for slot in slots {
            if let Some(player_id) = slot.player_id {
                self.teams.insert(player_id, slot.team);
            }
        }
    }

    pub fn insert(&mut self, player_id: PlayerId, team: u8) {
        self.teams.insert(player_id, team);
    }

    pub fn team_for(&self, player_id: PlayerId) -> Option<u8> {
        self.teams.get(&player_id).copied()
    }

    pub fn clear(&mut self) {
        self.teams.clear();
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct BoardRenderingConfig {
    pub health_bar_green_threshold: f32,
    pub health_bar_red_threshold: f32,
    pub co_occupancy_side_offset: f32,
}

impl Default for BoardRenderingConfig {
    fn default() -> Self {
        Self {
            health_bar_green_threshold: 0.6,
            health_bar_red_threshold: 0.3,
            co_occupancy_side_offset: DEFAULT_CO_OCCUPANCY_SIDE_OFFSET,
        }
    }
}

impl BoardRenderingConfig {
    pub fn assert_valid(self) {
        assert!(
            self.health_bar_red_threshold < self.health_bar_green_threshold,
            "HP threshold config invalid: red_threshold={} >= green_threshold={}",
            self.health_bar_red_threshold,
            self.health_bar_green_threshold
        );
        assert!(
            (MIN_CO_OCCUPANCY_SIDE_OFFSET..=MAX_CO_OCCUPANCY_SIDE_OFFSET)
                .contains(&self.co_occupancy_side_offset),
            "co_occupancy_side_offset={} outside allowed range {}..={}",
            self.co_occupancy_side_offset,
            MIN_CO_OCCUPANCY_SIDE_OFFSET,
            MAX_CO_OCCUPANCY_SIDE_OFFSET
        );
    }
}

pub const DEFAULT_RESOLUTION_REVEAL_TIMEOUT_MS: u64 = 2_000;
pub const MIN_RESOLUTION_REVEAL_TIMEOUT_MS: u64 = 1_500;
pub const MAX_RESOLUTION_REVEAL_TIMEOUT_MS: u64 = 5_000;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardRevealTimingConfig {
    pub resolution_reveal_timeout_ms: u64,
}

impl Default for BoardRevealTimingConfig {
    fn default() -> Self {
        Self {
            resolution_reveal_timeout_ms: DEFAULT_RESOLUTION_REVEAL_TIMEOUT_MS,
        }
    }
}

impl BoardRevealTimingConfig {
    pub fn assert_valid(self) {
        assert!(
            (MIN_RESOLUTION_REVEAL_TIMEOUT_MS..=MAX_RESOLUTION_REVEAL_TIMEOUT_MS)
                .contains(&self.resolution_reveal_timeout_ms),
            "resolution_reveal_timeout_ms={} outside allowed range {}..={}",
            self.resolution_reveal_timeout_ms,
            MIN_RESOLUTION_REVEAL_TIMEOUT_MS,
            MAX_RESOLUTION_REVEAL_TIMEOUT_MS
        );
    }

    fn timeout(self) -> Duration {
        Duration::from_millis(self.resolution_reveal_timeout_ms)
    }
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardLocalPlayer {
    pub player_id: Option<PlayerId>,
}

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnapshotRecoveryRequested {
    pub reason: SnapshotRecoveryReason,
}

impl SnapshotRecoveryRequested {
    fn resolution_reveal_stuck() -> Self {
        Self {
            reason: SnapshotRecoveryReason::ResolutionRevealStuck,
        }
    }

    fn pending_resolution_script_stuck() -> Self {
        Self {
            reason: SnapshotRecoveryReason::PendingResolutionScriptStuck,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotRecoveryReason {
    ResolutionRevealStuck,
    PendingResolutionScriptStuck,
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct PendingResolutionScript {
    script: Option<S2CResolutionEvent>,
    elapsed_without_reveal: Duration,
    recovery_requested: bool,
}

impl PendingResolutionScript {
    pub fn set(&mut self, script: S2CResolutionEvent) {
        self.script = Some(script);
        self.elapsed_without_reveal = Duration::ZERO;
        self.recovery_requested = false;
    }

    pub fn clear(&mut self) {
        self.script = None;
        self.elapsed_without_reveal = Duration::ZERO;
        self.recovery_requested = false;
    }

    pub fn is_some(&self) -> bool {
        self.script.is_some()
    }

    pub fn script(&self) -> Option<&S2CResolutionEvent> {
        self.script.as_ref()
    }

    fn tick_without_reveal(&mut self, delta: Duration, timeout: Duration) -> bool {
        if self.script.is_none() || self.recovery_requested {
            return false;
        }

        self.elapsed_without_reveal += delta;
        if self.elapsed_without_reveal >= timeout {
            self.recovery_requested = true;
            return true;
        }

        false
    }
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolutionRevealWait {
    active: bool,
    elapsed: Duration,
    recovery_requested: bool,
}

impl ResolutionRevealWait {
    pub fn start(&mut self) {
        self.active = true;
        self.elapsed = Duration::ZERO;
        self.recovery_requested = false;
    }

    pub fn clear(&mut self) {
        self.active = false;
        self.elapsed = Duration::ZERO;
        self.recovery_requested = false;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    fn tick(&mut self, delta: Duration, timeout: Duration) -> bool {
        if !self.active || self.recovery_requested {
            return false;
        }

        self.elapsed += delta;
        if self.elapsed >= timeout {
            self.recovery_requested = true;
            return true;
        }

        false
    }
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct PlacementRevealCollectState {
    pending: Option<PendingPlacementRevealBatch>,
}

impl PlacementRevealCollectState {
    pub fn start_from_reveal(
        &mut self,
        reveal: &S2CPlacementReveal,
        local_player_id: Option<PlayerId>,
    ) -> usize {
        let targets = reveal_targets(reveal, local_player_id);
        self.start(targets)
    }

    pub fn start_from_reveals(
        &mut self,
        reveals: &[S2CPlacementReveal],
        local_player_id: Option<PlayerId>,
    ) -> usize {
        let targets = reveals
            .iter()
            .flat_map(|reveal| reveal_targets(reveal, local_player_id))
            .collect::<Vec<_>>();
        self.start(targets)
    }

    pub fn is_pending(&self) -> bool {
        self.pending.is_some()
    }

    pub fn pending_target_count(&self) -> usize {
        self.pending
            .as_ref()
            .map(|pending| pending.targets.len())
            .unwrap_or(0)
    }

    pub fn clear(&mut self) {
        self.pending = None;
    }

    fn start(&mut self, targets: Vec<PlacementRevealTarget>) -> usize {
        let count = targets.len();
        self.pending = Some(PendingPlacementRevealBatch {
            targets,
            frames_until_collect: 1,
        });
        count
    }

    fn take_ready_targets(&mut self) -> Option<Vec<PlacementRevealTarget>> {
        let pending = self.pending.as_mut()?;
        if pending.frames_until_collect > 0 {
            pending.frames_until_collect -= 1;
            return None;
        }

        self.pending.take().map(|pending| pending.targets)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingPlacementRevealBatch {
    targets: Vec<PlacementRevealTarget>,
    frames_until_collect: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PlacementRevealTarget {
    owner_id: PlayerId,
    card_id: CardId,
    lane: u8,
    cell: u8,
}

fn reveal_targets(
    reveal: &S2CPlacementReveal,
    local_player_id: Option<PlayerId>,
) -> Vec<PlacementRevealTarget> {
    reveal
        .placements
        .iter()
        .filter_map(|placement| reveal_target(placement, local_player_id))
        .collect()
}

fn reveal_target(
    placement: &PlacedCardReveal,
    local_player_id: Option<PlayerId>,
) -> Option<PlacementRevealTarget> {
    let local_player_id = local_player_id?;
    if placement.owner_id == local_player_id {
        return None;
    }

    let PlayTarget::BoardCell { lane, cell } = placement.target else {
        return None;
    };

    Some(PlacementRevealTarget {
        owner_id: placement.owner_id,
        card_id: placement.card_id,
        lane,
        cell,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HpBarColor {
    Green,
    Yellow,
    Red,
}

impl HpBarColor {
    pub fn tint(self) -> Color {
        match self {
            Self::Green => Color::srgba(0.2, 0.92, 0.38, 1.0),
            Self::Yellow => Color::srgba(1.0, 0.78, 0.18, 1.0),
            Self::Red => Color::srgba(0.95, 0.18, 0.16, 1.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HpBarVisual {
    pub fill: f32,
    pub color: HpBarColor,
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardRenderState {
    #[default]
    Idle,
    Lobby,
    DraftInitial,
    DraftShop,
    DraftAuction,
    Placement,
    Resolution,
    ResolutionReveal,
    ResolutionExecuting,
    GameOver,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoardRenderSet {
    ReadMessages,
    ResolveStateMachine,
    SpawnEntities,
    ScheduleTweens,
    UpdateHpBars,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GhostUnit {
    pub card_id: CardId,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetUnitGhost {
    pub card_id: CardId,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectiveTargetGhost {
    pub card_id: CardId,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaneGhostWash {
    pub card_id: CardId,
    pub lane: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardGhostInteraction {
    pub card_id: CardId,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardGhostPickable;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpawnHighlightState {
    #[default]
    Inactive,
    ValidSpawn,
}

impl SpawnHighlightState {
    pub fn tint(self) -> Color {
        match self {
            Self::Inactive => Color::srgba(0.12, 0.24, 0.30, 0.55),
            Self::ValidSpawn => Color::srgba(1.0, 0.82, 0.24, 0.88),
        }
    }
}

pub struct BoardRenderingPlugin;

impl Plugin for BoardRenderingPlugin {
    fn build(&self, app: &mut App) {
        BoardRenderingConfig::default().assert_valid();
        BoardRevealTimingConfig::default().assert_valid();

        app.init_state::<ClientState>()
            .init_resource::<CurrentClientPhase>()
            .init_resource::<BoardRenderingConfig>()
            .init_resource::<BoardRevealTimingConfig>()
            .init_resource::<BoardRenderState>()
            .init_resource::<BoardLocalPlayer>()
            .init_resource::<ObjectiveIdentityCache>()
            .init_resource::<PlayerTeamMap>()
            .init_resource::<StatusDisplayDefinitions>()
            .init_resource::<PlacementRevealCollectState>()
            .init_resource::<PendingResolutionScript>()
            .init_resource::<ResolutionRevealWait>()
            .add_message::<ClientGameSnapshotMessage>()
            .add_message::<BoardRebuildRequested>()
            .add_message::<PlacementRevealAnimReady>()
            .add_message::<SnapshotRecoveryRequested>()
            .add_message::<GhostPlacementChanged>()
            .add_message::<GhostClickedEvent>()
            .add_message::<GhostDragStartEvent>()
            .add_message::<Pointer<Click>>()
            .add_message::<Pointer<Press>>()
            .configure_sets(
                Update,
                (
                    BoardRenderSet::ReadMessages,
                    BoardRenderSet::ResolveStateMachine,
                    BoardRenderSet::SpawnEntities,
                    BoardRenderSet::ScheduleTweens,
                    BoardRenderSet::UpdateHpBars,
                )
                    .chain()
                    .run_if(in_state(ClientState::InSession)),
            )
            .configure_sets(
                Update,
                (
                    BoardRenderSet::ReadMessages.in_set(PresentationSet::MessageDrain),
                    BoardRenderSet::ResolveStateMachine.in_set(PresentationSet::StateSync),
                    BoardRenderSet::SpawnEntities.in_set(PresentationSet::StateSync),
                    BoardRenderSet::ScheduleTweens.in_set(PresentationSet::StateSync),
                    BoardRenderSet::UpdateHpBars.in_set(PresentationSet::StateSync),
                ),
            )
            .add_systems(
                OnEnter(ClientState::InSession),
                insert_board_rendering_session_resources,
            )
            .add_systems(
                OnExit(ClientState::InSession),
                remove_board_rendering_session_resources,
            )
            .add_systems(
                Update,
                (
                    sync_reveal_state_from_snapshot_system.in_set(BoardRenderSet::ReadMessages),
                    rebuild_board_from_snapshot_system.in_set(BoardRenderSet::ReadMessages),
                    drain_resolution_event_system.in_set(BoardRenderSet::ReadMessages),
                    tick_reveal_recovery_timeouts_system
                        .in_set(BoardRenderSet::ResolveStateMachine),
                    (
                        collect_placement_reveal_batch_system,
                        send_snapshot_recovery_requests_system,
                    )
                        .chain()
                        .in_set(BoardRenderSet::ScheduleTweens),
                    update_hp_bars_system.in_set(BoardRenderSet::UpdateHpBars),
                    update_status_icons_system.in_set(BoardRenderSet::UpdateHpBars),
                ),
            )
            .add_systems(Update, drain_player_team_map_messages_system)
            .add_systems(
                Update,
                (
                    apply_ghost_placement_changed_system,
                    emit_ghost_drag_start_events_system,
                    emit_ghost_clicked_events_system,
                    drain_placement_reveal_system,
                )
                    .chain()
                    .in_set(PresentationSet::MessageDrain)
                    .run_if(in_state(ClientState::InSession)),
            );
    }
}

pub fn drain_player_team_map_messages_system(
    mut created_receivers: Query<&mut MessageReceiver<S2CRoomCreated>>,
    mut join_receivers: Query<&mut MessageReceiver<S2CJoinAck>>,
    mut slot_receivers: Query<&mut MessageReceiver<S2CSlotUpdated>>,
    mut player_team_map: ResMut<PlayerTeamMap>,
) {
    for mut receiver in &mut created_receivers {
        for message in receiver.receive() {
            player_team_map.apply_slots(&message.slots);
        }
    }

    for mut receiver in &mut join_receivers {
        for message in receiver.receive() {
            player_team_map.apply_slots(&message.slots);
        }
    }

    for mut receiver in &mut slot_receivers {
        for message in receiver.receive() {
            player_team_map.apply_slots(&message.slots);
        }
    }
}

pub fn apply_ghost_placement_changed_system(
    mut commands: Commands,
    board_layout: Res<BoardLayout>,
    mut changes: MessageReader<GhostPlacementChanged>,
    ghost_units: Query<(Entity, &GhostUnit)>,
    lane_washes: Query<(Entity, &LaneGhostWash)>,
    target_markers: Query<(Entity, &TargetUnitGhost, Option<&BoardGhostPickable>)>,
    objective_markers: Query<(Entity, &ObjectiveTargetGhost, Option<&BoardGhostPickable>)>,
    target_units: Query<(Entity, &PlacementTargetUnit, Option<&Pickable>)>,
    objectives: Query<(Entity, &ObjectiveCell, Option<&Pickable>)>,
) {
    let mut latest_changes: Vec<(CardId, Option<PlayTarget>)> = Vec::new();

    for change in changes.read() {
        let Some(card_id) = change.card_id else {
            continue;
        };

        if let Some((_existing_card_id, target)) = latest_changes
            .iter_mut()
            .find(|(existing_card_id, _target)| *existing_card_id == card_id)
        {
            *target = change.target.clone();
        } else {
            latest_changes.push((card_id, change.target.clone()));
        }
    }

    for (card_id, target) in latest_changes {
        clear_card_ghosts(
            &mut commands,
            card_id,
            &ghost_units,
            &lane_washes,
            &target_markers,
            &objective_markers,
        );

        match target {
            Some(PlayTarget::BoardCell { lane, cell }) => {
                spawn_ghost_unit(&mut commands, &board_layout, card_id, lane, cell);
            }
            Some(PlayTarget::TargetUnit { unit_id, .. }) => {
                apply_target_unit_ghost(&mut commands, card_id, unit_id, &target_units);
            }
            Some(PlayTarget::TargetObj { player_id, lane }) => {
                apply_objective_target_ghost(&mut commands, card_id, player_id, lane, &objectives);
            }
            Some(PlayTarget::LaneWide { lane }) => {
                spawn_lane_ghost_wash(&mut commands, &board_layout, card_id, lane);
            }
            Some(PlayTarget::Instant) | None => {}
        }
    }
}

pub fn drain_placement_reveal_system(
    mut commands: Commands,
    mut receivers: Query<&mut MessageReceiver<S2CPlacementReveal>>,
    local_player: Res<BoardLocalPlayer>,
    mut collect_state: ResMut<PlacementRevealCollectState>,
    mut reveal_wait: ResMut<ResolutionRevealWait>,
    mut render_state: ResMut<BoardRenderState>,
    ghost_units: Query<(Entity, &GhostUnit)>,
    lane_washes: Query<(Entity, &LaneGhostWash)>,
    target_markers: Query<(Entity, &TargetUnitGhost, Option<&BoardGhostPickable>)>,
    objective_markers: Query<(Entity, &ObjectiveTargetGhost, Option<&BoardGhostPickable>)>,
) {
    let mut reveals = Vec::new();
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            reveals.push(message);
        }
    }

    if reveals.is_empty() {
        return;
    }

    clear_all_board_ghosts(
        &mut commands,
        &ghost_units,
        &lane_washes,
        &target_markers,
        &objective_markers,
    );

    if local_player.player_id.is_none() {
        warn!("Board Rendering: placement reveal received before local player id was known");
    }

    collect_state.start_from_reveals(&reveals, local_player.player_id);
    reveal_wait.start();
    *render_state = BoardRenderState::ResolutionReveal;
}

pub fn emit_ghost_clicked_events_system(
    mut clicks: MessageReader<Pointer<Click>>,
    ghost_interactions: Query<&BoardGhostInteraction>,
    mut writer: MessageWriter<GhostClickedEvent>,
) {
    for click in clicks.read() {
        if click.event.button != PointerButton::Primary {
            continue;
        }

        let Ok(ghost) = ghost_interactions.get(click.entity) else {
            continue;
        };

        writer.write(GhostClickedEvent {
            card_id: ghost.card_id,
        });
    }
}

pub fn emit_ghost_drag_start_events_system(
    mut presses: MessageReader<Pointer<Press>>,
    ghost_interactions: Query<&BoardGhostInteraction>,
    mut writer: MessageWriter<GhostDragStartEvent>,
) {
    for press in presses.read() {
        if press.event.button != PointerButton::Primary {
            continue;
        }

        let Ok(ghost) = ghost_interactions.get(press.entity) else {
            continue;
        };

        writer.write(GhostDragStartEvent {
            card_id: ghost.card_id,
        });
    }
}

pub fn drain_resolution_event_system(
    mut receivers: Query<&mut MessageReceiver<S2CResolutionEvent>>,
    mut pending_script: ResMut<PendingResolutionScript>,
    mut reveal_wait: ResMut<ResolutionRevealWait>,
    mut render_state: ResMut<BoardRenderState>,
) {
    let mut latest = None;
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            latest = Some(message);
        }
    }

    let Some(message) = latest else {
        return;
    };

    pending_script.set(message);
    if reveal_wait.is_active() || *render_state == BoardRenderState::ResolutionReveal {
        reveal_wait.clear();
        *render_state = BoardRenderState::ResolutionExecuting;
    }
}

pub fn tick_reveal_recovery_timeouts_system(
    time: Res<Time<Virtual>>,
    timings: Res<BoardRevealTimingConfig>,
    render_state: Res<BoardRenderState>,
    mut reveal_wait: ResMut<ResolutionRevealWait>,
    mut pending_script: ResMut<PendingResolutionScript>,
    mut recovery_writer: MessageWriter<SnapshotRecoveryRequested>,
) {
    let timeout = timings.timeout();
    let delta = time.delta();

    if reveal_wait.tick(delta, timeout) {
        warn!("ResolutionReveal stuck; requesting authoritative snapshot");
        recovery_writer.write(SnapshotRecoveryRequested::resolution_reveal_stuck());
    }

    if *render_state == BoardRenderState::Placement
        && pending_script.tick_without_reveal(delta, timeout)
    {
        warn!("PendingResolutionScript stuck; requesting authoritative snapshot");
        recovery_writer.write(SnapshotRecoveryRequested::pending_resolution_script_stuck());
    }
}

pub fn send_snapshot_recovery_requests_system(
    mut requests: MessageReader<SnapshotRecoveryRequested>,
    mut senders: Query<&mut MessageSender<C2SRequestSnapshot>>,
) {
    for _request in requests.read() {
        let mut sent = false;
        for mut sender in &mut senders {
            sender.send::<ReliableChannel>(C2SRequestSnapshot {});
            sent = true;
        }

        if !sent {
            warn!("Board Rendering: C2SRequestSnapshot requested but no MessageSender exists");
        }
    }
}

pub fn collect_placement_reveal_batch_system(
    mut collect_state: ResMut<PlacementRevealCollectState>,
    units: Query<(Entity, &BoardUnitOwner, &BoardUnitCard, &LaneCell), With<BoardUnit>>,
    mut writer: MessageWriter<PlacementRevealAnimReady>,
) {
    let Some(targets) = collect_state.take_ready_targets() else {
        return;
    };

    let mut entries = Vec::new();
    for target in targets {
        for (entity, owner, card, lane_cell) in &units {
            if reveal_target_matches_unit(target, owner, card, lane_cell) {
                entries.push(PlacementRevealEntry {
                    unit: entity,
                    lane: lane_cell.lane,
                    cell: lane_cell.cell,
                });
            }
        }
    }

    entries.sort_by_key(|entry| (entry.lane, entry.cell, entry.unit.to_bits()));

    if !entries.is_empty() {
        writer.write(PlacementRevealAnimReady { entries });
    }
}

fn reveal_target_matches_unit(
    target: PlacementRevealTarget,
    owner: &BoardUnitOwner,
    card: &BoardUnitCard,
    lane_cell: &LaneCell,
) -> bool {
    owner.0 == target.owner_id
        && card.card_id == Some(target.card_id)
        && lane_cell.lane == target.lane
        && lane_cell.cell == target.cell
}

fn sync_reveal_state_from_snapshot_system(
    mut snapshots: MessageReader<ClientGameSnapshotMessage>,
    mut local_player: ResMut<BoardLocalPlayer>,
    mut reveal_collect: ResMut<PlacementRevealCollectState>,
    mut reveal_wait: ResMut<ResolutionRevealWait>,
    mut pending_script: ResMut<PendingResolutionScript>,
) {
    let mut latest_snapshot = None;
    for snapshot in snapshots.read() {
        latest_snapshot = Some(&snapshot.0);
    }

    let Some(snapshot) = latest_snapshot else {
        return;
    };

    local_player.player_id = Some(snapshot.recipient_player_id);
    reveal_collect.clear();
    reveal_wait.clear();
    pending_script.clear();
}

fn insert_board_rendering_session_resources(mut commands: Commands) {
    let board_layout = BoardLayout::default();

    commands.insert_resource(board_layout);
    commands.insert_resource(CardAtlas::default());
    spawn_board_camera(&mut commands, &board_layout);
    spawn_board_grid(&mut commands, &board_layout);
}

fn remove_board_rendering_session_resources(
    mut commands: Commands,
    board_entities: Query<Entity, With<BoardRenderingEntity>>,
    mut player_team_map: ResMut<PlayerTeamMap>,
) {
    for entity in &board_entities {
        commands.entity(entity).despawn();
    }

    player_team_map.clear();
    commands.remove_resource::<BoardLayout>();
    commands.remove_resource::<CardAtlas>();
}

#[allow(clippy::too_many_arguments)]
fn rebuild_board_from_snapshot_system(
    mut commands: Commands,
    mut snapshots: MessageReader<ClientGameSnapshotMessage>,
    board_layout: Option<Res<BoardLayout>>,
    card_atlas: Option<Res<CardAtlas>>,
    config: Res<BoardRenderingConfig>,
    player_team_map: Res<PlayerTeamMap>,
    stale_entities: Query<Entity, With<BoardSnapshotEntity>>,
    mut render_state: ResMut<BoardRenderState>,
    mut current_phase: Option<ResMut<CurrentClientPhase>>,
    mut objective_identity_cache: ResMut<ObjectiveIdentityCache>,
    mut rebuild_writer: MessageWriter<BoardRebuildRequested>,
    mut tweens: Query<&mut TweenAnim>,
    mut anim_queue: Option<ResMut<AnimQueue>>,
    mut pending_phase: Option<ResMut<PendingPhaseChange>>,
    mut pending_objectives: Option<ResMut<PendingObjectiveDestroyedEvents>>,
    mut staged_objectives: Option<ResMut<StagedObjectiveRevealQueue>>,
) {
    let mut latest_snapshot = None;
    for snapshot in snapshots.read() {
        latest_snapshot = Some(snapshot.0.clone());
    }

    let Some(snapshot) = latest_snapshot else {
        return;
    };
    let Some(board_layout) = board_layout else {
        warn!("Board Rendering: snapshot ignored because BoardLayout is missing");
        return;
    };
    let Some(card_atlas) = card_atlas else {
        warn!("Board Rendering: snapshot ignored because CardAtlas is missing");
        return;
    };

    clear_pending_visual_state(
        &mut rebuild_writer,
        &mut tweens,
        anim_queue.as_deref_mut(),
        pending_phase.as_deref_mut(),
        pending_objectives.as_deref_mut(),
        staged_objectives.as_deref_mut(),
    );
    objective_identity_cache.clear();

    for entity in &stale_entities {
        commands.entity(entity).despawn();
    }

    *render_state = BoardRenderState::from_snapshot_phase(snapshot.phase);
    if let Some(current_phase) = current_phase.as_deref_mut() {
        current_phase.phase = snapshot.phase;
        current_phase.round = snapshot.round_number;
    }

    spawn_snapshot_objectives(&mut commands, &board_layout, &card_atlas, &snapshot);
    spawn_snapshot_units(
        &mut commands,
        &board_layout,
        &card_atlas,
        &config,
        &player_team_map,
        &snapshot,
    );
}

fn clear_pending_visual_state(
    rebuild_writer: &mut MessageWriter<BoardRebuildRequested>,
    tweens: &mut Query<&mut TweenAnim>,
    anim_queue: Option<&mut AnimQueue>,
    pending_phase: Option<&mut PendingPhaseChange>,
    pending_objectives: Option<&mut PendingObjectiveDestroyedEvents>,
    staged_objectives: Option<&mut StagedObjectiveRevealQueue>,
) {
    rebuild_writer.write(BoardRebuildRequested);

    if let Some(anim_queue) = anim_queue {
        anim_queue.reset();
    }
    if let Some(pending_phase) = pending_phase {
        pending_phase.clear();
    }
    if let Some(pending_objectives) = pending_objectives {
        pending_objectives.clear();
    }
    if let Some(staged_objectives) = staged_objectives {
        staged_objectives.clear();
    }

    for mut tween in tweens.iter_mut() {
        if let Err(error) = cancel_tween_anim_in_place(&mut tween) {
            warn!("Board Rendering: failed to cancel tween during snapshot rebuild: {error}");
        }
    }
}

fn spawn_snapshot_units(
    commands: &mut Commands,
    board_layout: &BoardLayout,
    card_atlas: &CardAtlas,
    config: &BoardRenderingConfig,
    player_team_map: &PlayerTeamMap,
    snapshot: &S2CGameSnapshot,
) {
    let co_occupancy_offsets =
        snapshot_co_occupancy_offsets(snapshot, player_team_map, config.co_occupancy_side_offset);

    for unit in &snapshot.board.units {
        let co_occupancy_x_offset = co_occupancy_offsets
            .get(&unit.unit_id)
            .copied()
            .unwrap_or(0.0);
        spawn_snapshot_unit(
            commands,
            board_layout,
            card_atlas,
            config,
            snapshot,
            unit,
            co_occupancy_x_offset,
        );
    }
}

fn spawn_snapshot_unit(
    commands: &mut Commands,
    board_layout: &BoardLayout,
    card_atlas: &CardAtlas,
    config: &BoardRenderingConfig,
    snapshot: &S2CGameSnapshot,
    unit: &UnitBoardState,
    co_occupancy_x_offset: f32,
) {
    let Some((lane, cell)) = visible_unit_cell(unit, snapshot.recipient_player_id) else {
        warn!(
            "Board Rendering: unit {:?} has out-of-range snapshot location; skipped",
            unit.unit_id
        );
        return;
    };

    let stats = board_unit_stats(unit, card_atlas);
    let (frame_index, used_missing_art_fallback) = unit_frame_index(unit, card_atlas);
    let world_xy = board_layout.cell_to_world(lane, cell);

    let unit_entity = commands
        .spawn((
            BoardRenderingEntity,
            BoardSnapshotEntity,
            BoardUnit {
                unit_id: unit.unit_id,
            },
            BoardUnitOwner(unit.owner_id),
            BoardUnitCard {
                card_id: unit.card_id,
                frame_index,
                used_missing_art_fallback,
            },
            stats,
            LaneCell { lane, cell },
            StatusEffectsList::default(),
            unit_sprite(card_atlas, frame_index),
            Transform::from_xyz(
                world_xy.x + co_occupancy_x_offset,
                world_xy.y,
                rendering_constants::Z_UNITS,
            ),
        ))
        .id();

    if let Some(source_class) = unit.source_class {
        commands
            .entity(unit_entity)
            .insert(BoardUnitSourceClass(source_class));
    }

    spawn_hp_bar_children(commands, unit_entity, card_atlas, stats, config);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CoOccupancyTeamKey {
    Known(u8),
    UnknownOwner(PlayerId),
}

fn snapshot_co_occupancy_offsets(
    snapshot: &S2CGameSnapshot,
    player_team_map: &PlayerTeamMap,
    side_offset: f32,
) -> HashMap<EntityId, f32> {
    let mut groups = HashMap::<(CoOccupancyTeamKey, u8, u8), Vec<EntityId>>::new();

    for unit in &snapshot.board.units {
        let Some((lane, cell)) = visible_unit_cell(unit, snapshot.recipient_player_id) else {
            continue;
        };
        let team = player_team_map
            .team_for(unit.owner_id)
            .map(CoOccupancyTeamKey::Known)
            .unwrap_or(CoOccupancyTeamKey::UnknownOwner(unit.owner_id));
        groups
            .entry((team, lane, cell))
            .or_default()
            .push(unit.unit_id);
    }

    let mut offsets = HashMap::new();
    for units in groups.values_mut() {
        if units.len() <= 1 {
            continue;
        }

        units.sort_unstable();
        for (index, unit_id) in units.iter().copied().enumerate() {
            assert!(
                index <= u8::MAX as usize,
                "F3 co-occupancy: unit_index={} > 255 - invalid co-occupancy state",
                index
            );
            offsets.insert(unit_id, co_occupancy_offset(index as u8, side_offset));
        }
    }

    offsets
}

pub fn co_occupancy_offset(unit_index: u8, side_offset: f32) -> f32 {
    assert!(
        unit_index <= 1,
        "F3 co-occupancy: unit_index={} > 1 - invalid co-occupancy state",
        unit_index
    );
    (f32::from(unit_index) - 0.5) * side_offset
}

fn visible_unit_cell(unit: &UnitBoardState, recipient_player_id: PlayerId) -> Option<(u8, u8)> {
    match unit.location {
        UnitBoardLocation::BoardCell { lane, cell } => {
            in_board_bounds(lane, cell).then_some((lane, cell))
        }
        UnitBoardLocation::ObjectiveAttachment { lane } => {
            let cell = if unit.owner_id == recipient_player_id {
                BOARD_CELL_COUNT
            } else {
                1
            };
            in_board_bounds(lane, cell).then_some((lane, cell))
        }
    }
}

fn in_board_bounds(lane: u8, cell: u8) -> bool {
    (1..=BOARD_LANE_COUNT).contains(&lane) && (1..=BOARD_CELL_COUNT).contains(&cell)
}

fn unit_frame_index(unit: &UnitBoardState, card_atlas: &CardAtlas) -> (usize, bool) {
    let Some(card_id) = unit.card_id else {
        warn!(
            "Board Rendering asset-miss: unit {:?} has no card_id",
            unit.unit_id
        );
        return (UNIT_PLACEHOLDER_FRAME_INDEX, true);
    };

    if let Some(frame) = card_atlas.unit_frame(card_id) {
        (frame.frame_index, false)
    } else {
        warn!(
            "Board Rendering asset-miss: missing art for card_id {:?}; using placeholder",
            card_id
        );
        (UNIT_PLACEHOLDER_FRAME_INDEX, true)
    }
}

fn board_unit_stats(unit: &UnitBoardState, card_atlas: &CardAtlas) -> BoardUnitStats {
    let stats = unit.stats.unwrap_or_else(|| {
        warn!(
            "Board Rendering: unit {:?} missing stats in snapshot; defaulting to 1 HP",
            unit.unit_id
        );
        UnitStatsSnapshot {
            hp: 1,
            atk: 0,
            mp: 0,
            ar: 0,
        }
    });
    let hp_max = unit
        .card_id
        .and_then(|card_id| card_atlas.unit_frame(card_id))
        .map(|frame| frame.max_hp)
        .unwrap_or(stats.hp)
        .max(1);

    BoardUnitStats {
        hp_current: stats.hp,
        hp_max,
        atk: stats.atk,
        mp: stats.mp,
        ar: stats.ar,
    }
}

fn spawn_snapshot_objectives(
    commands: &mut Commands,
    board_layout: &BoardLayout,
    card_atlas: &CardAtlas,
    snapshot: &S2CGameSnapshot,
) {
    for player in &snapshot.players {
        for objective in &player.objectives {
            spawn_standing_objective(
                commands,
                board_layout,
                card_atlas,
                snapshot.recipient_player_id,
                player.player_id,
                objective,
            );
        }
    }
}

fn spawn_standing_objective(
    commands: &mut Commands,
    board_layout: &BoardLayout,
    card_atlas: &CardAtlas,
    recipient_player_id: PlayerId,
    owner_id: PlayerId,
    objective: &ObjectiveSnapshot,
) {
    if objective.is_destroyed {
        return;
    }

    let lane = objective.lane;
    let cell = objective_cell(owner_id, recipient_player_id);
    if !in_board_bounds(lane, cell) {
        warn!(
            "Board Rendering: objective for {:?} lane {} is out of range; skipped",
            owner_id, lane
        );
        return;
    }

    let world_xy = board_layout.cell_to_world(lane, cell);
    let hp = StandingObjectiveHp {
        hp_current: objective.hp,
        hp_max: objective.hp.max(1),
    };
    let objective_entity = commands
        .spawn((
            BoardRenderingEntity,
            BoardSnapshotEntity,
            StandingObjective { owner_id, lane },
            hp,
            LaneCell { lane, cell },
            objective_unknown_sprite(card_atlas),
            Transform::from_xyz(world_xy.x, world_xy.y, rendering_constants::Z_OBJECTIVES),
        ))
        .id();

    spawn_objective_hp_bar_children(commands, objective_entity, card_atlas, hp);
}

fn objective_cell(owner_id: PlayerId, recipient_player_id: PlayerId) -> u8 {
    if owner_id == recipient_player_id {
        1
    } else {
        BOARD_CELL_COUNT
    }
}

fn spawn_hp_bar_children(
    commands: &mut Commands,
    parent: Entity,
    card_atlas: &CardAtlas,
    stats: BoardUnitStats,
    config: &BoardRenderingConfig,
) {
    let visual = hp_bar_visual(stats.hp_current, stats.hp_max, *config);
    spawn_hp_bar_background(commands, parent, card_atlas);
    spawn_hp_bar_fill(commands, parent, card_atlas, visual);
}

pub fn update_status_icons_system(
    mut commands: Commands,
    card_atlas: Res<CardAtlas>,
    definitions: Res<StatusDisplayDefinitions>,
    units: Query<
        (Entity, &StatusEffectsList, Option<&Children>),
        (With<BoardUnit>, Changed<StatusEffectsList>),
    >,
    status_children: Query<(), Or<(With<StatusIcon>, With<StatusOverflowBadge>)>>,
) {
    for (unit, status_effects, children) in &units {
        if let Some(children) = children {
            for child in children.iter() {
                if status_children.get(child).is_ok() {
                    commands.entity(child).despawn();
                }
            }
        }

        spawn_status_icon_children(
            &mut commands,
            unit,
            &card_atlas,
            &definitions,
            status_effects,
        );
    }
}

fn spawn_status_icon_children(
    commands: &mut Commands,
    parent: Entity,
    card_atlas: &CardAtlas,
    definitions: &StatusDisplayDefinitions,
    status_effects: &StatusEffectsList,
) {
    let visible = visible_status_effects(status_effects, definitions);
    let hidden_count = visible.hidden_count.min(u8::MAX as usize) as u8;

    for (slot, effect) in visible.effects.iter().enumerate() {
        let slot = slot as u8;
        commands.spawn((
            BoardRenderingEntity,
            BoardSnapshotEntity,
            StatusIcon {
                key: effect.visual.key,
                slot,
                display_tier: effect.definition.display_tier,
                remaining_duration_sort_key: effect.visual.remaining_duration_sort_key,
            },
            status_icon_sprite(
                card_atlas,
                effect.definition.icon_frame_index,
                effect.definition.tint,
            ),
            Transform::from_translation(status_icon_slot_translation(slot)),
            Visibility::Inherited,
            ChildOf(parent),
        ));
    }

    if hidden_count > 0 {
        let slot = 3;
        commands.spawn((
            BoardRenderingEntity,
            BoardSnapshotEntity,
            StatusOverflowBadge { slot, hidden_count },
            status_icon_sprite(
                card_atlas,
                STATUS_OVERFLOW_BADGE_FRAME_INDEX,
                Color::srgba(0.10, 0.11, 0.14, 0.94),
            ),
            Transform::from_translation(status_icon_slot_translation(slot)),
            Visibility::Inherited,
            ChildOf(parent),
        ));
    }
}

struct VisibleStatusEffects {
    effects: Vec<StatusEffectRenderEntry>,
    hidden_count: usize,
}

#[derive(Clone, Copy)]
struct StatusEffectRenderEntry {
    visual: StatusEffectVisual,
    definition: StatusDisplayDefinition,
}

fn visible_status_effects(
    status_effects: &StatusEffectsList,
    definitions: &StatusDisplayDefinitions,
) -> VisibleStatusEffects {
    let mut effects = status_effects
        .effects()
        .iter()
        .filter_map(|visual| {
            let Some(definition) = definitions.definition(visual.key).copied() else {
                warn!(
                    "Board Rendering: missing status display definition for {:?}",
                    visual.key
                );
                return None;
            };
            Some(StatusEffectRenderEntry {
                visual: *visual,
                definition,
            })
        })
        .collect::<Vec<_>>();

    effects.sort_by(|left, right| {
        left.definition
            .display_tier
            .cmp(&right.definition.display_tier)
            .then_with(|| {
                right
                    .visual
                    .remaining_duration_sort_key
                    .cmp(&left.visual.remaining_duration_sort_key)
            })
            .then_with(|| {
                left.visual
                    .key
                    .deterministic_sort_key()
                    .cmp(&right.visual.key.deterministic_sort_key())
            })
    });

    let hidden_count = effects.len().saturating_sub(3);
    effects.truncate(3);

    VisibleStatusEffects {
        effects,
        hidden_count,
    }
}

pub fn status_icon_slot_translation(slot: u8) -> Vec3 {
    Vec3::new(
        rendering_constants::STATUS_ICON_TOP_RIGHT_X_OFFSET
            - f32::from(slot) * rendering_constants::STATUS_ICON_SLOT_STEP_X,
        rendering_constants::STATUS_ICON_TOP_RIGHT_Y_OFFSET,
        rendering_constants::STATUS_ICON_LOCAL_Z,
    )
}

fn status_icon_sprite(card_atlas: &CardAtlas, frame_index: usize, color: Color) -> Sprite {
    atlas_sprite(
        card_atlas.board_elements_image.clone(),
        card_atlas.board_elements_layout.clone(),
        frame_index,
        rendering_constants::STATUS_ICON_SIZE,
        color,
    )
}

fn spawn_objective_hp_bar_children(
    commands: &mut Commands,
    parent: Entity,
    card_atlas: &CardAtlas,
    hp: StandingObjectiveHp,
) {
    let visual = hp_bar_visual(hp.hp_current, hp.hp_max, BoardRenderingConfig::default());
    spawn_hp_bar_background(commands, parent, card_atlas);
    spawn_hp_bar_fill(commands, parent, card_atlas, visual);
}

fn spawn_hp_bar_background(commands: &mut Commands, parent: Entity, card_atlas: &CardAtlas) {
    commands.spawn((
        BoardRenderingEntity,
        BoardSnapshotEntity,
        HpBarBackground,
        hp_bar_sprite(
            card_atlas,
            Color::srgba(0.08, 0.08, 0.08, 0.76),
            rendering_constants::HP_BAR_SIZE,
        ),
        Transform::from_xyz(
            0.0,
            rendering_constants::HP_BAR_Y_OFFSET,
            rendering_constants::HEALTH_BAR_LOCAL_Z,
        ),
        Visibility::Inherited,
        ChildOf(parent),
    ));
}

fn spawn_hp_bar_fill(
    commands: &mut Commands,
    parent: Entity,
    card_atlas: &CardAtlas,
    visual: HpBarVisual,
) {
    commands.spawn((
        BoardRenderingEntity,
        BoardSnapshotEntity,
        HpBarFill,
        hp_bar_sprite(
            card_atlas,
            visual.color.tint(),
            rendering_constants::HP_BAR_SIZE,
        ),
        Transform {
            translation: Vec3::new(
                hp_fill_offset_x(visual.fill),
                rendering_constants::HP_BAR_Y_OFFSET,
                rendering_constants::HEALTH_BAR_LOCAL_Z,
            ),
            scale: Vec3::new(visual.fill, 1.0, 1.0),
            ..default()
        },
        Visibility::Inherited,
        ChildOf(parent),
    ));
}

fn unit_sprite(card_atlas: &CardAtlas, frame_index: usize) -> Sprite {
    atlas_sprite(
        card_atlas.image.clone(),
        card_atlas.layout.clone(),
        frame_index,
        rendering_constants::UNIT_SPRITE_SIZE,
        Color::srgba(1.0, 1.0, 1.0, 1.0),
    )
}

fn objective_unknown_sprite(card_atlas: &CardAtlas) -> Sprite {
    atlas_sprite(
        card_atlas.board_elements_image.clone(),
        card_atlas.board_elements_layout.clone(),
        OBJECTIVE_UNKNOWN_FRAME_INDEX,
        rendering_constants::OBJECTIVE_SPRITE_SIZE,
        Color::srgba(1.0, 1.0, 1.0, 1.0),
    )
}

fn hp_bar_sprite(card_atlas: &CardAtlas, color: Color, size: Vec2) -> Sprite {
    atlas_sprite(
        card_atlas.image.clone(),
        card_atlas.layout.clone(),
        HP_BAR_WHITE_PIXEL_FRAME_INDEX,
        size,
        color,
    )
}

fn atlas_sprite(
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    index: usize,
    custom_size: Vec2,
    color: Color,
) -> Sprite {
    Sprite {
        image,
        texture_atlas: Some(TextureAtlas { layout, index }),
        custom_size: Some(custom_size),
        color,
        ..default()
    }
}

fn update_hp_bars_system(
    config: Res<BoardRenderingConfig>,
    units: Query<(&BoardUnitStats, &Children), With<BoardUnit>>,
    mut fills: Query<(&mut Transform, &mut Sprite), With<HpBarFill>>,
) {
    for (stats, children) in &units {
        let visual = hp_bar_visual(stats.hp_current, stats.hp_max, *config);
        for child in children.iter() {
            if let Ok((mut transform, mut sprite)) = fills.get_mut(child) {
                apply_hp_fill_visual(&mut transform, &mut sprite, visual);
            }
        }
    }
}

fn apply_hp_fill_visual(transform: &mut Transform, sprite: &mut Sprite, visual: HpBarVisual) {
    transform.scale.x = visual.fill;
    transform.translation.x = hp_fill_offset_x(visual.fill);
    sprite.color = visual.color.tint();
}

pub fn hp_bar_visual(hp_current: u8, hp_max: u8, config: BoardRenderingConfig) -> HpBarVisual {
    config.assert_valid();
    let hp_max_safe = hp_max.max(1);
    if hp_max == 0 {
        warn!("Board Rendering: UnitStats.hp_max=0 from server; clamped to 1");
    }

    let fill = (f32::from(hp_current) / f32::from(hp_max_safe)).clamp(0.0, 1.0);
    let color = if fill >= config.health_bar_green_threshold - HP_THRESHOLD_EPSILON {
        HpBarColor::Green
    } else if fill >= config.health_bar_red_threshold - HP_THRESHOLD_EPSILON {
        HpBarColor::Yellow
    } else {
        HpBarColor::Red
    };

    HpBarVisual { fill, color }
}

fn hp_fill_offset_x(fill: f32) -> f32 {
    -rendering_constants::HP_BAR_SIZE.x * (1.0 - fill) * 0.5
}

fn spawn_board_camera(commands: &mut Commands, board_layout: &BoardLayout) {
    let camera_xy = board_center(board_layout);

    commands.spawn((
        BoardRenderingEntity,
        BoardCamera,
        Camera2d,
        Transform::from_xyz(
            camera_xy.x,
            camera_xy.y,
            rendering_constants::Z_BOARD_CAMERA,
        ),
    ));
}

fn spawn_board_grid(commands: &mut Commands, board_layout: &BoardLayout) {
    for lane in 1..=BOARD_LANE_COUNT {
        for cell in 1..=BOARD_CELL_COUNT {
            spawn_cell_node(commands, board_layout, lane, cell);
        }
    }
}

fn spawn_cell_node(commands: &mut Commands, board_layout: &BoardLayout, lane: u8, cell: u8) {
    let world_xy = board_layout.cell_to_world(lane, cell);
    let highlight_state = SpawnHighlightState::Inactive;

    commands.spawn((
        BoardRenderingEntity,
        BoardCellNode,
        LaneCell { lane, cell },
        highlight_state,
        Sprite::from_color(
            highlight_state.tint(),
            Vec2::splat(rendering_constants::CELL_NODE_SIZE),
        ),
        Transform::from_xyz(world_xy.x, world_xy.y, rendering_constants::Z_CELL_NODES),
    ));
}

fn spawn_ghost_unit(
    commands: &mut Commands,
    board_layout: &BoardLayout,
    card_id: CardId,
    lane: u8,
    cell: u8,
) {
    let world_xy = board_layout.cell_to_world(lane, cell);

    commands.spawn((
        BoardRenderingEntity,
        GhostUnit { card_id },
        BoardGhostInteraction { card_id },
        Pickable::default(),
        Sprite::from_color(
            Color::srgba(1.0, 1.0, 1.0, 0.5),
            Vec2::splat(rendering_constants::CELL_NODE_SIZE),
        ),
        Transform::from_xyz(world_xy.x, world_xy.y, rendering_constants::Z_GHOST_UNIT),
    ));
}

fn spawn_lane_ghost_wash(
    commands: &mut Commands,
    board_layout: &BoardLayout,
    card_id: CardId,
    lane: u8,
) {
    let start = board_layout.cell_to_world(lane, 1);
    let end = board_layout.cell_to_world(lane, BOARD_CELL_COUNT);
    let center = (start + end) * 0.5;
    let size = Vec2::new(
        board_layout.cell_width * f32::from(BOARD_CELL_COUNT),
        board_layout.lane_height * 0.72,
    );

    commands.spawn((
        BoardRenderingEntity,
        LaneGhostWash { card_id, lane },
        BoardGhostInteraction { card_id },
        Pickable::default(),
        Sprite::from_color(Color::srgba(0.36, 0.74, 1.0, 0.28), size),
        Transform::from_xyz(center.x, center.y, rendering_constants::Z_FIELD_WASH),
    ));
}

fn apply_target_unit_ghost(
    commands: &mut Commands,
    card_id: CardId,
    unit_id: shared::protocol::EntityId,
    target_units: &Query<(Entity, &PlacementTargetUnit, Option<&Pickable>)>,
) {
    let Some((entity, _target_unit, pickable)) = target_units
        .iter()
        .find(|(_entity, target_unit, _pickable)| target_unit.unit_id == unit_id)
    else {
        return;
    };

    insert_target_marker(
        commands,
        entity,
        pickable.is_some(),
        TargetUnitGhost { card_id },
    );
}

fn apply_objective_target_ghost(
    commands: &mut Commands,
    card_id: CardId,
    player_id: shared::session::PlayerId,
    lane: u8,
    objectives: &Query<(Entity, &ObjectiveCell, Option<&Pickable>)>,
) {
    let Some((entity, _objective, pickable)) =
        objectives.iter().find(|(_entity, objective, _pickable)| {
            objective.player_id == player_id && objective.lane == lane
        })
    else {
        return;
    };

    insert_objective_marker(
        commands,
        entity,
        pickable.is_some(),
        ObjectiveTargetGhost { card_id },
    );
}

fn insert_target_marker(
    commands: &mut Commands,
    entity: Entity,
    has_pickable: bool,
    marker: TargetUnitGhost,
) {
    let mut entity_commands = commands.entity(entity);
    entity_commands.insert((
        marker,
        BoardGhostInteraction {
            card_id: marker.card_id,
        },
    ));
    if !has_pickable {
        entity_commands.insert((Pickable::default(), BoardGhostPickable));
    }
}

fn insert_objective_marker(
    commands: &mut Commands,
    entity: Entity,
    has_pickable: bool,
    marker: ObjectiveTargetGhost,
) {
    let mut entity_commands = commands.entity(entity);
    entity_commands.insert((
        marker,
        BoardGhostInteraction {
            card_id: marker.card_id,
        },
    ));
    if !has_pickable {
        entity_commands.insert((Pickable::default(), BoardGhostPickable));
    }
}

fn clear_card_ghosts(
    commands: &mut Commands,
    card_id: CardId,
    ghost_units: &Query<(Entity, &GhostUnit)>,
    lane_washes: &Query<(Entity, &LaneGhostWash)>,
    target_markers: &Query<(Entity, &TargetUnitGhost, Option<&BoardGhostPickable>)>,
    objective_markers: &Query<(Entity, &ObjectiveTargetGhost, Option<&BoardGhostPickable>)>,
) {
    for (entity, ghost) in ghost_units {
        if ghost.card_id == card_id {
            despawn_if_exists(commands, entity);
        }
    }

    for (entity, wash) in lane_washes {
        if wash.card_id == card_id {
            despawn_if_exists(commands, entity);
        }
    }

    for (entity, marker, owned_pickable) in target_markers {
        if marker.card_id == card_id {
            remove_target_ghost_marker(commands, entity, owned_pickable.is_some());
        }
    }

    for (entity, marker, owned_pickable) in objective_markers {
        if marker.card_id == card_id {
            remove_objective_ghost_marker(commands, entity, owned_pickable.is_some());
        }
    }
}

fn clear_all_board_ghosts(
    commands: &mut Commands,
    ghost_units: &Query<(Entity, &GhostUnit)>,
    lane_washes: &Query<(Entity, &LaneGhostWash)>,
    target_markers: &Query<(Entity, &TargetUnitGhost, Option<&BoardGhostPickable>)>,
    objective_markers: &Query<(Entity, &ObjectiveTargetGhost, Option<&BoardGhostPickable>)>,
) {
    for (entity, _ghost) in ghost_units {
        despawn_if_exists(commands, entity);
    }

    for (entity, _wash) in lane_washes {
        despawn_if_exists(commands, entity);
    }

    for (entity, _marker, owned_pickable) in target_markers {
        remove_target_ghost_marker(commands, entity, owned_pickable.is_some());
    }

    for (entity, _marker, owned_pickable) in objective_markers {
        remove_objective_ghost_marker(commands, entity, owned_pickable.is_some());
    }
}

fn despawn_if_exists(commands: &mut Commands, entity: Entity) {
    if let Ok(mut entity_commands) = commands.get_entity(entity) {
        entity_commands.despawn();
    }
}

fn remove_target_ghost_marker(commands: &mut Commands, entity: Entity, remove_pickable: bool) {
    let Ok(mut entity_commands) = commands.get_entity(entity) else {
        return;
    };

    entity_commands.remove::<(TargetUnitGhost, BoardGhostInteraction)>();
    if remove_pickable {
        entity_commands.remove::<(Pickable, BoardGhostPickable)>();
    }
}

fn remove_objective_ghost_marker(commands: &mut Commands, entity: Entity, remove_pickable: bool) {
    let Ok(mut entity_commands) = commands.get_entity(entity) else {
        return;
    };

    entity_commands.remove::<(ObjectiveTargetGhost, BoardGhostInteraction)>();
    if remove_pickable {
        entity_commands.remove::<(Pickable, BoardGhostPickable)>();
    }
}

fn board_center(board_layout: &BoardLayout) -> Vec2 {
    Vec2::new(
        board_layout.board_origin.x
            + f32::from(BOARD_CELL_COUNT - 1) * board_layout.cell_width * 0.5,
        board_layout.board_origin.y
            - f32::from(BOARD_LANE_COUNT - 1) * board_layout.lane_height * 0.5,
    )
}

impl BoardRenderState {
    fn from_snapshot_phase(phase: RoundPhase) -> Self {
        match phase {
            RoundPhase::Handshaking => Self::Idle,
            RoundPhase::Lobby => Self::Lobby,
            RoundPhase::DraftInitial => Self::DraftInitial,
            RoundPhase::DraftShop => Self::DraftShop,
            RoundPhase::DraftAuction => Self::DraftAuction,
            RoundPhase::Placement => Self::Placement,
            RoundPhase::Resolution => Self::Resolution,
            RoundPhase::GameOver => Self::GameOver,
        }
    }
}
