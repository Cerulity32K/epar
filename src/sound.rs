#![allow(dead_code)]
#![allow(unused_variables)]
use std::sync::{Arc, Mutex};

type ThreadSafe<T> = Arc<Mutex<T>>;

use soloud::{Soloud, AudioExt, Handle, SoloudError};

pub struct SfxCreator {
    sl: ThreadSafe<Soloud>
}
impl SfxCreator {
    pub fn new(sl: ThreadSafe<Soloud>) -> Self { SfxCreator { sl } }
    pub fn spawn_sfx(&self, sfx: &impl AudioExt) -> Handle { self.sl.lock().unwrap().play(sfx) }
}

pub struct Music {
    sl: ThreadSafe<Soloud>,
    handle: Option<Handle>,
    bpm: f32,
    offset: f32,
    sought: f32,
    speed: f32,
}
impl Music {
    pub fn new(sl: ThreadSafe<Soloud>) -> Self {
        Music { sl, handle: None, bpm: 0.0, offset: 0.0, sought: 0.0, speed: 1.0 }
    }
    pub fn replace(&mut self, new_music: &impl AudioExt, bpm: f32, offset: f32) -> Handle {
        if let Some(handle) = self.handle { self.sl.lock().unwrap().stop(handle); }
        let handle = self.sl.lock().unwrap().play(new_music);
        //self.sl.lock().unwrap().seek(handle, offset as f64 * self.bpm as f64 / 60.0);
        self.handle = Some(handle);
        self.bpm = bpm;
        self.offset = offset;
        self.sought = 0.0;
        handle
    }
    pub fn speed(&mut self, speed: f32) -> Option<Result<(), SoloudError>> {
        self.speed = speed;
        if let Some(handle) = self.handle {
            let mut guard = self.sl.lock().unwrap();
            Some(guard.set_relative_play_speed(handle, speed))
        } else {
            None
        }
    }
    pub fn get_speed(&self) -> f32 { self.speed }
    pub fn stop(&mut self) -> Option<Handle> {
        if let Some(handle) = self.handle { self.handle = None; Some(handle) }
        else { None }
    }
    pub fn current_beat(&self) -> Option<f32> {
        match self.handle {
            Some(h) => {
                let sl = self.sl.lock().unwrap();
                let sr = sl.samplerate(h);
                let buf_size = sl.backend_buffer_size() as f32;
                let offset = buf_size / sr;
                
                let beat = ((sl.stream_time(h) as f32 + offset) * self.bpm / 60.0 + self.offset + self.sought) * self.speed;
                Some(beat)
            }
            None => None
        }
    }
    pub fn check(&mut self) {
        if let Some(handle) = self.handle {
            if !self.sl.lock().unwrap().is_valid_voice_handle(handle) {
                self.handle = None;
            }
        }
    }
    pub fn is_playing(&self) -> bool {
        if let Some(handle) = self.handle {
            self.sl.lock().unwrap().is_valid_voice_handle(handle)
        } else {
            false
        }
    }
    pub fn seek(&mut self, beats: f32) -> Result<(), SoloudError> {
        if let Some(h) = self.handle {
            let sl = self.sl.lock().unwrap();
            sl.seek(h, (beats * 60.0 / self.bpm) as f64)?;
            self.sought += beats;
        }
        Ok(())
    }
}
