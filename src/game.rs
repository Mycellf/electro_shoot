use macroquad::camera::Camera2D;
use slotmap::{HopSlotMap, new_key_type};

use crate::{enemy::Enemy, particle::Particle, projectile::Projectile, turret::Turret, utils};

#[derive(Debug, Default)]
pub struct Game {
    pub enemies: HopSlotMap<EnemyKey, Enemy>,
    pub projectiles: HopSlotMap<ProjectileKey, Projectile>,
    pub particles: HopSlotMap<ParticleKey, Particle>,
    pub turret: Turret,
}

new_key_type! {
    pub struct EnemyKey;
    pub struct ProjectileKey;
    pub struct ParticleKey;
}

impl Game {
    pub fn draw(&self) {
        self.turret.draw();

        for (_, enemy) in &self.enemies {
            enemy.draw();
        }

        for (_, particle) in &self.particles {
            particle.draw();
        }

        for (_, projectile) in &self.projectiles {
            projectile.draw();
        }
    }

    pub fn tick_input(&mut self, dt: f64) {
        self.turret.input.tick(dt);
    }

    pub fn tick(&mut self, camera: &mut Camera2D, dt: f64) {
        self.turret
            .tick(utils::mouse_position(camera), &mut self.projectiles, dt);

        let camera_bounds = utils::bounds_of_camera(camera);

        self.projectiles.retain(|_, projectile| {
            projectile.tick(&mut self.enemies, &mut self.particles, dt);
            !projectile.should_delete()
                && camera_bounds.is_colliding(&projectile.shape, projectile.position)
        });

        self.enemies.retain(|_, enemy| {
            enemy.tick(dt);
            !enemy.should_delete()
        });

        self.particles.retain(|_, particle| {
            particle.tick(dt);
            !particle.should_delete()
        });
    }
}
