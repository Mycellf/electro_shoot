use std::ops::{Deref, DerefMut};

use crate::object::Object;

#[derive(Clone, Debug)]
pub struct Enemy {
    pub object: Object,

    pub health: u32,
    pub time_since_hit: f64,
}

impl Enemy {
    pub fn tick(&mut self, dt: f64) {
        self.object.tick(dt);

        self.time_since_hit += dt;
    }

    pub fn hit(&mut self, damage: u32) {
        self.health = self.health.saturating_sub(damage);
        self.time_since_hit = 0.0;
    }

    pub fn should_delete(&self) -> bool {
        self.health == 0
    }
}

impl Deref for Enemy {
    type Target = Object;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl DerefMut for Enemy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.object
    }
}
