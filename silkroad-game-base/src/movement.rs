#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MovementSpeed {
    Running,
    Walking,
    Berserk,
}

impl Default for MovementSpeed {
    fn default() -> Self {
        MovementSpeed::Running
    }
}
