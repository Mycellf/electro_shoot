use slotmap::{SlotMap, new_key_type};

use crate::{enemy::Enemy, projectile::Projectile};

pub struct Game {
    pub enemies: SlotMap<EnemyKey, Enemy>,
    pub projectiles: SlotMap<ProjectileKey, Projectile>,
}

new_key_type! {
    pub struct EnemyKey;
    pub struct ProjectileKey;
}

impl Game {
    pub fn tick(&mut self, dt: f64) {
        for (_, projectile) in &mut self.projectiles {
            projectile.tick(dt);
        }

        for (_, enemy) in &mut self.enemies {
            enemy.tick(dt);
        }
    }
}
