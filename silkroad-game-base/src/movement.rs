#[derive(Copy, Clone, Eq, PartialEq, Default)]
pub enum MovementSpeed {
    #[default]
    Running,
    Walking,
    Berserk,
}
