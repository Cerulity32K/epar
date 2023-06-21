
use std::error::Error;

use macroquad::{prelude::{Vec2, Color, is_key_down, KeyCode, vec2, is_key_pressed, RED, SKYBLUE, WHITE}, window::{screen_width, screen_height, clear_background}, shapes::{draw_circle, draw_rectangle}, rand::gen_range, text::draw_text};
use soloud::{Wav, AudioExt, LoadExt};

use crate::{game_objects::Obstacle, utils::{mix, centered_text_draw, acmul}, state_control::{EparLevel, EparState}, sound::Music};

use super::game_objects::{Player, Obst};

pub fn soft_pink() -> Color { Color { r: 1.0, g: 0.5, b: 0.8, a: 1.0 } }
pub fn hit_color() -> Color { mix(soft_pink(), RED, 0.5) }
pub fn dash_color() -> Color { mix(soft_pink(), SKYBLUE, 0.5) }
pub fn hitdash_color() -> Color { mix(hit_color(), dash_color(), 0.5) }

/// Will lag the game INTENSELY. Basically enables a "shader" (on the CPU!) for debugging collisions, not for actual use.
pub const COLLISION_DBG: bool = false;
/// When `COLLISION_DBG` is enabled, specifies the size of the rectangles used for collision debugging.
pub const COLLISION_FRAGMENT_SIZE: usize = 20;

/// Extra arguments for specializing `StateModifier`s and `Accumulatee`s
#[derive(Default, Clone, Copy)]
pub struct ModifyArgs {
    pub step: usize,
    pub pos: Vec2,
    pub vel: Vec2,
    pub rad: f32
}
macro_rules! builder {
    ($name:tt: $type:ty) => {
        pub fn $name(mut self, $name: $type) -> Self { self.$name = $name; self }
    };
}
impl ModifyArgs {
    pub fn new() -> Self { Self::default() }
    builder!(step: usize);
    builder!(pos: Vec2);
    builder!(vel: Vec2);
    builder!(rad: f32);
}

pub struct UpdateAccumulator {
    obstacles_to_add: Vec<Obst>,
    events: Vec<Box<dyn StateModifier>>,
    jerk: Vec2,
    bg: Option<Color>,
    fg: Option<Color>,
    float: Option<f32>,
    shake: f32
}
impl UpdateAccumulator {
    pub fn new() -> Self {
        UpdateAccumulator {
            obstacles_to_add: vec![],
            events: vec![],
            jerk: Vec2::ZERO,
            bg: None,
            fg: None,
            float: None,
            shake: 0.0,
        }
    }
    pub fn obst(&mut self, obst: impl Obstacle) {
        self.obstacles_to_add.push(Obst::new(obst.box_clone()));
    }
    pub fn obstacle(&mut self, obst: Obst) {
        self.obstacles_to_add.push(obst);
    }
    pub fn jerk(&mut self, jerk: Vec2) {
        self.jerk += jerk;
    }
    pub fn shake(&mut self, shake: f32) {
        self.shake += shake;
    }
    pub fn bg(&mut self, bg: Color) {
        self.bg = Some(bg);
    }
    pub fn fg(&mut self, fg: Color) {
        self.fg = Some(fg);
    }
    pub fn float(&mut self, float: f32) {
        self.float = Some(float)
    }
    pub fn sm(&mut self, modifier: Box<dyn StateModifier>) {
        self.events.push(modifier);
    }
    pub fn smi(&mut self, modifier: impl StateModifier + 'static) {
        self.events.push(box modifier);
    }
}

pub trait StateModifier {
    fn run(&self, state: &mut GameState, _args: ModifyArgs);
    fn box_clone(&self) -> Box<dyn StateModifier>;
}

impl<T> StateModifier for T where T: Fn(&mut GameState, ModifyArgs) + Clone + 'static{
    fn box_clone(&self) -> Box<dyn StateModifier> {
        Box::new(self.clone())
    }
    fn run(&self, state: &mut GameState, sm: ModifyArgs) {
        self(state, sm)
    }
}

pub trait Accumulatee {
    fn run(&self, to_add: &mut UpdateAccumulator, _args: ModifyArgs);
    fn box_clone(&self) -> Box<dyn Accumulatee>;
}

impl<T> Accumulatee for T where T: Fn(&mut UpdateAccumulator, ModifyArgs) + Clone + 'static{
    fn box_clone(&self) -> Box<dyn Accumulatee> {
        Box::new(self.clone())
    }
    fn run(&self, to_add: &mut UpdateAccumulator, sm: ModifyArgs) {
        self(to_add, sm)
    }
}

#[derive(Clone, Copy)]
pub struct ClosureSM<T: Fn(&mut UpdateAccumulator) + Clone + 'static>(pub T);
impl<T: Fn(&mut UpdateAccumulator) + Clone> Accumulatee for ClosureSM<T> {
    fn box_clone(&self) -> Box<dyn Accumulatee> {
        Box::new(self.clone())
    }
    fn run(&self, gs: &mut UpdateAccumulator, _: ModifyArgs) {
        (self.0)(gs)
    }
}

pub struct GSEvent(pub f32, pub Box<dyn Accumulatee>);
impl GSEvent {
    pub fn time(mut self, time: f32) -> Self {
        self.0 = time;
        self
    }
}
impl Clone for GSEvent {
    fn clone(&self) -> Self {
        GSEvent(self.0, self.1.box_clone())
    }
}
pub struct GameState {
    events: Vec<GSEvent>,
    obsts: Vec<Obst>,
    time: f32,
    pub player: Player,
    pub hits_left: usize,
    pub fg_color: Box<dyn Fn(f32) -> Color>,
    pub bg_color: Box<dyn Fn(f32) -> Color>,
    pub cam_jerk: Vec2,
    pub cam_shake: f32,
    pub cam_float: f32,
    pub mus: Music,
    pub epar_state: EparState,
    pub bpm: f32,
    pub wav: Wav
}
impl GameState {
    pub fn set_fg_color(&mut self, clr: Color) {
        self.fg_color = Box::new(move|_|clr);
    }
    pub fn set_bg_color(&mut self, clr: Color) {
        self.bg_color = Box::new(move|_|clr);
    }
    pub fn new(mus: Music) -> Self {
        GameState {
            events: vec![],
            obsts: vec![],
            player: Player::default(),
            time: 0.0,
            hits_left: 3,
            fg_color: box |_|Color::new(1.0, 0.0, 0.5, 1.0),
            bg_color: box |_|Color::new(0.0, 0.0, 0.0, 1.0),
            cam_jerk: Vec2::ZERO,
            cam_shake: 0.0,
            cam_float: 0.0,
            bpm: 0.0,
            epar_state: EparState::MainMenu,
            mus,
            wav: Wav::default()
        }
    }
    pub fn load_level(&mut self, lvl: EparLevel, start: f32, speed: f32) -> Result<(), Box<dyn Error>> {
        self.events = vec![];
        self.wav = Wav::default();
        let (offset, bpm, audiofile) = lvl.level()(self);
        self.bpm = bpm;
        self.sort();
        self.wav.load(audiofile)?;
        self.mus.replace(&self.wav, bpm, offset / speed);
        self.mus.speed(speed);
        self.snip(start + offset);
        self.mus.seek(start / speed)?;
        Ok(())
    }
    pub fn reset(&mut self) {
        self.mus.stop();
        self.fg_color = box |_|Color::new(1.0, 0.0, 0.5, 1.0);
        self.bg_color = box |_|Color::new(0.0, 0.0, 0.0, 1.0);
        self.cam_float = 0.0;
        self.cam_jerk = Vec2::ZERO;
        self.cam_shake = 0.0;
        self.bpm = 0.0;
        self.wav = Wav::default();
        self.hits_left = 3;
        self.time = 0.0;
        self.events = vec![];
        self.obsts = vec![];
    }
    pub fn exit(&mut self) {
        self.mus.stop();
        self.epar_state = EparState::MainMenu;
    }
    pub fn add_event(&mut self, event: GSEvent) {
        self.events.push(event);
    }
    pub fn add_events(&mut self, events: impl IntoIterator<Item = GSEvent>) {
        self.events.append(&mut events.into_iter().collect::<Vec<GSEvent>>());
    }
    /// Call after initializing the events and before the update loop.
    pub fn sort(&mut self) {
        self.events.sort_by(|a, b|a.0.total_cmp(&b.0));
    }
    /// Cuts out events before `time`.
    pub fn snip(&mut self, time: f32) {
        // more performant to do a reverse removal loop
        if self.events.len() == 0 { return; }
        let mut i = self.events.len() - 1;
        while i > 0 {
            if self.events[i].0 < time {
                self.events.remove(i);
            }
            i -= 1;
        }
    }
    pub fn clear_events(&mut self) {
        self.events.clear();
    }
    pub fn update(&mut self, mus_time: f32, frame_time: f32) {
        if is_key_pressed(KeyCode::Escape) {
            self.reset();
            return;
        }
        self.time = mus_time;
        let smargs = ModifyArgs::default();
        let mut accum = UpdateAccumulator::new();
        'event_calls: loop {
            if self.events.len() > 0 && self.events[0].0 <= mus_time {
                let ev = self.events.remove(0);
                ev.1.run(&mut accum, smargs);
            } else {
                break 'event_calls;
            }
        }
        if self.player.dash > 0.0 {
            self.player.pps = 800.0;
            self.player.dash -= frame_time;
        } else {
            self.player.pps = 300.0;
        }
        if self.player.isecs > 0.0 {
            self.player.isecs -= frame_time;
        }
        if is_key_down(KeyCode::W) { self.player.pos.y -= self.player.pps * frame_time; }
        if is_key_down(KeyCode::S) { self.player.pos.y += self.player.pps * frame_time; }
        if is_key_down(KeyCode::A) { self.player.pos.x -= self.player.pps * frame_time; }
        if is_key_down(KeyCode::D) { self.player.pos.x += self.player.pps * frame_time; }
        if self.player.dash <= 0.0 && is_key_pressed(KeyCode::Space) { self.player.dash = 0.3; }
        self.cam_jerk *= 0.8;
        self.cam_shake *= 0.95;

        let mut i = 0;
        while i < self.obsts.len() {
            self.obsts[i].obstacle.update(&mut accum, frame_time, frame_time / 60.0 * self.bpm * self.mus.get_speed());
            i += 1;
        }
        for obst in &self.obsts {
            if self.player.dash <= 0.0 && self.player.isecs <= 0.0 && obst.obstacle.collides(self.player) {
                self.player.isecs = 2.0;
                println!("hit {}", self.hits_left);
                if self.hits_left > 0 {
                    self.hits_left -= 1;
                }
            }
        }
        let mut idx = 0;
        while idx < self.obsts.len() {
            if self.obsts[idx].marked_for_removal || self.obsts[idx].obstacle.should_kill() {
                self.obsts.swap_remove(idx).obstacle.kill(&mut accum);
            } else {
                idx += 1;
            }
        }
        self.obsts.append(&mut accum.obstacles_to_add);
        self.cam_jerk += accum.jerk;
        self.cam_shake += accum.shake;
        if let Some(fg) = accum.fg { self.set_fg_color(fg); }
        if let Some(bg) = accum.bg { self.set_bg_color(bg); }
        if let Some(float) = accum.float { self.cam_float = float; }
        for i in accum.events {
            i.run(self, smargs);
        }
    }
    pub fn draw(&mut self) {
        let offset = self.cam_jerk
            + vec2(gen_range(-self.cam_shake, self.cam_shake), gen_range(-self.cam_shake, self.cam_shake))
            + vec2((self.time).sin(), (self.time * 1.2).sin()) * self.cam_float;
        clear_background((self.bg_color)(self.time));
        for obst in &mut self.obsts {
            obst.obstacle.draw((self.fg_color)(self.time), offset);
        }
        let color = match (self.player.isecs > 0.0, self.player.dash > 0.0) {
            (false, false) => soft_pink(),
            (true, false) => hit_color(),
            (false, true) => dash_color(),
            (true, true) => hitdash_color()
        };
        draw_circle(self.player.pos.x + offset.x, self.player.pos.y + offset.y, self.player.rad, color);
        let tpos = self.player.pos + offset + vec2(-self.player.rad, -self.player.rad * 2.0);
        draw_text(&format!("{}", self.hits_left), tpos.x, tpos.y, self.player.rad * 5.0, WHITE);
        if COLLISION_DBG {
            for x in (0..screen_width() as usize).step_by(COLLISION_FRAGMENT_SIZE) {
                for y in (0..screen_height() as usize).step_by(COLLISION_FRAGMENT_SIZE) {
                    for i in &self.obsts {
                        if i.obstacle.collides(Player {
                            pos: vec2(x as f32, y as f32),
                            ..self.player
                        }) {
                            draw_rectangle(x as f32, y as f32, COLLISION_FRAGMENT_SIZE as f32, COLLISION_FRAGMENT_SIZE as f32, acmul(RED, 0.5));
                        }
                    }
                }
            }
        }
    }
    pub fn add_obstacle(&mut self, obst: Obst) {
        self.obsts.push(obst);
    }
    pub fn add_obst(&mut self, obst: impl Obstacle + 'static) {
        self.obsts.push(Obst::new(Box::new(obst)))
    }
}
