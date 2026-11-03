//! Game-specific data and simulation foundations.
//!
//! GPU rendering and window input live in the executable; world and player
//! logic can be tested without creating a window or graphics adapter.

pub mod geography;
pub mod material_contract;
pub mod player;
pub mod world;
