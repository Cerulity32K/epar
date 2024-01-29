use std::{error::Error, fmt::Display};

use macroquad::color::Color;
use soloud::{Wav, AudioExt, LoadExt};

use crate::{game::{GameState, LevelState, ColorEase, StateModifier, ModifyArgs}, sound::Music};

pub type LevelInfo = (f32, f32, &'static str);
pub type LevelLoader = fn(&mut GameState) -> LevelInfo;

pub enum EparState {
    MainMenu,
    InGame(LevelState)
}
impl EparState {
    pub fn map<R, F: FnOnce(&mut LevelState) -> R>(&mut self, map_fn: F) -> Option<R> {
        if let EparState::InGame(state) = self {
            Some(map_fn(state))
        } else { None }
    }
}

pub struct ColorChange {
    color: Box<dyn ColorEase>,
    is_fg: bool
}
impl ColorChange {
    pub fn fg(fg: Box<dyn ColorEase>) -> Self {
        Self { color: fg, is_fg: true }
    }
    pub fn bg(bg: Box<dyn ColorEase>) -> Self {
        Self { color: bg, is_fg: false }
    }
}
impl StateModifier for ColorChange {
    fn box_clone(&self) -> Box<dyn StateModifier> {
        Box::new(Self { color: self.color.box_clone(), is_fg: self.is_fg })
    }
    fn run(&self, state: &mut GameState, _: ModifyArgs) {
        state.state.map(|s|
            if self.is_fg {
                s.fg_color = self.color.box_clone();
            } else {
                s.bg_color = self.color.box_clone();
            }
        ).unwrap_or(())
    }
}

macro_rules! pat_lvls {
    ($($lvl:tt),+) => {
        $(
            EparLevel::$lvl
        )|+
    };
}

#[derive(strum_macros::EnumIter, strum_macros::EnumCount, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// Sorted by difficulty (easy -> hard)
pub enum EparLevel {
    MoonlightSonata,
    FriendlyFaithPlate,
    Inferno,
    Smoke,
    Granite,
    Isolation,
    Kocmoc,
    Sparkler,
    Firestarter
}
impl EparLevel {
    pub fn level(&self) -> LevelLoader {
        use crate::levels;
        match self {
            EparLevel::MoonlightSonata => levels::moonlight_sonata,
            EparLevel::FriendlyFaithPlate => levels::friendly_faith_plate,
            EparLevel::Smoke => levels::smoke,
            EparLevel::Granite => levels::granite,
            EparLevel::Inferno => levels::inferno,
            EparLevel::Isolation => levels::isolation,
            EparLevel::Kocmoc => levels::kocmoc,
            EparLevel::Sparkler => levels::sparkler,
            EparLevel::Firestarter => levels::firestarter
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            EparLevel::MoonlightSonata => "Moonlight Sonata (Meganeko Remix, Final Drop)",
            EparLevel::FriendlyFaithPlate => "Friendly Faith Plate",
            EparLevel::Smoke => "Smoke",
            EparLevel::Granite => "Granite",
            EparLevel::Inferno => "Inferno",
            EparLevel::Isolation => "Isolation (LIMBO Remix)",
            EparLevel::Kocmoc => "KOCMOC (Albee Remix)",
            EparLevel::Sparkler => "Sparkler",
            EparLevel::Firestarter => "Firestarter",
        }
    }
    /// Used to filter out levels that are under development
    pub fn finished(&self) -> bool {
        match self {
            pat_lvls!(
                Granite,
                Inferno,
                FriendlyFaithPlate,
                Isolation,
                Sparkler,
                Firestarter
            ) => false,
            _ => true
        }
    }
}
impl Display for EparLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
