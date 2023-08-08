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
    Sparkler
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
            EparLevel::Sparkler => levels::sparkler
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
            EparLevel::Sparkler => "Sparkler"
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
                Sparkler
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
