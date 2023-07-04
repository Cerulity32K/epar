
use macroquad::{window::{screen_width, screen_height}, prelude::{Vec2, rand::gen_range, vec2}};
use rand::{thread_rng, Rng};

use crate::game::{ModifyArgs, UpdateAccumulator};

use super::{game::{GameState, Accumulatee}, game_objects::{Bomb, Obst, GrowLaser}};

pub struct BombSideSpawner {
    pub pellets: usize,
    pub pellet_vel: f32,
    pub pellet_rad: f32,
    pub bomb_life: f32,
    pub spawner: Box<dyn Accumulatee>
}
impl BombSideSpawner {
    pub fn new(pellets: usize, pellet_vel: f32, pellet_rad: f32, bomb_life: f32) -> Self {
        BombSideSpawner {
            pellets,
            pellet_vel,
            pellet_rad,
            bomb_life,
            spawner: Box::new(move |gs: &mut UpdateAccumulator, args: ModifyArgs| Bomb::pellet_spawner(gs, args)),
        }
    }
    pub fn proj_spawner(mut self, spawner: Box<dyn Accumulatee>) -> Self {
        self.spawner = spawner;
        self
    }
}
impl Clone for BombSideSpawner {
    fn clone(&self) -> Self {
        BombSideSpawner {
            spawner: self.spawner.box_clone(),
            ..*self
        }
    }
}
impl Accumulatee for BombSideSpawner {
    fn box_clone(&self) -> Box<dyn Accumulatee> {
        Box::new(self.clone())
    }
    fn run(&self, gs: &mut UpdateAccumulator, _: ModifyArgs) {
        gs.obst(Bomb::new(
            Vec2 { x: screen_width(), y: gen_range(0.0, screen_height()) },
            Vec2 { x: screen_width() - 100.0, y: gen_range(0.0, screen_height()) },
            self.bomb_life, self.pellets, self.pellet_vel, self.pellet_rad, self.spawner.box_clone()
        ))
    }
}

pub struct MultiSpawner(pub Vec<Obst>);
impl Accumulatee for MultiSpawner {
    fn box_clone(&self) -> Box<dyn Accumulatee> { Box::new(self.clone()) }
    fn run(&self, gs: &mut UpdateAccumulator, _: ModifyArgs) {
        for i in &self.0 {
            gs.obstacle(i.clone());
        }
    }
}
impl Clone for MultiSpawner {
    fn clone(&self) -> Self {
        let mut new_obsts: Vec<Obst> = vec![];
        for i in &self.0 {
            new_obsts.push(i.clone());
        }
        MultiSpawner(new_obsts)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct HorLaserSpawner {
    pub warning_time: f32,
    pub show_time: f32,
    pub thickness: f32,
    pub jerk: f32,
}
impl HorLaserSpawner {
    pub fn new(warning_time: f32, show_time: f32, thickness: f32, jerk: f32) -> Self {
        HorLaserSpawner { warning_time, show_time, thickness, jerk }
    }
}
impl Accumulatee for HorLaserSpawner {
    fn box_clone(&self) -> Box<dyn Accumulatee> { Box::new(self.clone()) }
    fn run(&self, gs: &mut UpdateAccumulator, _: ModifyArgs) {
        let y = gen_range(0.0, screen_height());
        gs.obst(
            GrowLaser::new(vec2(-100.0, y), vec2(screen_width() + 100.0, y), self.thickness, self.warning_time, self.show_time, vec2(gen_range(-self.jerk, self.jerk), 0.0))
        );
    }
}
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct VertLaserSpawner {
    pub warning_time: f32,
    pub show_time: f32,
    pub thickness: f32,
    pub jerk: f32,
}
impl VertLaserSpawner {
    pub fn new(warning_time: f32, show_time: f32, thickness: f32, jerk: f32) -> Self {
        VertLaserSpawner { warning_time, show_time, thickness, jerk }
    }
}
impl Accumulatee for VertLaserSpawner {
    fn box_clone(&self) -> Box<dyn Accumulatee> { Box::new(self.clone()) }
    fn run(&self, gs: &mut UpdateAccumulator, _: ModifyArgs) {
        let x = gen_range(0.0, screen_width());
        gs.obst(
            GrowLaser::new(vec2(x, -100.0), vec2(x, screen_height() + 100.0), self.thickness, self.warning_time, self.show_time, vec2(0.0, gen_range(-self.jerk, self.jerk)))
        );
    }
}
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct LaserSpawner {
    pub warning_time: f32,
    pub show_time: f32,
    pub thickness: f32,
    pub jerk: f32,
}
impl LaserSpawner {
    pub fn new(warning_time: f32, show_time: f32, thickness: f32, jerk: f32) -> Self {
        LaserSpawner { warning_time, show_time, thickness, jerk }
    }
}
impl Accumulatee for LaserSpawner {
    fn box_clone(&self) -> Box<dyn Accumulatee> { Box::new(self.clone()) }
    fn run(&self, gs: &mut UpdateAccumulator, sm: ModifyArgs) {
        if thread_rng().gen_bool(0.5) {
            HorLaserSpawner::new(self.warning_time, self.show_time, self.thickness, self.jerk).run(gs, sm)
        } else {
            VertLaserSpawner::new(self.warning_time, self.show_time, self.thickness, self.jerk).run(gs, sm)
        }
    }
}
