#![feature(unboxed_closures)]
#![feature(fn_traits)]

// Comment out to show all >50 warnings about unused stuff
#![allow(unused)]

use core::time;
use std::{sync::{Arc, Mutex}, error::Error};

use macroquad::{window::next_frame, prelude::*, time::get_frame_time};
use soloud::{Soloud, SoloudFlag, Backend, Wav, AudioExt, LoadExt};
use sound::Music;
use game::GameState;
use state_control::{EparState, EparLevel};
use strum::{IntoEnumIterator, EnumCount};
use utils::{screen_size, cmul};

mod sound;
mod utils;
mod levels;
mod game_objects;
mod spawners;
mod generators;
mod game;
mod state_control;

type AnyErr = Box<dyn Error>;
type Possibly<T> = Result<T, AnyErr>;
type CanErr = Possibly<()>;

//#[inline]
//fn beatify(sec: f32, bpm: f32) -> f32 { sec / 60.0 * bpm }

//#[inline]
//fn vec2((x, y): (f32, f32)) -> Vec2 { Vec2::new(x, y) }

#[macroquad::main("Exclusively Polygons Alonside Rhythms")]
async fn main() -> CanErr {
    request_new_screen_size(1600.0, 900.0);
    next_frame().await;
    let sl = Arc::new(Mutex::new(Soloud::new(SoloudFlag::empty(), Backend::Auto, 44100, 1024, 2)?));
    //let sfx = SfxCreator::new(sl.clone());
    let mut state = GameState::new(Music::new(sl.clone()));
    loop {
        match state.epar_state {
            EparState::MainMenu => {
                let show_unfinished = is_key_down(KeyCode::U);
                let lvls = EparLevel::iter().filter(move |lvl| show_unfinished || lvl.finished()).collect::<Vec<_>>();
                let length = lvls.len();
                let rect_height = screen_height() / length as f32;
                let rect_width = screen_width();
                let rect_padding = 50.0;
                let pos = mouse_position();
                let mouse_pos = vec2(pos.0, pos.1);
                'elit: for (idx, lvl) in lvls.into_iter().enumerate() {
                    let y_offset = -((length as f32 - 1.0) / 2.0 - idx as f32) * rect_height + screen_height() / 2.0;
                    let x_offset = screen_width() / 2.0;
                    let rsize = vec2(rect_width, rect_height) - rect_padding;
                    let rpos = vec2(x_offset, y_offset);
                    let r = Rect::new(x_offset - rsize.x / 2.0, y_offset - rsize.y / 2.0, rsize.x, rsize.y);
    
                    let color;
                    if r.contains(mouse_pos){
                        color = cmul(WHITE, 0.3);
                        if is_mouse_button_pressed(MouseButton::Left) {
                            macroquad::rand::srand((get_time() * 1_000_000.0) as u64);
                            state.reset();
                            state.load_level(lvl, 0.0, 1.0);
                            state.epar_state = EparState::InGame;
                            break 'elit;
                        }
                    } else {
                        color = cmul(WHITE, 0.1);
                    }
                    draw_rectangle(x_offset - rsize.x / 2.0, y_offset - rsize.y / 2.0, rsize.x, rsize.y, color);
                    let fsize = 40;
                    let txt = &format!("{lvl}");
                    let dims = measure_text(txt, None, fsize, 1.0);
                    draw_text(txt, x_offset - dims.width / 2.0, y_offset + dims.offset_y / 2.0, fsize as f32, if lvl.finished() { WHITE } else { RED });
                }
                next_frame().await;
            }
            EparState::InGame => {
                state.player.pos = vec2(0.125, 0.5) * screen_size();
                while state.mus.is_playing() {
                    state.mus.check();
                    if let Some(f) = state.mus.current_beat() {
                        let ft = get_frame_time();
                        state.update(f, ft);
                        //println!("{f:.2}");
                    }
                    state.draw();
                    next_frame().await;
                }
                state.epar_state = EparState::MainMenu;
            }
        }
    }
    Ok(())
}
