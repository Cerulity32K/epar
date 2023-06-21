use std::f32::consts::TAU;

use macroquad::{prelude::{Vec2, Rect, Color, WHITE, vec2}, shapes::{draw_circle, draw_line, draw_triangle}, window::{screen_height, screen_width}, rand::gen_range};
use perlin2d::PerlinNoise2D;
use rand::{seq::SliceRandom, thread_rng};

use crate::{utils::{sq, self, collide_cr, mix, draw_rrect, collide_cc, screen_center, acmul, circ_climb, adjust, screen_size}, game::{Accumulatee, ModifyArgs, UpdateAccumulator}};

use super::game::GameState;

/// Creates a function to make an obstacle in builder style.\
/// Implemented at the creation of Smoke. Not used in many game objects.
macro_rules! builder {
    ($name:ident: $type:ty) => {
        pub fn $name(mut self, $name: $type) -> Self { self.$name = $name; self }
    };
}
macro_rules! vec2_builder {
    ($fname:ident -> $vname:ident) => {
        pub fn $fname(mut self, x: f32, y: f32) -> Self { self.$vname = macroquad::prelude::Vec2::new(x, y); self }
    };
}

/// Traits cannot hold members, so Obst contains markers (e.g. manual removal)
pub struct Obst {
    pub obstacle: Box<dyn Obstacle>,
    pub marked_for_removal: bool,
}
impl Obst {
    pub fn new(obst: Box<dyn Obstacle>) -> Self {
        Obst { obstacle: obst, marked_for_removal: false }
    }
}
impl Clone for Obst {
    fn clone(&self) -> Self {
        Obst { obstacle: self.obstacle.box_clone(), ..*self }
    }
}

#[derive(Clone, Copy)]
pub struct Player {
    pub pos: Vec2,
    pub rad: f32,
    pub pps: f32,
    pub dash: f32,
    pub isecs: f32,
}
impl Default for Player {
    fn default() -> Self {
        Player { pos: Vec2::new(screen_width() / 2.0, screen_height() / 2.0), rad: 5.0, pps: 100.0, dash: 0.0, isecs: 0.0 }
    }
}
pub trait Obstacle {
    fn update(&mut self, to_add: &mut UpdateAccumulator, frame_time: f32, beat_delta: f32);
    fn draw(&self, color: Color, offset: Vec2);
    fn box_clone(&self) -> Box<dyn Obstacle>;
    fn collides(&self, player: Player) -> bool;
    fn should_kill(&mut self) -> bool;
    /// Called before dropping. Use to trigger behaviour on death (e.g. bombs).
    fn kill(&mut self, to_add: &mut UpdateAccumulator) {}
}
#[derive(Clone, Copy)]
pub struct Pellet {
    pub pos: Vec2,
    pub vel: Vec2,
    pub rad: f32
}
impl Pellet {
    pub fn new(pos: Vec2, vel: Vec2, rad: f32) -> Self {
        Pellet { pos, vel, rad }
    }
}
impl Obstacle for Pellet {
    fn box_clone(&self) -> Box<dyn Obstacle> { Box::new(*self) }
    fn collides(&self, player: Player) -> bool {
        self.pos.distance_squared(player.pos) <= sq(self.rad + player.rad)
    }
    fn draw(&self, color: Color, offset: Vec2) {
        draw_circle(self.pos.x + offset.x, self.pos.y + offset.y, self.rad, color);
    }
    fn should_kill(&mut self) -> bool {
        !Rect::new(-self.rad, -self.rad, screen_width() + self.rad, screen_height() + self.rad).contains(self.pos)
    }
    fn update(&mut self, to_add: &mut UpdateAccumulator, frame_time: f32, beat_delta: f32) {
        self.pos += self.vel * beat_delta;
    }
}

pub struct Bomb {
    pub start: Vec2,
    pub target: Vec2,
    pub time: f32,
    pub life: f32,
    pub pellets: usize,
    pub pellet_vel: f32,
    pub pellet_rad: f32,
    pub snappiness: f32,
    pub rad: f32,
    pub spawner: Box<dyn Accumulatee>
}
impl Bomb {
    pub fn new(start: Vec2, target: Vec2, lifetime: f32, pellets: usize, pellet_vel: f32, pellet_rad: f32, spawner: Box<dyn Accumulatee>) -> Self {
        Bomb {
            start,
            target,
            time: 0.0,
            life: lifetime,
            pellets,
            pellet_vel,
            pellet_rad,
            snappiness: 20.0 / lifetime,
            rad: 30.0 / lifetime,
            spawner
        }
    }
    pub fn pellet_spawner(gs: &mut UpdateAccumulator, args: ModifyArgs) {
        gs.obstacle(Obst::new(Box::new(Pellet {
            pos: args.pos,
            vel: args.vel,
            rad: args.rad
        })))
    }
    pub fn pos(&self, offset: Vec2) -> Vec2 {
        (self.start - self.target) / (self.time * self.snappiness + 1.0) + self.target + offset
    }
}
impl Clone for Bomb {
    fn clone(&self) -> Self {
        Bomb {
            spawner: self.spawner.box_clone(),
            ..*self
        }
    }
}
impl Obstacle for Bomb {
    fn update(&mut self, to_add: &mut UpdateAccumulator, frame_time: f32, beat_delta: f32) { self.time += beat_delta; }
    fn draw(&self, color: Color, offset: Vec2) {
        let pos = self.pos(offset);
        let size = self.time * self.rad;
        draw_circle(pos.x, pos.y, size, color);
        let rot = self.time * 3.0;
        let size_fac = 1.2;
        let c1 = vec2(rot.cos(), rot.sin()) * size * size_fac + pos;
        let c2 = vec2(-rot.sin(), rot.cos()) * size * size_fac + pos;
        let c3 = vec2(-rot.cos(), -rot.sin()) * size * size_fac + pos;
        let c4 = vec2(rot.sin(), -rot.cos()) * size * size_fac + pos;
        draw_triangle(c1, c2, c3, color);
        draw_triangle(c1, c4, c3, color);
    }
    fn box_clone(&self) -> Box<dyn Obstacle> { Box::new(self.clone()) }
    fn collides(&self, player: Player) -> bool {
        utils::collide_cc(self.pos(Vec2::ZERO), self.rad * self.time, player.pos, player.rad)
    }
    fn should_kill(&mut self) -> bool { self.time >= self.life }
    fn kill(&mut self, to_add: &mut UpdateAccumulator) {
        let pos = self.pos(Vec2::ZERO);
        for i in 0..self.pellets {
            let period = i as f32 / self.pellets as f32 * TAU;
            self.spawner.run(to_add, ModifyArgs::new().pos(pos).vel(Vec2 {
                x: period.sin() * self.pellet_vel,
                y: period.cos() * self.pellet_vel
            }).rad(self.pellet_rad));
        }
    }
}

#[derive(Clone, Copy)]
pub struct GrowLaser {
    pub start: Vec2,
    pub end: Vec2,
    pub thickness: f32,
    pub warning_time: f32,
    pub show_time: f32,
    pub current_time: f32,
    pub grow_time: f32,
    pub shown: bool,
    pub jerk: Vec2,
    pub fade_in: f32,
    pub fade_opacity: f32
}
impl GrowLaser {
    pub fn new(start: Vec2, end: Vec2, thickness: f32, warning_time: f32, show_time: f32, jerk: Vec2) -> Self {
        GrowLaser {
            start,
            end,
            thickness,
            warning_time,
            show_time,
            jerk,
            current_time: 0.0,
            shown: false,
            grow_time: 0.25,
            fade_opacity: 0.5,
            fade_in: warning_time
        }
    }
    builder!(fade_opacity: f32);
    builder!(fade_in: f32);
    pub fn grow_time(mut self, new_time: f32) -> Self {
        self.grow_time = new_time;
        self
    }
    /// Calculates smoothed thickness
    pub fn thick(&self) -> f32 {
        let total_time = self.warning_time + self.show_time;
        if (self.warning_time..=self.warning_time + self.grow_time).contains(&self.current_time) {
            self.thickness * (self.current_time - self.warning_time) / self.grow_time
        } else if self.current_time > total_time - self.grow_time {
            self.thickness * (self.current_time - total_time) / -self.grow_time + 1.0
        } else {
            self.thickness
        }
    }
}
impl Obstacle for GrowLaser {
    fn update(&mut self, accum: &mut UpdateAccumulator, frame_time: f32, beat_delta: f32) {
        self.current_time += beat_delta;
        if !self.shown && self.current_time >= self.warning_time {
            accum.jerk(self.jerk);
            self.shown = true;
        }
    }

    fn draw(&self, mut color: Color, offset: Vec2) {
        if self.current_time < self.warning_time {
            if self.current_time < self.fade_in {
                color.a = self.current_time / self.fade_in * self.fade_opacity;
            } else {
                color.a = self.fade_opacity;
            }
        }
        draw_line(self.start.x + offset.x, self.start.y + offset.y, self.end.x + offset.x, self.end.y + offset.y, self.thick(), color);
    }

    fn box_clone(&self) -> Box<dyn Obstacle> {
        Box::new(*self)
    }

    fn collides(&self, player: Player) -> bool {
        self.current_time >= self.warning_time && {
            let (center, size, rot) = utils::rectify_line(self.start, self.end, self.thick());
            utils::collide_cr(center, size, rot, player.pos, player.rad)
        }
    }

    fn should_kill(&mut self) -> bool {
        self.current_time >= self.warning_time + self.show_time
    }
}

#[derive(Clone, Copy)]
pub struct SlamLaser {
    pub start: Vec2,
    pub end: Vec2,
    pub thickness: f32,
    pub warning_time: f32,
    pub show_time: f32,
    pub current_time: f32,
    pub leave_time: f32,
    pub anticipation: f32,
    pub shown: bool,
    pub jerk: Vec2,
    pub shake: f32,
}
impl SlamLaser {
    pub fn new(start: Vec2, end: Vec2, thickness: f32, warning_time: f32, show_time: f32, anticipation: f32, jerk: Vec2, shake: f32) -> Self {
        SlamLaser {
            start,
            end,
            thickness,
            warning_time,
            show_time,
            jerk,
            shake,
            current_time: 0.0,
            shown: false,
            anticipation,
            leave_time: 2.0
        }
    }
    /// Will flash and fade out from white for `self.grow_time` beats, this function calculates the mix.
    pub fn color(&self, normal: Color) -> Color {
        if (self.warning_time..=self.warning_time + 0.5).contains(&self.current_time) {
            mix(WHITE, normal, (self.current_time - self.warning_time) / 0.5)
        } else {
            normal
        }
    }
    pub fn leave_time(mut self, new_time: f32) -> Self {
        self.leave_time = new_time;
        self
    }
    /// Calculates slam lerp factor (0-1)
    pub fn slam(&self) -> f32 {
        let total = self.warning_time + self.show_time;
        if self.current_time < self.warning_time {
            self.current_time / self.warning_time * self.anticipation
        } else if self.current_time > total - self.leave_time {
            let exit_point = total - self.leave_time;
            -sq((self.current_time - exit_point) / self.leave_time) + 1.0
        } else {
            1.0
        }
    }
}
impl Obstacle for SlamLaser {
    fn update(&mut self, accum: &mut UpdateAccumulator, frame_time: f32, beat_delta: f32) {
        self.current_time += beat_delta;
        if !self.shown && self.current_time >= self.warning_time {
            accum.jerk(self.jerk);
            accum.shake(self.shake);
            self.shown = true;
        }
    }

    fn draw(&self, mut color: Color, offset: Vec2) {
        let mut color = self.color(color);
        let end = self.start.lerp(self.end, self.slam());
        draw_line(self.start.x + offset.x, self.start.y + offset.y, end.x + offset.x, end.y + offset.y, self.thickness, color);
        if self.current_time < self.warning_time {
            color.a = self.current_time / self.warning_time * 0.5;
            draw_line(self.start.x + offset.x, self.start.y + offset.y, self.end.x + offset.x, self.end.y + offset.y, self.thickness, color);
        }
    }

    fn box_clone(&self) -> Box<dyn Obstacle> {
        Box::new(*self)
    }

    fn collides(&self, player: Player) -> bool {
        self.current_time >= self.warning_time && {
            let (center, size, rot) = utils::rectify_line(self.start, self.start.lerp(self.end, self.slam()), self.thickness);
            utils::collide_cr(center, size, rot, player.pos, player.rad)
        }
    }

    fn should_kill(&mut self) -> bool {
        self.current_time >= self.warning_time + self.show_time
    }
}

pub struct Periodic {
    pub modifier: Box<dyn Accumulatee>,
    pub time_mod: f32,
    pub time_div: usize,
    pub interval: f32,
    pub max_steps: usize,
}
impl Periodic {
    pub fn new(steps: usize, interval: f32, modifier: Box<dyn Accumulatee>) -> Self {
        Periodic {
            modifier,
            time_mod: 0.0,
            time_div: 0,
            interval,
            max_steps: steps
        }
    }
    pub fn rect_trail(rect_life: f32, warning_time: f32, grow_time: f32, positioner: impl Fn(usize) -> (Vec2, Vec2, f32) + Clone + 'static) -> Box<dyn Accumulatee> {
        Box::new(move |gs: &mut UpdateAccumulator, sm: ModifyArgs| {
            let (center, size, rot) = positioner(sm.step);
            gs.obst(RotatableRect {
                center,
                size,
                rot,
                warning_time,
                show_time: rect_life,
                current_time: 0.0,
                grow_time,
            })
        })
    }
    pub fn linear(rect_life: f32, warning_time: f32, grow_time: f32, start: Vec2, delta: Vec2, scale: Vec2, rot: f32) -> Box<dyn Accumulatee> {
        Self::rect_trail(rect_life, warning_time, grow_time, move |i| (start + delta * (i as f32 - 1.0), scale, rot))
    }
}
impl Obstacle for Periodic {
    fn box_clone(&self) -> Box<dyn Obstacle> {
        Box::new(Periodic {
            modifier: self.modifier.box_clone(),
            ..*self
        })
    }
    fn collides(&self, player: Player) -> bool { false }
    fn draw(&self, color: Color, offset: Vec2) { }
    fn should_kill(&mut self) -> bool {
        self.time_div >= self.max_steps
    }
    fn update(&mut self, game_state: &mut UpdateAccumulator, frame_time: f32, beat_delta: f32) {
        self.time_mod += beat_delta;
        while self.time_mod >= self.interval {
            self.modifier.run(game_state, ModifyArgs::new().step(self.time_div));
            self.time_mod -= self.interval;
            self.time_div += 1;
        }
    }
}

#[derive(Clone, Copy)]
pub struct RotatableRect {
    pub center: Vec2,
    pub size: Vec2,
    pub rot: f32,
    pub warning_time: f32,
    pub show_time: f32,
    pub current_time: f32,
    pub grow_time: f32,
}
impl RotatableRect {
    /// Calculates the animated size\
    /// `allow_oversize` specifies whether or not the size can overshoot `self.size`.
    pub fn size(&self, allow_oversize: bool) -> Vec2 {
        let total_time = self.warning_time + self.show_time;
        if allow_oversize && (self.warning_time..=self.warning_time + self.grow_time).contains(&self.current_time) {
            self.size * ((self.current_time - self.warning_time) / -self.grow_time + 2.0)
        } else if self.current_time >= total_time - self.grow_time {
            self.size * ((self.current_time - total_time - self.grow_time) / -self.grow_time - 1.0)
        } else {
            self.size
        }
    }
    /// Will flash and fade out from white for `self.grow_time` beats, this function calculates the mix.
    pub fn color(&self, normal: Color) -> Color {
        if (self.warning_time..=self.warning_time + self.grow_time).contains(&self.current_time) {
            mix(WHITE, normal, (self.current_time - self.warning_time) / self.grow_time)
        } else {
            normal
        }
    }
}
impl Obstacle for RotatableRect {
    fn box_clone(&self) -> Box<dyn Obstacle> {
        Box::new(self.clone())
    }
    fn collides(&self, player: Player) -> bool {
        self.current_time >= self.warning_time && collide_cr(self.center, self.size(false), self.rot, player.pos, player.rad)
    }
    fn draw(&self, mut color: Color, offset: Vec2) {
        color = self.color(color);
        if self.current_time < self.warning_time {
            color.a = self.current_time / self.warning_time * 0.5;
        }
        draw_rrect(self.center + offset, self.size(true), self.rot, color)
    }
    fn should_kill(&mut self) -> bool {
        self.current_time >= self.show_time + self.warning_time
    }
    fn update(&mut self, game_state: &mut UpdateAccumulator, frame_time: f32, beat_delta: f32) {
        self.current_time += beat_delta;
    }
}

#[derive(Clone, Copy)]
pub struct RotatingRect {
    pub center: Vec2,
    pub size: Vec2,
    pub rot: f32,
    pub warning_time: f32,
    pub show_time: f32,
    pub current_time: f32,
    pub grow_time: f32,
    pub rpb: f32,
}
impl RotatingRect {
    /// Calculates the animated size\
    /// `allow_oversize` specifies whether or not the size can overshoot `self.size`.
    pub fn size(&self) -> Vec2 {
        let total_time = self.warning_time + self.show_time;
        if self.current_time >= total_time - self.grow_time {
            self.size * ((self.current_time - total_time - self.grow_time) / -self.grow_time - 1.0)
        } else {
            self.size
        }
    }
    /// Will flash and fade out from white for `self.grow_time` beats, this function calculates the mix.
    pub fn color(&self, normal: Color) -> Color {
        if (self.warning_time..=self.warning_time + self.grow_time).contains(&self.current_time) {
            mix(WHITE, normal, (self.current_time - self.warning_time) / self.grow_time)
        } else {
            normal
        }
    }
    /// Starts showing at `rot` radians, spins at `rps` revolutions per second.
    pub fn rot(&self) -> f32 {
        self.rot + (self.current_time - self.warning_time) * self.rpb * TAU
    }
}
impl Obstacle for RotatingRect {
    fn box_clone(&self) -> Box<dyn Obstacle> {
        Box::new(self.clone())
    }
    fn collides(&self, player: Player) -> bool {
        self.current_time >= self.warning_time && collide_cr(self.center, self.size(), -self.rot(), player.pos, player.rad)
    }
    fn draw(&self, mut color: Color, offset: Vec2) {
        color = self.color(color);
        if self.current_time < self.warning_time {
            color.a = self.current_time / self.warning_time * 0.5;
        }
        draw_rrect(self.center + offset, self.size(), self.rot(), color)
    }
    fn should_kill(&mut self) -> bool {
        self.current_time >= self.show_time + self.warning_time
    }
    fn update(&mut self, game_state: &mut UpdateAccumulator, frame_time: f32, beat_delta: f32) {
        self.current_time += beat_delta;
    }
}

#[derive(Clone, Copy)]
pub struct PelletSpinner {
    // counting
    count: usize,
    max: usize,

    // timing
    phase: f32,
    period: f32,
    start_time: f32,

    // pellet
    rad: f32,
    speed: f32
}
impl PelletSpinner {
    pub fn run(&mut self, time: f32, cur_pos: Vec2, cur_rad: f32, to_add: &mut UpdateAccumulator) -> bool {
        if time >= self.start_time + self.period * self.count as f32 && self.count < self.max {
            self.count += 1;
            let circ = vec2(
                ((self.count as f32 / self.max as f32 + self.phase) * TAU).cos(),
                ((self.count as f32 / self.max as f32 + self.phase) * TAU).sin(),
            );
            to_add.obst(Pellet::new(cur_pos + circ * (cur_rad - self.rad), circ * self.speed, self.rad))
        }
        self.count >= self.max
    }
}

#[derive(Clone)]
pub struct SmokeProj {
    disp_amp: f32,
    disp_freq: f32,
    time: f32,
    rad: f32,
    pulse: f32,
    warning_time: f32,
    show_time: f32,
    leave_time: f32,
    disp_phase: f32,
    pub events: Vec<(f32, SmokeEvent)>,
    pellet_spinners: Vec<PelletSpinner>
}
impl Default for SmokeProj {
    fn default() -> Self {
        SmokeProj {
            disp_amp: 75.0,
            disp_freq: 1.0,
            time: 0.0,
            rad: 20.0,
            pulse: 0.0,
            warning_time: 1.0,
            show_time: 32.0,
            leave_time: 0.25,
            events: vec![],
            disp_phase: 0.0,
            pellet_spinners: vec![]
        }
    }
}
impl SmokeProj {
    pub fn new() -> SmokeProj {
        Self::default()
    }
    pub fn trackpos(&self, time: f32) -> Vec2 {
        //let time = circ_climb(time);
        // Perlin construction does zero extra logic; inexpensive
        let perlin = PerlinNoise2D::new(5, 2.0, 1.0, 0.5, 1.2, (1.0, 1.0), 0.0, 0);
        (vec2(
            perlin.get_noise((time * self.disp_freq) as f64, (time * self.disp_freq) as f64) as f32,
            perlin.get_noise(-(time * self.disp_freq) as f64, -(time * self.disp_freq) as f64) as f32
        ) * 0.5 + vec2(
            (time * 1.25 * self.disp_freq + self.disp_phase * TAU).sin(),
            (time * 1.25 * self.disp_freq + self.disp_phase * TAU).cos()
        )
        ) * self.disp_amp + screen_center()
    }
    builder!(disp_amp: f32);
    builder!(disp_freq: f32);
    builder!(disp_phase: f32);
    builder!(leave_time: f32);
    builder!(warning_time: f32);
    builder!(show_time: f32);
    pub fn smevs(mut self, mut events: impl IntoIterator<Item = (f32, SmokeEvent)>) -> Self {
        for i in events.into_iter() {
            self.events.push(i);
        }
        self
    }
    pub fn color(&self, color: Color, time: f32) -> Color {
        if time < self.warning_time {
            acmul(color, self.time / self.warning_time * 0.5)
        } else {
            mix(color, WHITE, self.pulse)
        }
    }
    pub fn size(&self, time: f32) -> f32 {
        self.rad * (self.pulse + 1.0) * if self.time - self.warning_time > self.show_time - self.leave_time {
            (self.warning_time + self.show_time - self.time) / self.leave_time
        } else {
            1.0
        }
    }
    pub fn sort(mut self) -> Self {
        self.events.sort_by(|(a, _), (b, _)|a.total_cmp(b));
        self
    }
    pub fn employ(&mut self, event: SmokeEvent, to_add: &mut UpdateAccumulator) {
        match event {
            SmokeEvent::Pulse => {
                self.pulse = 1.0;
                to_add.shake(10.0);
            },
            SmokeEvent::Lasers(count, phase) => {
                let start = self.trackpos(self.time + 1.0);
                for i in 0..count {
                    to_add.obst(SlamLaser::new(start, start + vec2(
                        ((i as f32 / count as f32 + phase) * TAU).cos(),
                        ((i as f32 / count as f32 + phase) * TAU).sin()
                    ) * 1250.0, 20.0, 1.0, 1.0, 0.05, Vec2::ZERO, 0.0).leave_time(0.5))
                }
            },
            SmokeEvent::Pellets(count, speed, rad, phase) => {
                let start = self.trackpos(self.time);
                for i in 0..count {
                    let circ = vec2(
                        ((i as f32 / count as f32 + phase) * TAU).cos(),
                        ((i as f32 / count as f32 + phase) * TAU).sin(),
                    );
                    to_add.obst(Pellet::new(start + circ * (self.rad - rad), circ * speed, rad))
                }
            },
            SmokeEvent::PelletSpinner(count, speed, rad, phase, ppb) => {
                self.pellet_spinners.push(PelletSpinner {
                    count: 0,
                    max: count,
                    phase,
                    period: 1.0 / ppb,
                    start_time: self.time,
                    rad,
                    speed
                })
            },
            SmokeEvent::SPulse(strength) => {
                self.pulse = 1.0;
                to_add.shake(strength);
            }
        }
    }
}
impl Obstacle for SmokeProj {
    fn update(&mut self, to_add: &mut UpdateAccumulator, frame_time: f32, beat_delta: f32) {
        self.time += beat_delta;
        self.pulse *= 0.975;
        while self.events.len() > 0 {
            if self.time - self.warning_time >= self.events[0].0 {
                self.employ(self.events[0].1, to_add);
                self.events.remove(0);
            } else {
                break;
            }
        }
        let mut i = 0;
        let pos = self.trackpos(self.time);
        while i < self.pellet_spinners.len() {
            if self.pellet_spinners[i].run(self.time, pos, self.rad, to_add) {
                self.pellet_spinners.remove(i);
            } else {
                i += 1;
            }
        }
    }
    fn draw(&self, color: Color, offset: Vec2) {
        let pos = self.trackpos(self.time) + offset;
        draw_circle(pos.x, pos.y, self.size(self.time), self.color(color, self.time));
    }
    fn box_clone(&self) -> Box<dyn Obstacle> { box self.clone() }
    fn collides(&self, player: Player) -> bool { collide_cc(self.trackpos(self.time), self.size(self.time), player.pos, player.rad) }
    fn should_kill(&mut self) -> bool {
        self.time > self.warning_time + self.show_time
    }
}
#[derive(Clone, Copy)]
pub enum SmokeEvent {
    Pulse,
    /// pulse strength
    SPulse(f32),
    /// count, phase
    Lasers(usize, f32),
    /// count, speed, rad, phase
    Pellets(usize, f32, f32, f32),
    /// count, speed, rad, phase, ppb
    PelletSpinner(usize, f32, f32, f32, f32)
}

pub const MOORE_OFFSETS: [(isize, isize); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1)
];
#[derive(Clone)]
pub struct GOLGrid {
    width: usize,
    height: usize,
    gol: Vec<bool>,
    moore_begin: [bool; 9],
    moore_stay: [bool; 9],

    ticks: usize,
    max: usize,
    period: f32,
    time: f32,
    warning_time: f32,
    first_warning_time: f32
}
impl Default for GOLGrid {
    fn default() -> Self {
        GOLGrid {
            width: 32,
            height: 18,
            gol: vec![false; 32 * 18],
            moore_begin: [false, false, false, true, false, false, true, false, false],
            moore_stay:  [false, false, true, true, false, false, false, true, false],

            ticks: 0,
            max: 32,
            period: 1.0,
            time: 0.0,
            warning_time: 0.0,
            first_warning_time: 1.0
        }
    }
}
impl GOLGrid {
    builder!(max: usize);
    builder!(period: f32);
    builder!(warning_time: f32);
    builder!(first_warning_time: f32);
    pub fn dims(mut self, w: usize, h: usize) -> Self {
        adjust(&mut self.gol, w * h, false);
        self.width = w;
        self.height = h;
        self
    }
    pub fn tick(&mut self) -> Vec<bool> {
        self.ticks += 1;
        let mut new = vec![false; self.width * self.height];
        for x in 0..self.width {
            for y in 0..self.height {
                if self.get_next(x, y) {
                    new[y * self.width + x] = true;
                }
            }
        }
        std::mem::replace(&mut self.gol, new)
    }
    pub fn get_next(&self, x: usize, y: usize) -> bool {
        if self.get(x as isize, y as isize) {
            self.moore_stay[self.neighbors(x, y)]
        } else {
            self.moore_begin[self.neighbors(x, y)]
        }
    }
    pub fn neighbors(&self, x: usize, y: usize) -> usize {
        let mut count = 0;
        for (ox, oy) in MOORE_OFFSETS {
            if self.get((x as isize + ox), (y as isize + oy)) {
                count += 1;
            }
        }
        count
    }
    pub fn get(&self, x: isize, y: isize) -> bool {
        if x < 0 || y < 0 { false } else {
            *self.gol.get(y as usize * self.width + x as usize).unwrap_or(&false)
        }
    }
    pub fn populate(mut self, count: usize) -> Self {
        let len = self.gol.len();
        for _ in 0..count {
            self.gol[gen_range(0, len)] = true;
        }
        self
    }
}
impl Obstacle for GOLGrid {
    fn update(&mut self, to_add: &mut UpdateAccumulator, frame_time: f32, beat_delta: f32) {
        self.time += beat_delta;
        let first = self.ticks == 0;
        if first || self.time > self.period * self.ticks as f32 + self.first_warning_time - self.warning_time {
            self.tick();
            let pfac = screen_size() / vec2(self.width as f32, self.height as f32);
            for x in 0..self.width {
                for y in 0..self.height {
                    if self.get(x as isize, y as isize) {
                        to_add.obst(RotatableRect {
                            center: vec2(x as f32, y as f32) * pfac + pfac / 2.0,
                            size: pfac,
                            rot: 0.0,
                            warning_time: if first {self.first_warning_time } else { self.warning_time },
                            show_time: self.period * 1.25,
                            current_time: 0.0,
                            grow_time: self.period / 4.0,
                        })
                    }
                }
            }
        }
    }
    fn draw(&self, color: Color, offset: Vec2) { }
    fn box_clone(&self) -> Box<dyn Obstacle> { box self.clone() }
    fn collides(&self, player: Player) -> bool { false }
    fn should_kill(&mut self) -> bool { self.ticks >= self.max }
}
