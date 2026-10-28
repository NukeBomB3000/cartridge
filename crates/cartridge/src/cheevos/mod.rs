//! RetroAchievements, softcore only: the game in the core is followed for unlocks, which pop up
//! over it. Ported from nickclyde/slot-plus's and fragilesilver/slot-fragile's `cheevos`, with
//! the notices drawn in Cartridge's own pixel style.

pub mod boot;
mod live;
pub mod overlay;

pub use live::*;
