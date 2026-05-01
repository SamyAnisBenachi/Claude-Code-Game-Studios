use std::collections::VecDeque;

use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct StagedObjectiveRevealQueue {
    reveals: VecDeque<(u8, Timer)>,
}

impl StagedObjectiveRevealQueue {
    pub fn push(&mut self, lane: u8, timer: Timer) {
        self.reveals.push_back((lane, timer));
    }

    pub fn pop_front(&mut self) -> Option<(u8, Timer)> {
        self.reveals.pop_front()
    }

    pub fn clear(&mut self) {
        self.reveals.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.reveals.is_empty()
    }

    pub fn len(&self) -> usize {
        self.reveals.len()
    }
}
