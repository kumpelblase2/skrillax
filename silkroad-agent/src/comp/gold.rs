use bevy_ecs_macros::Component;
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
    type Change = GoldChange;

    fn as_change(&self) -> Self::Change {
        GoldChange(self.0)
    }
}

#[derive(Copy, Clone)]
pub(crate) struct GoldChange(pub u64);
