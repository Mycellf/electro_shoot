use std::ops::{Deref, DerefMut};

use crate::object::Object;

pub struct Projectile {
    pub object: Object,
}

impl Deref for Projectile {
    type Target = Object;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl DerefMut for Projectile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.object
    }
}
