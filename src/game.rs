use slotmap::{HopSlotMap, new_key_type};

use crate::{enemy::Enemy, projectile::Projectile};

#[derive(Debug, Default)]
pub struct Game {
    pub enemies: HopSlotMap<EnemyKey, Enemy>,
    pub projectiles: HopSlotMap<ProjectileKey, Projectile>,
}

new_key_type! {
    pub struct EnemyKey;
    pub struct ProjectileKey;
}

impl Game {
    pub fn draw(&self) {
        for (_, enemy) in &self.enemies {
            enemy.draw();
        }

        for (_, projectile) in &self.projectiles {
            projectile.draw();
        }
    }

    pub fn tick(&mut self, dt: f64) {
        self.projectiles.retain(|_, projectile| {
            projectile.tick(&mut self.enemies, dt);
            !projectile.should_delete()
        });

        self.enemies.retain(|_, enemy| {
            enemy.tick(dt);
            !enemy.should_delete()
        });
    }
}
