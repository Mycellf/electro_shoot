use std::ops::{Deref, DerefMut};

use crate::object::Transform;

pub struct Particle {
    pub transform: Transform,
}

impl Deref for Particle {
    type Target = Transform;

    fn deref(&self) -> &Self::Target {
        &self.transform
    }
}

impl DerefMut for Particle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transform
    }
}
