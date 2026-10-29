use cartridge_gfx::{out_w, Draw, MeshId, TexId, FOCAL};
use cartridge_store::{Cart, Platform};

use crate::cart::{cart_box, label_colour, label_text, model_box, model_of, CART_H};
use crate::chrome::BAR_H;
use crate::hud::Millis;
use crate::models::disc::{self, DiscStack};
use crate::models::{hucard, md, nes, snes};
use crate::motion::ease;
use crate::silhouette::GbShell;
use crate::spin::{self, CartMeshes, CartModel, Pose};
use crate::stage::{stage_y, MOCKUP_H};
use crate::status_bar::TOP_BAR_H;

const SIDE_SCALE: f32 = 0.78;
/// How much bigger than its own size the cart the shelf rests on stands.
pub const ACTIVE_SCALE: f32 = 1.12;
const SIDE_ALPHA: f32 = 0.55;
/// How big a Game Boy pak stands on the shelf, as a share of its own size: at its own size it
/// would stand too tall beside a GBA cart. It keeps this size on its way into the handheld.
pub const GB_SHELF_SCALE: f32 = 0.8;

/// How big a cart for `platform` stands on the shelf, as a share of its own size.
pub fn shelf_scale(platform: Platform) -> f32 {
    match platform {
        Platform::Gba => 1.0,
        Platform::Md => md::SHELF_SCALE,
        Platform::Gg | Platform::Sms => crate::models::sega8::SHELF_SCALE,
        Platform::Nds => crate::models::nds::SHELF_SCALE,
        Platform::Music => crate::models::tape::SHELF_SCALE,
        Platform::Nes => nes::SHELF_SCALE,
        Platform::Pce => hucard::SHELF_SCALE,
        Platform::Gb | Platform::Gbc => GB_SHELF_SCALE,
        Platform::Snes => snes::SHELF_SCALE,
        Platform::Ps1 => disc::SHELF_SCALE,
    }
}

/// How big `model` stands on the shelf, as a share of its own size.
pub fn model_scale(model: CartModel) -> f32 {
    match model {
        CartModel::Gba => 1.0,
        CartModel::Gb(_) => GB_SHELF_SCALE,
        CartModel::Snes(_) => snes::SHELF_SCALE,
        CartModel::Disc => disc::SHELF_SCALE,
        CartModel::HuCard => hucard::SHELF_SCALE,
        CartModel::Nds => crate::models::nds::SHELF_SCALE,
        CartModel::Tape => crate::models::tape::SHELF_SCALE,
        CartModel::Nes(_) => nes::SHELF_SCALE,
        CartModel::Md(_) => md::SHELF_SCALE,
        CartModel::Sega8(_) => crate::models::sega8::SHELF_SCALE,
    }
}

/// The size `model` stands on the shelf at, and spaces its neighbours by.
pub fn model_shelf_box(model: CartModel) -> (f32, f32) {
    let (w, h) = model_box(model);
    let scale = model_scale(model);
    (w as f32 * scale, h as f32 * scale)
}

/// How far apart `model`'s carts stand on the shelf: their width, but for a HuCard's, which
/// stands narrower than its place.
pub fn model_pitch(model: CartModel) -> f32 {
    match model {
        CartModel::HuCard => hucard::PITCH,
        CartModel::Nds => crate::models::nds::PITCH,
        _ => model_shelf_box(model).0,
    }
}

/// The size a cart for `platform` stands on the shelf at, and spaces its neighbours by.
pub fn shelf_box(platform: Platform) -> (f32, f32) {
    let (w, h) = cart_box(platform);
    let scale = shelf_scale(platform);
    (w as f32 * scale, h as f32 * scale)
}

/// Where a cart `h` tall stands on the shelf, between the bar across the top and the bar across
/// the bottom: a pak taller than a GBA cart is centred there, and anything
/// shorter stands on the floor a GBA cart centred there would. A pak shrunk on the shelf is
/// still taller than a GBA cart, so it is centred where it would stand at its own size.
pub fn rest_y(h: f32) -> f32 {
    let (top, floor) = (TOP_BAR_H, MOCKUP_H - BAR_H);
    let y = if h > CART_H as f32 {
        top + (floor - top - h) / 2.0
    } else {
        (top + floor + CART_H as f32) / 2.0 - h
    };
    y + stage_y()
}

pub fn foot_y(h: f32) -> f32 {
    rest_y(h) + h
}

/// How far above where it stands a cart `w` by `h` can reach as it sways: it floats up, and
/// the side that turns towards the eye grows.
pub fn sway_reach(w: f32, h: f32) -> f32 {
    let grown = FOCAL / (FOCAL - w / 2.0 * SWAY.sin());
    BOB + h / 2.0 * (grown - 1.0)
}
const OMEGA: f32 = 16.0;
const PART: f32 = 130.0;

const SLOTS: i32 = 3;

/// The selected cart sways this far either way about its upright axis, in radians, once the
/// shelf stands still...
const SWAY: f32 = 20.0 * std::f32::consts::PI / 180.0;
const SWAY_S: f32 = 3.6;
/// ...and floats this many canvas pixels up and down, out of step with the sway so the two
/// never fall into one mechanical beat.
const BOB: f32 = 4.0;
const BOB_S: f32 = 2.4;
/// Both periods divide this, so the clock that drives them wraps here without a jump.
const LOOP_S: f32 = 7.2;
/// How fast the sway comes in once the shelf stops, and goes once it moves, per second.
const CALM_IN: f32 = 1.6;
const CALM_OUT: f32 = 8.0;
/// Carts lean into a scroll this many radians per cart a second, up to LEAN_MAX, and stand
/// straight once the scroll is slower than LEAN_FROM carts a second.
const LEAN: f32 = 0.045;
const LEAN_MAX: f32 = 0.35;
const LEAN_FROM: f32 = 0.05;
/// A turn or a lift smaller than these cannot be seen, so the cart is drawn square and level.
const STILL_YAW: f32 = 0.002;
const STILL_LIFT: f32 = 0.05;

/// A cart left alone this long starts its tricks, and then one comes every so often.
const FIRST_MOVE_S: f32 = 3.0;
const MOVE_GAP_S: (f32, f32) = (3.5, 7.0);
/// How high a hop lifts the cart, in canvas pixels.
const HOP: f32 = 9.0;
/// A cart landing on its place bounces this high, and the bounce dies away this fast a second.
const LAND: f32 = 5.0;
const LAND_DECAY: f32 = 5.0;
/// How far a scroll may overshoot its stop: the shelf is a spring a little under critical.
const DAMPING: f32 = 1.0;
/// The carts beside the middle one float this far, each out of step with the next.
const NEIGHBOUR_BOB: f32 = 1.4;

/// One of the things a cart does when it is left standing: how long it takes, in seconds.
#[derive(Copy, Clone, PartialEq, Debug)]
enum Trick {
    /// A full turn about its upright axis, showing its back.
    Twirl,
    /// Two hops, landing with a little roll.
    Hop,
    /// A quick shake.
    Wiggle,
    /// Turns to look aside and comes back.
    Peek,
    /// One turn in the canvas, lifted a little.
    Cartwheel,
}

impl Trick {
    const ALL: [Trick; 5] = [
        Trick::Twirl,
        Trick::Hop,
        Trick::Wiggle,
        Trick::Peek,
        Trick::Cartwheel,
    ];

    fn seconds(self) -> f32 {
        match self {
            Trick::Twirl => 1.7,
            Trick::Hop => 1.1,
            Trick::Wiggle => 0.9,
            Trick::Peek => 2.2,
            Trick::Cartwheel => 1.5,
        }
    }

    /// The pose this trick adds `t` of the way through (0 to 1), turning towards `dir` (+1/-1),
    /// and how much of the resting sway it leaves: none in the middle of it.
    fn at(self, t: f32, dir: f32) -> (Pose, f32) {
        let tau = std::f32::consts::TAU;
        let bell = (std::f32::consts::PI * t).sin();
        let fade = 1.0 - t;
        let pose = match self {
            Trick::Twirl => Pose {
                yaw: dir * tau * ease(t),
                lift: 4.0 * bell,
                roll: 0.0,
            },
            Trick::Hop => Pose {
                yaw: 0.0,
                lift: HOP * (tau * t).sin().abs() * (1.0 - 0.4 * t),
                roll: 0.06 * dir * (2.0 * tau * t).sin() * fade,
            },
            Trick::Wiggle => Pose {
                yaw: 0.2 * (3.0 * tau * t + 1.0).sin() * fade,
                lift: 0.0,
                roll: 0.11 * dir * (3.0 * tau * t).sin() * fade,
            },
            Trick::Peek => Pose {
                yaw: dir * 0.95 * bell * bell,
                lift: 3.0 * bell * bell,
                roll: -0.07 * dir * bell * bell,
            },
            Trick::Cartwheel => Pose {
                yaw: 0.0,
                lift: 5.0 * bell,
                roll: dir * tau * ease(t),
            },
        };
        (pose, 1.0 - bell)
    }
}

const REPEAT_DELAY_MS: Millis = 400;
const REPEAT_MS: [Millis; 4] = [110, 85, 65, 50];

pub struct Shelf {
    pub carts: Vec<Cart>,
    pub index: usize,
    pub scroll: f32,
    faces: Vec<TexId>,
    shadows: [Option<TexId>; CartModel::ALL.len()],
    models: Vec<CartModel>,
    /// How many discs each cart stands as: one, but for a game on several discs.
    discs: Vec<usize>,
    meshes: Option<CartMeshes>,
    ride: f32,
    vel: f32,
    held: Option<(i32, Millis, usize)>,
    calm: f32,
    age: f32,
    /// How far the discs have turned about their own middles, in radians.
    spin: f32,
    still: bool,
    /// How long the middle cart has stood still, and when its next trick starts.
    idle: f32,
    next_trick: f32,
    trick: Option<(Trick, f32, f32)>,
    /// Seconds since the cart came to rest on its place, for the bounce.
    landed: f32,
    was_moving: bool,
    dice: u32,
    playful: bool,
}

impl Shelf {
    pub fn new(carts: Vec<Cart>) -> Self {
        Shelf {
            models: carts.iter().map(model_of).collect(),
            discs: carts.iter().map(disc::stack_of).collect(),
            carts,
            index: 0,
            scroll: 0.0,
            faces: Vec::new(),
            shadows: [None; CartModel::ALL.len()],
            meshes: None,
            ride: 0.0,
            vel: 0.0,
            held: None,
            calm: 0.0,
            age: 0.0,
            spin: 0.0,
            still: false,
            idle: 0.0,
            next_trick: FIRST_MOVE_S,
            trick: None,
            landed: 10.0,
            was_moving: false,
            dice: 0x9E37_79B9,
            playful: false,
        }
    }

    /// Lets the carts do more than sway: tricks while one stands alone, a bounce on landing,
    /// the others floating. A shelf made plain keeps to the sway and the lean.
    pub fn set_playful(&mut self, on: bool) {
        self.playful = on;
    }

    pub fn select(&mut self, i: usize) {
        self.index = i;
        self.scroll = i as f32;
        self.ride = i as f32;
        self.vel = 0.0;
    }

    pub fn set_shadow(&mut self, face: TexId) {
        self.set_model_shadow(CartModel::Gba, face);
    }

    /// The black outline that backs a `model` cart not drawn as a model.
    pub fn set_model_shadow(&mut self, model: CartModel, face: TexId) {
        self.shadows[model.index()] = Some(face);
    }

    /// The models carts are drawn with from now on, sides and all.
    pub fn set_meshes(&mut self, meshes: CartMeshes) {
        self.meshes = Some(meshes);
    }

    pub fn set_gb_shadow(&mut self, shell: GbShell, face: TexId) {
        self.set_model_shadow(CartModel::Gb(shell), face);
    }

    pub fn set_faces(&mut self, faces: Vec<TexId>) {
        self.faces = faces;
    }

    /// Its faces, given up for the shelves to be laid out anew.
    pub fn take_faces(&mut self) -> Vec<TexId> {
        std::mem::take(&mut self.faces)
    }

    /// The model the cart `stem` is drawn with, once models are set: a game on several discs
    /// all its discs in one model.
    pub fn mesh_for(&self, stem: &str) -> Option<MeshId> {
        let i = self.carts.iter().position(|c| c.stem == stem)?;
        let meshes = self.meshes?;
        let model = *self.models.get(i)?;
        match (model, meshes.stacks()) {
            (CartModel::Disc, Some(stacks)) => Some(stacks.whole(self.discs_at(i))),
            _ => Some(meshes.of(model)),
        }
    }

    /// The discs the game `stem` stands as, once its face and the models are set; `None` for
    /// a cart.
    pub fn stack_for(&self, stem: &str) -> Option<DiscStack> {
        let i = self.carts.iter().position(|c| c.stem == stem)?;
        self.stack_at(i)
    }

    /// The discs the game at `i` stands as, once its face and the models are set.
    pub fn stack_at(&self, i: usize) -> Option<DiscStack> {
        if self.model_at(i)? != CartModel::Disc {
            return None;
        }
        Some(DiscStack {
            tex: *self.faces.get(i)?,
            discs: self.discs_at(i),
            meshes: self.meshes?.stacks()?,
        })
    }

    /// How many discs the cart at `i` stands as.
    fn discs_at(&self, i: usize) -> usize {
        match self.discs.get(i) {
            Some(n) => *n,
            // a cart pushed onto the row after it was made has not been counted
            None => self.carts.get(i).map_or(1, disc::stack_of),
        }
    }

    /// The model the cart at `i` stands as.
    pub fn model_at(&self, i: usize) -> Option<CartModel> {
        self.models.get(i).copied()
    }

    pub fn find(&self, stem: &str) -> Option<(&Cart, Option<TexId>)> {
        let i = self.carts.iter().position(|c| c.stem == stem)?;
        Some((&self.carts[i], self.faces.get(i).copied()))
    }

    /// The cart `stem` of `platform`, and its face: a shelf may hold several systems' carts.
    pub fn find_of(&self, platform: Platform, stem: &str) -> Option<(&Cart, Option<TexId>)> {
        let i = self
            .carts
            .iter()
            .position(|c| c.platform == platform && c.stem == stem)?;
        Some((&self.carts[i], self.faces.get(i).copied()))
    }

    pub fn left(&mut self) {
        self.step(-1);
    }

    pub fn right(&mut self) {
        self.step(1);
    }

    pub fn jump_next_letter(&mut self) {
        self.jump(1);
    }

    pub fn jump_prev_letter(&mut self) {
        self.jump(-1);
    }

    fn start_of_letter(&self, from: usize) -> usize {
        let n = self.carts.len();
        let letter = cartridge_store::initial(&self.carts[from].stem);
        let mut at = from;
        for _ in 0..n {
            let before = (at as i32 - 1).rem_euclid(n as i32) as usize;
            if cartridge_store::initial(&self.carts[before].stem) != letter {
                break;
            }
            at = before;
        }
        at
    }

    fn jump(&mut self, dir: i32) {
        let n = self.carts.len();
        if n < 2 {
            return;
        }
        let wrap = |i: i32| i.rem_euclid(n as i32) as usize;
        let here = cartridge_store::initial(&self.carts[self.index].stem);
        if self
            .carts
            .iter()
            .all(|c| cartridge_store::initial(&c.stem) == here)
        {
            return;
        }
        let target = match dir > 0 {
            true => {
                let mut at = self.index;
                for _ in 0..n {
                    at = wrap(at as i32 + 1);
                    if cartridge_store::initial(&self.carts[at].stem) != here {
                        break;
                    }
                }
                at
            }
            false => {
                let start = self.start_of_letter(self.index);
                match start == self.index {
                    true => self.start_of_letter(wrap(start as i32 - 1)),
                    false => start,
                }
            }
        };
        let ahead = (target as i32 - self.index as i32).rem_euclid(n as i32);
        let delta = match dir > 0 {
            true => ahead,
            false => ahead - n as i32,
        };
        self.index = target;
        self.ride += delta as f32;
    }

    pub fn hold_left(&mut self, now: Millis) {
        self.hold(-1, now);
    }

    pub fn hold_right(&mut self, now: Millis) {
        self.hold(1, now);
    }

    fn hold(&mut self, by: i32, now: Millis) {
        self.step(by);
        self.held = Some((by, now + REPEAT_DELAY_MS, 0));
    }

    pub fn release_left(&mut self) {
        self.release(-1);
    }

    pub fn release_right(&mut self) {
        self.release(1);
    }

    fn release(&mut self, by: i32) {
        if matches!(self.held, Some((held, _, _)) if held == by) {
            self.held = None;
        }
    }

    pub fn release_hold(&mut self) {
        self.held = None;
    }

    pub fn tick(&mut self, now: Millis) {
        let Some((by, due, fired)) = self.held else {
            return;
        };
        if now < due {
            return;
        }
        self.step(by);
        let rate = REPEAT_MS[fired.min(REPEAT_MS.len() - 1)];
        self.held = Some((by, now + rate, fired + 1));
    }

    fn step(&mut self, by: i32) {
        let n = self.carts.len();
        if n < 2 {
            return;
        }
        self.index = (self.index as i32 + by).rem_euclid(n as i32) as usize;
        self.ride += by as f32;
    }

    pub fn scroll_target(&self) -> f32 {
        let n = self.carts.len();
        if n == 0 {
            return 0.0;
        }
        let from = self.ride;
        let n = n as f32;
        from + (self.index as f32 - from + n / 2.0).rem_euclid(n) - n / 2.0
    }

    pub fn cart_at_offset(&self, off: i32) -> Option<usize> {
        let n = self.carts.len() as i32;
        if n == 0 {
            return None;
        }
        if n == 1 {
            return (off == 0).then_some(self.index);
        }
        Some((self.index as i32 + off).rem_euclid(n) as usize)
    }

    /// Where the selected cart's left edge is on the row right now, and how big it is drawn
    /// there as a share of its own size.
    pub fn selected_at(&self) -> (f32, f32) {
        let model = self.model_at(self.index).unwrap_or(CartModel::Gba);
        let w = model_shelf_box(model).0;
        let pitch = model_pitch(model);
        let offset = self.scroll_target() - self.scroll;
        let scale = shrink(offset);
        (
            out_w() as f32 / 2.0 + offset * pitch - w * scale / 2.0,
            scale * model_scale(model),
        )
    }

    /// How the selected cart is held right now, wherever it is in the row.
    pub fn pose(&self) -> Pose {
        let pose = self.pose_at(self.scroll_target() - self.scroll, self.index);
        // a cart caught mid-twirl is the same cart a turn on
        let tau = std::f32::consts::TAU;
        Pose {
            yaw: match pose.yaw.abs() > tau / 2.0 {
                true => (pose.yaw + tau / 2.0).rem_euclid(tau) - tau / 2.0,
                false => pose.yaw,
            },
            ..pose
        }
    }

    /// A number from `lo` to `hi`, from a small shuffle of its own: the tricks do not repeat
    /// in any order the eye could learn.
    fn roll_dice(&mut self, lo: f32, hi: f32) -> f32 {
        self.dice ^= self.dice << 13;
        self.dice ^= self.dice >> 17;
        self.dice ^= self.dice << 5;
        lo + (hi - lo) * (self.dice >> 8) as f32 / (1u32 << 24) as f32
    }

    /// How a cart `offset` places from the middle of the row is held: the one in the middle
    /// sways and floats while the shelf stands still, and every cart leans into a scroll.
    fn pose_at(&self, offset: f32, cart: usize) -> Pose {
        let alone = ease(self.calm);
        let calm = alone * (1.0 - offset.abs().min(1.0));
        let lean = match self.vel.abs() < LEAN_FROM {
            true => 0.0,
            false => (-self.vel * LEAN).clamp(-LEAN_MAX, LEAN_MAX),
        };
        let wave = |period: f32| (self.age * std::f32::consts::TAU / period).sin();
        let seen = |v: f32, least: f32| if v.abs() < least { 0.0 } else { v };
        let (trick, sway) = match self.trick {
            Some((trick, t, dir)) if offset.abs() < 0.5 => {
                let (pose, sway) = trick.at(t, dir);
                (pose.toward_rest(1.0 - calm), sway)
            }
            _ => (Pose::REST, 1.0),
        };
        // the landing: a bounce that dies away, for the cart that has just come to its place
        let settle = (-LAND_DECAY * self.landed).exp() * (1.0 - offset.abs().min(1.0));
        let land_lift = LAND * settle * (11.0 * self.landed).sin().abs();
        let land_roll = 0.035 * settle * (14.0 * self.landed).sin();
        // the others float too, each out of step: never two moving as one
        let phase = cart as f32 * 2.399;
        let drift = match self.playful {
            true => alone * (1.0 - calm) * NEIGHBOUR_BOB,
            false => 0.0,
        };
        let side = drift * (self.age * std::f32::consts::TAU / BOB_S + phase).sin();
        let side_yaw = drift * 0.02 * (self.age * std::f32::consts::TAU / SWAY_S + phase).sin();
        Pose {
            yaw: seen(
                SWAY * calm * sway * wave(SWAY_S) + lean + trick.yaw + side_yaw,
                STILL_YAW,
            ),
            lift: seen(
                BOB * calm * sway * wave(BOB_S) + trick.lift + land_lift + side,
                STILL_LIFT,
            ),
            roll: seen(trick.roll + land_roll, STILL_YAW),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.calm_down(dt);
        let accel = -2.0 * DAMPING * OMEGA * self.vel
            - OMEGA * OMEGA * (self.scroll - self.scroll_target());
        self.vel += accel * dt;
        self.scroll += self.vel * dt;
    }

    /// Keeps the selected cart from swaying while something else has it, such as the core
    /// picker: it settles square and level, and starts again once let go.
    pub fn hold_still(&mut self, still: bool) {
        self.still = still;
    }

    fn calm_down(&mut self, dt: f32) {
        let moving = (self.scroll - self.scroll_target()).abs() > 0.01 || self.vel.abs() > 0.2;
        if self.playful && self.was_moving && !moving {
            self.landed = 0.0;
        }
        self.was_moving = moving;
        self.landed = (self.landed + dt).min(10.0);
        if self.playful {
            self.tricks(dt, moving || self.still);
        }
        if moving || self.still {
            self.calm = (self.calm - CALM_OUT * dt).max(0.0);
        } else {
            if self.calm == 0.0 {
                self.age = 0.0;
            }
            self.calm = (self.calm + CALM_IN * dt).min(1.0);
        }
        self.age = (self.age + dt).rem_euclid(LOOP_S);
        self.spin = (self.spin + disc::SPIN * std::f32::consts::TAU * ease(self.calm) * dt)
            .rem_euclid(std::f32::consts::TAU);
    }

    /// Runs the middle cart's tricks: once it has stood alone a while it starts one, and goes
    /// on to another every few seconds. A scroll or anything holding it still ends the trick.
    fn tricks(&mut self, dt: f32, disturbed: bool) {
        if disturbed {
            self.idle = 0.0;
            self.trick = None;
            self.next_trick = FIRST_MOVE_S;
            return;
        }
        self.idle += dt;
        if let Some((trick, t, dir)) = self.trick {
            let t = t + dt / trick.seconds();
            self.trick = (t < 1.0).then_some((trick, t, dir));
            if t >= 1.0 {
                self.next_trick = self.idle + self.roll_dice(MOVE_GAP_S.0, MOVE_GAP_S.1);
            }
        } else if self.idle >= self.next_trick && self.calm >= 1.0 {
            let k = self.roll_dice(0.0, Trick::ALL.len() as f32) as usize;
            let dir = if self.roll_dice(0.0, 1.0) < 0.5 {
                -1.0
            } else {
                1.0
            };
            self.trick = Some((Trick::ALL[k.min(Trick::ALL.len() - 1)], 0.0, dir));
        }
    }

    /// The row, none of its carts held.
    pub fn draw(&self, shake: f32, out: &mut Vec<Draw>) {
        self.draw_row(None, shake, 0.0, 1.0, out);
    }

    pub fn draw_row(
        &self,
        hidden: Option<&str>,
        shake: f32,
        recede: f32,
        dim: f32,
        out: &mut Vec<Draw>,
    ) {
        let recede = recede.clamp(0.0, 1.0);
        let dim = dim.clamp(0.0, 1.0);
        let target = self.scroll_target();
        for slot in -SLOTS..=SLOTS {
            let Some(i) = self.cart_at_offset(slot) else {
                continue;
            };
            let cart = &self.carts[i];
            if hidden == Some(cart.stem.as_str()) {
                continue;
            }
            let offset = target + slot as f32 - self.scroll;
            let t = offset.abs().min(1.0);
            let scale = shrink(offset);
            let alpha = (1.0 + (SIDE_ALPHA - 1.0) * t) * (1.0 - recede);
            // a cart pushed onto the row after it was made has no model looked up yet
            let model = self
                .models
                .get(i)
                .copied()
                .unwrap_or_else(|| model_of(cart));
            let (cw, ch) = model_shelf_box(model);
            let (w, h) = (cw * scale, ch * scale);
            let away = offset.signum() * (1.0 + offset.abs());
            let x =
                out_w() as f32 / 2.0 + offset * model_pitch(model) - w / 2.0 + away * PART * recede;
            if x + w <= 0.0 || x >= out_w() as f32 || alpha <= 0.0 {
                continue;
            }
            let x = x + shake;
            let y = foot_y(ch) - h;
            let face = self.faces.get(i).copied();
            // A cart still waiting for its face stands still: only a face can turn.
            let pose = match face {
                Some(_) => self.pose_at(offset, i).toward_rest(recede),
                None => Pose::REST,
            };
            let y = y - pose.lift;
            if let (Some(tex), Some(meshes)) = (face, self.meshes) {
                // a side cart is the face dimmed over its black backing, as one opaque model
                let colour = match alpha < 1.0 {
                    true => {
                        let k = alpha * dim;
                        [k, k, k, recede_alpha(alpha)]
                    }
                    false => [1.0, 1.0, 1.0, dim],
                };
                match self.stack_at(i) {
                    // a game on several discs is each of them, one behind the other
                    Some(stack) => disc::push_stack(
                        out,
                        (x, y, w, h),
                        &stack,
                        colour,
                        self.spin + pose.roll,
                        pose.yaw,
                    ),
                    None => out.push(spin::posed_cart(
                        (x, y, w, h),
                        meshes.of(model),
                        tex,
                        colour,
                        pose.roll,
                        pose.yaw,
                    )),
                }
                continue;
            }
            if let Some(tex) = face {
                spin::push_cart_side(
                    out,
                    (x, y, w, h),
                    tex,
                    pose.roll,
                    pose.yaw,
                    alpha * dim,
                    recede_alpha(alpha),
                );
            }
            if alpha < 1.0 {
                if let Some(tex) = self.shadows[model.index()] {
                    out.push(spin::posed_face(
                        (x, y, w, h),
                        tex,
                        recede_alpha(alpha),
                        pose.roll,
                        pose.yaw,
                    ));
                }
            }
            out.push(match face {
                Some(tex) => spin::posed_face((x, y, w, h), tex, alpha * dim, pose.roll, pose.yaw),
                None => {
                    let c = label_colour(&label_text(cart));
                    Draw::Rect {
                        x,
                        y,
                        w,
                        h,
                        colour: [
                            c[0] as f32 / 255.0,
                            c[1] as f32 / 255.0,
                            c[2] as f32 / 255.0,
                            alpha * dim,
                        ],
                    }
                }
            });
        }
    }
}

fn shrink(offset: f32) -> f32 {
    ACTIVE_SCALE + (SIDE_SCALE - ACTIVE_SCALE) * offset.abs().min(1.0)
}

fn recede_alpha(face_alpha: f32) -> f32 {
    (face_alpha / SIDE_ALPHA).clamp(0.0, 1.0)
}
