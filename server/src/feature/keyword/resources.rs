use std::collections::VecDeque;

use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct ChainDeathBuffer(pub VecDeque<(Entity, Option<Entity>)>);
