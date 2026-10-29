mod app_state;
mod atomic;
mod backdrop;
pub mod cart_shell;
mod config;
mod core;
mod focus;
mod font;
pub mod gb;
mod gb_palette;
mod gba;
pub mod ini;
pub mod md;
pub mod music;
pub mod nds;
mod platform;
pub mod ps1;
mod region;
mod ring;
mod scan;
pub mod snes;
mod stamp;
mod stick_lights;
mod theme;
mod title;

pub use app_state::{
    read_app_state, write_app_state, AppState, BLUE_LIGHT_MAX, BRIGHTNESS_MAX, EQ_BANDS, EQ_DB_MAX,
    FF_SPEEDS, FF_SPEED_DEFAULT, RUMBLE_LEVEL_MAX, SLEEP_IDLE_CHOICES, SLEEP_OFF_CHOICES,
    UTC_OFFSET_MAX, UTC_OFFSET_MIN, VOLUME_MAX,
};
pub use atomic::{atomic_write, atomic_write_dated, sweep_temps};
pub use backdrop::{colour_index, BackdropStyle, BACKDROP_COLOURS, NIGHT};
pub use cart_shell::{
    key as cart_shell_key, Outline, ShellChoice, ShellFinish, CART_SHELL_FILE, LABELS_SHELL_FILE,
};
pub use config::{move_config, CONFIG_DIR};
pub use core::{
    core_for, core_for_platform, read_selected_cores, write_selected_core, Core, SELECTED_CORE_FILE,
};
pub use focus::{read_focus, write_focus, FOCUS_FILE, FOCUS_MAX};
pub use font::Font;
pub use gb_palette::{GbColors, GbPalette};
pub use gba::{header_clean, header_code, header_title};
pub use platform::Platform;
pub use region::Region;
pub use ring::{StateEntry, StateRing, RING_MAX};
pub use scan::{initial, is_hidden, scan, sort_key, Cart, StoreError};
pub use stamp::{
    civil_from_days, days_from_civil, days_in_month, format_stamp, parse_stamp, stamp_now,
};
pub use stick_lights::{light_colour_index, StickLights, LIGHT_COLOURS, LIGHT_LEVEL_MAX};
pub use theme::{Theme, THEME_FILE};
pub use title::{clean_title, TITLE_MAX};
