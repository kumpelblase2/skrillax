use bevy_ecs::prelude::*;
use std::collections::HashSet;

#[derive(Component)]
pub struct Visibility {
    pub visibility_radius: f32,
    pub entities_in_radius: HashSet<Entity>,
    pub added_entities: Vec<Entity>,
    pub removed_entities: Vec<Entity>,
}
