//! Each menu screen's input and drawing, as `impl App` blocks: `App::apply` hands a press to
//! the screen it is on and `App::draw` draws the screen it shows, and what each screen does
//! with them lives in its own file here.

pub mod about;
pub mod background;
pub mod bar;
pub mod bluetooth;
pub mod cheevos;
pub mod clock;
pub mod controls;
pub mod equalizer;
pub mod info;
pub mod keyboard;
pub mod labels;
pub mod lone;
pub mod music;
pub mod music_play;
pub mod network;
pub mod picture_view;
pub mod power;
pub mod reader;
pub mod settings;
pub mod shelf;
pub mod sleep;
pub mod ssh;
pub mod video_play;
