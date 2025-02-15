use bevy::prelude::*;

pub(crate) trait Reset {
    fn reset(&mut self);
}

pub(crate) trait AppResetExt {
    fn reset<T: Reset + Component>(&mut self) -> &mut Self;
}

impl AppResetExt for App {
    fn reset<T: Reset + Component>(&mut self) -> &mut Self {
        self.add_systems(Last, reset_component::<T>);
        self
    }
}

fn reset_component<T: Reset + Component>(mut query: Query<&mut T>) {
    for mut item in query.iter_mut() {
        item.bypass_change_detection().reset();
    }
}
