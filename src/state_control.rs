use std::{error::Error, fmt::Display};

use soloud::{Wav, AudioExt, LoadExt};

use crate::{game::GameState, sound::Music};

pub type LevelInfo = (f32, f32, &'static str);
pub type LevelLoader = fn(&mut GameState) -> LevelInfo;

#[derive(Debug)]
pub enum EparState {
    MainMenu,
    InGame
}

#[derive(strum_macros::EnumIter, strum_macros::EnumCount, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EparLevel {
    MoonlightSonata,
    FriendlyFaithPlate,
    Smoke,
}
impl EparLevel {
    pub fn level(&self) -> LevelLoader {
        use crate::levels;
        match self {
            EparLevel::Smoke => levels::smoke,
            EparLevel::MoonlightSonata => levels::moonlight_sonata,
            EparLevel::FriendlyFaithPlate => levels::friendly_faith_plate
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            EparLevel::FriendlyFaithPlate => "Friendly Faith Plate",
            EparLevel::MoonlightSonata => "Moonlight Sonata Remix (Encore Drop)",
            EparLevel::Smoke => "Smoke"
        }
    }
}
impl Display for EparLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
