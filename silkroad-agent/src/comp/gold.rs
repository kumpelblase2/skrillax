use bevy::prelude::*;
use derive_more::Constructor;
use silkroad_game_base::ChangeProvided;

#[derive(Component, Copy, Clone, Constructor)]
pub(crate) struct GoldPouch(u64);

impl GoldPouch {
    pub(crate) fn amount(&self) -> u64 {
        self.0
    }

    pub(crate) fn gain(&mut self, amount: u64) {
        self.0 = self.0.saturating_add(amount);
    }

    pub(crate) fn spend(&mut self, amount: u64) {
        self.0 = self.0.saturating_sub(amount)
    }
}

impl ChangeProvided for GoldPouch {
    type Change = Self; // GoldPouch itself is the snapshot

    fn as_change(&self) -> Self::Change {
        *self // Return a copy of self since GoldPouch is Copy
    }
}

// GoldChange struct is no longer needed here as GoldPouch itself will be the Change type
// and will have the ApplyToDatabase implementation.
// pub(crate) struct GoldChange(pub u64); // Removed
