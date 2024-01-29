
use std::error::Error;

use macroquad::{prelude::{Vec2, Color, is_key_down, KeyCode, vec2, is_key_pressed, RED, SKYBLUE, WHITE}, window::{screen_width, screen_height, clear_background}, shapes::{draw_circle, draw_rectangle}, rand::gen_range, text::draw_text, miniquad::log::Level};
use soloud::{Wav, AudioExt, LoadExt};

use crate::{game_objects::Obstacle, utils::{mix, centered_text_draw, acmul}, state_control::{EparLevel, EparState, ColorChange}, sound::Music};

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
    pub rad: f32,
    pub time: f32
}
macro_rules! builder {
    ($name:tt: $type:ty) => {
        pub fn $name(mut self, $name: $type) -> Self { self.$name = $name; self }
    };
}
impl ModifyArgs {
    pub fn new(time: f32) -> Self { ModifyArgs { time, ..Self::default() } }
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
    shake: f32,
    time: f32,
}
impl UpdateAccumulator {
    pub fn time(&self) -> f32 {
        self.time
    }
    pub fn new() -> Self {
        UpdateAccumulator {
            obstacles_to_add: vec![],
            events: vec![],
            jerk: Vec2::ZERO,
            bg: None,
            fg: None,
            float: None,
            shake: 0.0,
            time: 0.0
        }
    }
    pub fn obst(&mut self, obst: impl Obstacle) {
        self.obstacles_to_add.push(Obst::new(obst.box_clone(), self.time));
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
    pub fn bg_raw(&mut self, bg: Box<dyn ColorEase>) {
        self.smi(ColorChange::bg(bg));
    }
    pub fn fg_raw(&mut self, fg: Box<dyn ColorEase>) {
        self.smi(ColorChange::fg(fg));
    }
    pub fn float(&mut self, float: f32) {
        self.float = Some(float)
    }
    pub fn sm(&mut self, modifier: Box<dyn StateModifier>) {
        self.events.push(modifier);
    }
    pub fn smi(&mut self, modifier: impl StateModifier + 'static) {
        self.events.push(Box::new(modifier));
    }
}

pub trait ColorEase {
    fn apply(&self, time: f32) -> Color;
    fn box_clone(&self) -> Box<dyn ColorEase>;
}
impl<T: Fn(f32) -> Color + Clone + 'static> ColorEase for T {
    fn apply(&self, time: f32) -> Color {
        self(time)
    }
    fn box_clone(&self) -> Box<dyn ColorEase> { Box::new(self.clone()) }
}

pub trait StateModifier {
    fn run(&self, state: &mut GameState, _args: ModifyArgs);
    fn box_clone(&self) -> Box<dyn StateModifier>;
}

impl<T> StateModifier for T where T: Fn(&mut GameState, ModifyArgs) + Clone + 'static {
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

impl<T> Accumulatee for T where T: Fn(&mut UpdateAccumulator, ModifyArgs) + Clone + 'static {
    fn box_clone(&self) -> Box<dyn Accumulatee> {
        Box::new(self.clone())
    }
    fn run(&self, to_add: &mut UpdateAccumulator, sm: ModifyArgs) {
        self(to_add, sm)
    }
}

pub struct GSEvent(pub f32, pub Box<dyn Accumulatee>);
impl GSEvent {
    pub fn new(time: f32, ev: impl Accumulatee + 'static) -> Self {
        GSEvent(time, Box::new(ev))
    }
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
pub struct LevelState {
    events: Vec<GSEvent>,
    obsts: Vec<Obst>,
    time: f32,
    pub player: Player,
    pub hits_left: usize,
    pub fg_color: Box<dyn ColorEase>,
    pub bg_color: Box<dyn ColorEase>,
    pub cam_jerk: Vec2,
    pub cam_shake: f32,
    pub cam_float: f32,
}
impl LevelState {
    pub fn new() -> Self {
        LevelState {
            events: vec![],
            obsts: vec![],
            player: Player::default(),
            time: 0.0,
            hits_left: 3,
            fg_color: Box::new(|_|Color::new(1.0, 0.0, 0.5, 1.0)),
            bg_color: Box::new(|_|Color::new(0.0, 0.0, 0.0, 1.0)),
            cam_jerk: Vec2::ZERO,
            cam_shake: 0.0,
            cam_float: 0.0,
        }
    }
}
pub struct GameState {
    pub state: EparState,
    pub mus: Music,
    pub bpm: f32,
    pub wav: Wav
}
impl GameState {
    pub fn set_fg_color(&mut self, clr: Color) {
        self.state.map(|s|s.fg_color = Box::new(move|_|clr));
    }
    pub fn set_bg_color(&mut self, clr: Color) {
        self.state.map(|s|s.bg_color = Box::new(move|_|clr));
    }
    pub fn new(mus: Music) -> Self {
        GameState {
            bpm: 0.0,
            state: EparState::MainMenu,
            mus,
            wav: Wav::default()
        }
    }
    pub fn load_level(&mut self, lvl: EparLevel, start: f32, speed: f32) -> Result<(), Box<dyn Error>> {
        let state = LevelState::new();
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
        self.state.map(|s| {
            s.fg_color = Box::new(|_|Color::new(1.0, 0.0, 0.5, 1.0));
            s.bg_color = Box::new(|_|Color::new(0.0, 0.0, 0.0, 1.0));
            s.cam_float = 0.0;
            s.cam_jerk = Vec2::ZERO;
            s.cam_shake = 0.0;
            s.hits_left = 3;
            s.time = 0.0;
            s.events = vec![];
            s.obsts = vec![];
        });
        self.bpm = 0.0;
        self.wav = Wav::default();
    }
    pub fn exit(&mut self) {
        self.mus.stop();
        self.state = EparState::MainMenu;
    }
    pub fn add_event(&mut self, event: GSEvent) {
        self.state.map(|s|s.events.push(event));
    }
    pub fn event<T: Accumulatee + 'static>(&mut self, time: f32, event: T) {
        self.state.map(|s|s.events.push(GSEvent(time, Box::new(event))));
    }
    pub fn instantly(&mut self, event: impl Accumulatee + 'static) {
        self.add_event(GSEvent(f32::NEG_INFINITY, Box::new(event)))
    }
    pub fn add_events(&mut self, events: impl IntoIterator<Item = GSEvent>) {
        self.state.map(|s|s.events.append(&mut events.into_iter().collect::<Vec<GSEvent>>()));
    }
    /// Call after initializing the events and before the update loop.
    pub fn sort(&mut self) {
        self.state.map(|s|s.events.sort_by(|a, b|a.0.total_cmp(&b.0)));
    }
    /// Cuts out events before `time`.
    pub fn snip(&mut self, time: f32) {
        // more performant to do a reverse removal loop
        self.state.map(|s| {
            if s.events.len() == 0 { return; }
            let mut i = s.events.len() - 1;
            while i > 0 {
                if s.events[i].0 < time {
                    s.events.remove(i);
                }
                i -= 1;
            }
        });
    }
    pub fn clear_events(&mut self) {
        self.state.map(|s|s.events.clear());
    }
    pub fn update(&mut self, mus_time: f32, frame_time: f32) {
        match &mut self.state {
            EparState::InGame(state) => {
                if is_key_pressed(KeyCode::Escape) {
                    self.reset();
                    return;
                }
                state.time = mus_time;
                let smargs = ModifyArgs::default();
                let mut accum = UpdateAccumulator::new();
                'event_calls: loop {
                    if state.events.is_empty() { break 'event_calls; }
                    let time = state.events[0].0;
                    if state.events.len() > 0 && time <= mus_time {
                        let ev = state.events.remove(0);
                        accum.time = time;
                        ev.1.run(&mut accum, smargs);
                    } else {
                        break 'event_calls;
                    }
                }
                if state.player.dash > 0.0 {
                    state.player.pps = 800.0;
                    state.player.dash -= frame_time;
                } else {
                    state.player.pps = 300.0;
                }
                if state.player.isecs > 0.0 {
                    state.player.isecs -= frame_time;
                }
                if is_key_down(KeyCode::W) { state.player.pos.y -= state.player.pps * frame_time; }
                if is_key_down(KeyCode::S) { state.player.pos.y += state.player.pps * frame_time; }
                if is_key_down(KeyCode::A) { state.player.pos.x -= state.player.pps * frame_time; }
                if is_key_down(KeyCode::D) { state.player.pos.x += state.player.pps * frame_time; }
                if state.player.dash <= 0.0 && is_key_pressed(KeyCode::Space) { state.player.dash = 0.3; }
                state.cam_jerk *= 0.8;
                state.cam_shake *= 0.95;
        
                accum.time = state.time;
        
                let mut i = 0;
                while i < state.obsts.len() {
                    let start = state.obsts[i].start_time;
                    let dt = frame_time / 60.0 * self.bpm * self.mus.get_speed();
                    let t = state.time - start;
                    state.obsts[i].obstacle.update(&mut accum, dt, t, dt, t);
                    i += 1;
                }
                for obst in &state.obsts {
                    if state.player.dash <= 0.0 && state.player.isecs <= 0.0 && obst.obstacle.collides(state.player) {
                        state.player.isecs = 2.0;
                        println!("hit {}", state.hits_left);
                        if state.hits_left > 0 {
                            state.hits_left -= 1;
                        }
                    }
                }
                let mut idx = 0;
                while idx < state.obsts.len() {
                    if state.obsts[idx].marked_for_removal || state.obsts[idx].obstacle.should_kill() {
                        state.obsts.swap_remove(idx).obstacle.kill(&mut accum);
                    } else {
                        idx += 1;
                    }
                }
                state.obsts.append(&mut accum.obstacles_to_add);
                state.cam_jerk += accum.jerk;
                state.cam_shake += accum.shake;
                if let Some(fg) = accum.fg { state.fg_color = Box::new(move |_|fg); }
                if let Some(bg) = accum.bg { state.bg_color = Box::new(move |_|bg); }
                if let Some(float) = accum.float { state.cam_float = float; }
                for i in accum.events {
                    i.run(self, smargs);
                }
            }
            _ => {}
        }
    }
    pub fn draw(&mut self) {
        self.state.map(|s| {
            let offset = s.cam_jerk
                + vec2(gen_range(-s.cam_shake, s.cam_shake), gen_range(-s.cam_shake, s.cam_shake))
                + vec2((s.time).sin(), (s.time * 1.2).sin()) * s.cam_float;
            clear_background(s.bg_color.apply(s.time));
            for obst in &mut s.obsts {
                obst.obstacle.draw(s.fg_color.apply(s.time), offset);
            }
            let color = match (s.player.isecs > 0.0, s.player.dash > 0.0) {
                (false, false) => soft_pink(),
                (true, false) => hit_color(),
                (false, true) => dash_color(),
                (true, true) => hitdash_color()
            };
            draw_circle(s.player.pos.x + offset.x, s.player.pos.y + offset.y, s.player.rad, color);
            let tpos = s.player.pos + offset + vec2(-s.player.rad, -s.player.rad * 2.0);
            draw_text(&format!("{}", s.hits_left), tpos.x, tpos.y, s.player.rad * 5.0, WHITE);
            if COLLISION_DBG {
                for x in (0..screen_width() as usize).step_by(COLLISION_FRAGMENT_SIZE) {
                    for y in (0..screen_height() as usize).step_by(COLLISION_FRAGMENT_SIZE) {
                        for i in &s.obsts {
                            if i.obstacle.collides(Player {
                                pos: vec2(x as f32, y as f32),
                                ..s.player
                            }) {
                                draw_rectangle(x as f32, y as f32, COLLISION_FRAGMENT_SIZE as f32, COLLISION_FRAGMENT_SIZE as f32, acmul(RED, 0.5));
                            }
                        }
                    }
                }
            }
        });
    }
    pub fn add_obst(&mut self, obst: Obst) {
        self.state.map(|s|s.obsts.push(obst));
    }
    pub fn add_obstacle(&mut self, obst: impl Obstacle + 'static, time: f32) {
        self.state.map(|s|s.obsts.push(Obst::new(Box::new(obst), time)));
    }
}
