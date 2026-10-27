use std::collections::HashSet;

use cartridge_store::{colour_index, BACKDROP_COLOURS};
use cartridge_ui::kit::palette::{palette, rgba, Palette};
use cartridge_ui::BACKDROP_PALETTE;

/// What assets/ui/foundation/palette.js gives for each backdrop colour, in palette.js's order:
/// --oz-bg, then the ramp darkest first. To make it again, run this from the repository's root
/// and paste what it prints:
///
/// ```text
/// node -e '
/// globalThis.window = globalThis;
/// globalThis.CustomEvent = class {};
/// globalThis.CartridgeBackdrop = { paint() {} };
/// globalThis.document = { documentElement: { style: { setProperty() {} }, dataset: {} }, dispatchEvent() {} };
/// eval(require("fs").readFileSync("assets/ui/foundation/palette.js", "utf8"));
/// for (const name of CartridgePalette.COLOURS) {
///   const hex = Object.values(CartridgePalette.palette(name)).map(c => `"${c}"`);
///   console.log(`("${name}", [${hex.join(", ")}]),`);
/// }'
/// ```
const PALETTE_JS: [(&str, [&str; 13]); 17] = [
    (
        "night",
        [
            "#050508", "#181818", "#494949", "#383838", "#595959", "#636363", "#7d7d7d", "#828282",
            "#9d9d9d", "#b7b7b7", "#c4c4c4", "#cfcfcf", "#e6e6e6",
        ],
    ),
    (
        "black",
        [
            "#000000", "#181818", "#494949", "#383838", "#595959", "#636363", "#7d7d7d", "#828282",
            "#9d9d9d", "#b7b7b7", "#c4c4c4", "#cfcfcf", "#e6e6e6",
        ],
    ),
    (
        "slate",
        [
            "#22252c", "#16181a", "#454a4f", "#35383c", "#56595e", "#60646a", "#797e85", "#7d828a",
            "#9a9ca6", "#b2b7c1", "#c0c5d0", "#cbcfd7", "#e5e6ea",
        ],
    ),
    (
        "crimson",
        [
            "#300f16", "#290d19", "#6e324c", "#52293a", "#7d4558", "#8f4b5f", "#b45f77", "#bc6080",
            "#df7b82", "#fe91a8", "#ffa9b7", "#ffb9c2", "#ffdcdc",
        ],
    ),
    (
        "coral",
        [
            "#3a1810", "#2b0d0e", "#723235", "#55292a", "#804642", "#924d45", "#b76157", "#c0625d",
            "#dd815e", "#ff9682", "#ffad9a", "#ffbcab", "#ffded0",
        ],
    ),
    (
        "brown",
        [
            "#2a1c12", "#22140e", "#604135", "#48322a", "#6e5245", "#7c5b4a", "#9b735d", "#a37661",
            "#bb9472", "#dcab8c", "#eeb893", "#ecc6ac", "#f5e2d1",
        ],
    ),
    (
        "amber",
        [
            "#302208", "#261300", "#6a3d01", "#4f300d", "#765020", "#845a1a", "#a57221", "#b0731b",
            "#be9732", "#e6ab47", "#f8b83f", "#f2c87d", "#f5e5bb",
        ],
    ),
    (
        "olive",
        [
            "#262a0c", "#1c1800", "#534a00", "#3f3907", "#5f5b1f", "#68681d", "#828326", "#8c8617",
            "#8fa94c", "#b7c053", "#c1d052", "#cad888", "#ddedc4",
        ],
    ),
    (
        "green",
        [
            "#102c16", "#0e1d04", "#34531c", "#2a3f1a", "#416436", "#42713d", "#538e4f", "#5a9348",
            "#53b37b", "#7bce82", "#7bdf8c", "#9de3aa", "#c7f2d8",
        ],
    ),
    (
        "mint",
        [
            "#0e2e26", "#021e10", "#185639", "#19412d", "#2d664e", "#29735a", "#339072", "#359670",
            "#39b3a1", "#59cfae", "#52e0bd", "#8ae3cc", "#c1f2e9",
        ],
    ),
    (
        "teal",
        [
            "#0b2b2e", "#001d1b", "#005551", "#0f403d", "#266463", "#227172", "#2b8d90", "#239492",
            "#47adbf", "#57cbd3", "#53dbe6", "#8cdfe8", "#c5eef8",
        ],
    ),
    (
        "sky",
        [
            "#0c2a3e", "#001b26", "#005169", "#003e52", "#1c617e", "#1c6c91", "#2687b7", "#028ebe",
            "#59a2e6", "#60c2ff", "#88cdff", "#a1d6ff", "#d4e8ff",
        ],
    ),
    (
        "navy",
        [
            "#101b36", "#04192e", "#1d4b78", "#1b3a59", "#385a86", "#416499", "#547ec0", "#4d84ca",
            "#8397ea", "#91b6ff", "#a9c4ff", "#b9ceff", "#dfe5ff",
        ],
    ),
    (
        "indigo",
        [
            "#1a1640", "#0e162f", "#34467a", "#29365a", "#495586", "#555e99", "#6d76c0", "#6a7ccb",
            "#9c8fe4", "#a9aeff", "#bbbeff", "#c8c9ff", "#e6e2ff",
        ],
    ),
    (
        "plum",
        [
            "#291431", "#1c122b", "#523c72", "#3e2f55", "#644d7d", "#73548c", "#9269af", "#946dbc",
            "#c182c9", "#d59df3", "#e5aaff", "#ebbafc", "#fbd9fb",
        ],
    ),
    (
        "wine",
        [
            "#300c22", "#240e22", "#643560", "#4b2b48", "#75476b", "#874d75", "#a96193", "#af649e",
            "#d77ba4", "#f293cd", "#ffa3d7", "#ffb5db", "#ffdbe7",
        ],
    ),
    (
        "rose",
        [
            "#36122a", "#240f23", "#633662", "#4a2b49", "#73476d", "#854e79", "#a76297", "#ac65a2",
            "#d57ba9", "#ef94d2", "#ffa1dd", "#ffb4e0", "#ffdae9",
        ],
    ),
];

/// A palette's colours in palette.js's order, each by the --oz-* property it is in the kit.
fn tokens(p: &Palette) -> [(&'static str, [u8; 3]); 13] {
    [
        ("--oz-bg", p.bg),
        ("--oz-dark", p.dark),
        ("--oz-edge", p.edge),
        ("--oz-well", p.well),
        ("--oz-plate", p.plate),
        ("--oz-tab", p.tab),
        ("--oz-band", p.band),
        ("--oz-stripe", p.stripe),
        ("--oz-dim", p.dim),
        ("--oz-frame", p.frame),
        ("--oz-frame-hi", p.frame_hi),
        ("--oz-label", p.label),
        ("--oz-cream", p.cream),
    ]
}

fn hex([r, g, b]: [u8; 3]) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

fn named(name: &str) -> &'static Palette {
    palette(colour_index(name).expect("a backdrop colour"))
}

#[test]
fn every_colour_is_the_one_palette_js_gives() {
    let mut wrong = Vec::new();
    for (colour, (name, want)) in PALETTE_JS.iter().enumerate() {
        assert_eq!(BACKDROP_COLOURS[colour], *name, "the table is out of order");
        for ((token, got), want) in tokens(palette(colour as u8)).into_iter().zip(want) {
            if hex(got) != *want {
                wrong.push(format!("{name}'s {token} is {}, not {want}", hex(got)));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn each_palette_sits_on_the_backdrop_the_shelf_is_drawn_on() {
    for (colour, rgb) in BACKDROP_PALETTE.into_iter().enumerate() {
        let name = BACKDROP_COLOURS[colour];
        assert_eq!(palette(colour as u8).bg, rgb, "{name}");
    }
}

#[test]
fn amber_s_darkest_brown_leans_toward_red_only_as_far_as_the_others_may() {
    // The game's own is #2a0f04, further from the ramp's hue than 15 degrees.
    assert_eq!(hex(named("amber").dark), "#261300");
}

#[test]
fn night_turns_the_ramp_to_plain_greys() {
    for (token, [r, g, b]) in tokens(named("night")).into_iter().skip(1) {
        assert!(r == g && g == b, "Night's {token} is {}", hex([r, g, b]));
    }
}

#[test]
fn a_colour_past_the_last_is_night() {
    for colour in [BACKDROP_COLOURS.len() as u8, 17, 200, u8::MAX] {
        assert_eq!(palette(colour), named("night"), "{colour}");
    }
}

#[test]
fn every_sprite_key_is_its_colour_and_nothing_else_is_one() {
    // Plum's steps all differ, so a key that stood for the wrong one would show.
    let p = named("plum");
    let keys = [
        ('k', p.dark),
        ('e', p.edge),
        ('w', p.well),
        ('p', p.plate),
        ('t', p.tab),
        ('b', p.band),
        ('s', p.stripe),
        ('d', p.dim),
        ('A', p.frame),
        ('H', p.frame_hi),
        ('L', p.label),
        ('c', p.cream),
    ];
    let steps: HashSet<_> = keys.iter().map(|(_, colour)| colour).collect();
    assert_eq!(steps.len(), keys.len(), "plum repeats a step");
    for (k, colour) in keys {
        assert_eq!(p.key(k), Some(colour), "{k}");
    }
    for k in ['.', ' ', 'D', 'K', 'E', 'a', 'h', 'l', 'C', 'x'] {
        assert_eq!(p.key(k), None, "{k:?}");
    }
}

#[test]
fn rgba_is_the_colour_out_of_one_with_its_alpha() {
    assert_eq!(rgba([255, 0, 51], 0.5), [1.0, 0.0, 0.2, 0.5]);
}
