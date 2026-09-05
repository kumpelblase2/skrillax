use bevy::prelude::Resource;
use silkroad_starter_gear::StarterGear;

/// Immutable, startup-compiled starter gear used during Character Selection.
#[derive(Resource)]
pub(crate) struct StarterGearResource(pub(crate) StarterGear);
