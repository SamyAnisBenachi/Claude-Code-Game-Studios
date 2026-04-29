// shared — protocol types, GameConfig struct, shared data definitions
// ADR-003: zero Bevy plugin deps; compiles with bevy default-features=false only
// ADR-002: no game logic here — pure data + message type definitions

pub mod protocol;
pub mod card;
pub mod config;
