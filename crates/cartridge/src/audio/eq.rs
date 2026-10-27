//! The equalizer: five bands over everything the speaker plays, a low shelf, three peaks and a
//! high shelf, as RBJ's cookbook draws them. The handhelds' speakers are too small for real
//! bass, so the lowest band sits at 150 Hz, where they still answer. Nothing is turned down to
//! make room for a band raised: what clips is held at the top.

use cartridge_store::EQ_BANDS;

/// The bands' middles, in Hz, low to high.
pub const EQ_HZ: [f32; EQ_BANDS] = [150.0, 400.0, 1000.0, 2500.0, 6000.0];

/// How wide each peak is: an octave and a bit, so neighbouring bands meet without a dip.
const Q: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    LowShelf,
    Peak,
    HighShelf,
}

/// One band's filter: its coefficients, normalised so a0 is 1, and each channel's last two
/// inputs and outputs.
#[derive(Clone, Copy, Debug, Default)]
struct Biquad {
    b: [f32; 3],
    a: [f32; 2],
    x: [[f32; 2]; 2],
    y: [[f32; 2]; 2],
}

impl Biquad {
    fn new(kind: Kind, hz: f32, db: f32, rate: f32) -> Biquad {
        let a = 10f32.powf(db / 40.0);
        let w = std::f32::consts::TAU * hz / rate;
        let (sin, cos) = w.sin_cos();
        let (b, den) = match kind {
            Kind::Peak => {
                let alpha = sin / (2.0 * Q);
                (
                    [1.0 + alpha * a, -2.0 * cos, 1.0 - alpha * a],
                    [1.0 + alpha / a, -2.0 * cos, 1.0 - alpha / a],
                )
            }
            Kind::LowShelf | Kind::HighShelf => {
                // a shelf's slope of 1, as steep as it goes without a bump at its knee
                let alpha = sin / 2.0 * 2f32.sqrt();
                let root = 2.0 * a.sqrt() * alpha;
                let s = if kind == Kind::LowShelf { 1.0 } else { -1.0 };
                (
                    [
                        a * ((a + 1.0) - s * (a - 1.0) * cos + root),
                        s * 2.0 * a * ((a - 1.0) - s * (a + 1.0) * cos),
                        a * ((a + 1.0) - s * (a - 1.0) * cos - root),
                    ],
                    [
                        (a + 1.0) + s * (a - 1.0) * cos + root,
                        -s * 2.0 * ((a - 1.0) + s * (a + 1.0) * cos),
                        (a + 1.0) + s * (a - 1.0) * cos - root,
                    ],
                )
            }
        };
        Biquad {
            b: [b[0] / den[0], b[1] / den[0], b[2] / den[0]],
            a: [den[1] / den[0], den[2] / den[0]],
            ..Biquad::default()
        }
    }

    fn run(&mut self, ch: usize, x: f32) -> f32 {
        let [x1, x2] = self.x[ch];
        let [y1, y2] = self.y[ch];
        let y = self.b[0] * x + self.b[1] * x1 + self.b[2] * x2 - self.a[0] * y1 - self.a[1] * y2;
        self.x[ch] = [x, x1];
        self.y[ch] = [y, y1];
        y
    }
}

/// The five bands at some gains, for interleaved stereo at one rate.
#[derive(Clone, Debug, Default)]
pub struct Equalizer {
    gains: [i8; EQ_BANDS],
    rate: u32,
    bands: Vec<Biquad>,
}

impl Equalizer {
    /// The bands at `gains` dB for sound at `rate`; the filters keep what they hold when only a
    /// gain changes, so a band moved while something plays does not click.
    pub fn set(&mut self, gains: [i8; EQ_BANDS], rate: u32) {
        if gains == self.gains && rate == self.rate {
            return;
        }
        let kept = (rate == self.rate).then(|| std::mem::take(&mut self.bands));
        self.gains = gains;
        self.rate = rate;
        self.bands.clear();
        if self.flat() || rate == 0 {
            return;
        }
        for (i, (&hz, &db)) in EQ_HZ.iter().zip(&gains).enumerate() {
            let kind = match i {
                0 => Kind::LowShelf,
                i if i == EQ_BANDS - 1 => Kind::HighShelf,
                _ => Kind::Peak,
            };
            let mut band = Biquad::new(kind, hz, f32::from(db), rate as f32);
            if let Some(old) = kept.as_ref().and_then(|k| k.get(i)) {
                band.x = old.x;
                band.y = old.y;
            }
            self.bands.push(band);
        }
    }

    /// Every band at 0 dB: the sound passes as it is.
    pub fn flat(&self) -> bool {
        self.gains.iter().all(|&g| g == 0)
    }

    /// `samples`, interleaved stereo, through the bands in place.
    pub fn process(&mut self, samples: &mut [i16]) {
        if self.bands.is_empty() {
            return;
        }
        for (i, s) in samples.iter_mut().enumerate() {
            let ch = i & 1;
            let mut v = f32::from(*s);
            for band in &mut self.bands {
                v = band.run(ch, v);
            }
            *s = v.clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        }
    }
}
