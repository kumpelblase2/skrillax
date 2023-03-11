use bevy_ecs_macros::Resource;

#[derive(Default, Resource)]
pub struct AttackInstanceCounter(u32);

impl AttackInstanceCounter {
    pub fn next(&mut self) -> u32 {
        let current = self.0;
        self.0 = self.0.wrapping_add(1);
        current
    }
}
