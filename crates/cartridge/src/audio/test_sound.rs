//! The sound the EQUALIZER screen plays to hear the bands on: `System/eq-test.wav` on the card,
//! 16-bit stereo at the output's 48 kHz, or, with none there, a few bars made here that reach
//! into every band: a kick and a bass line, chords, a lead and hats.

use std::path::Path;

use super::OUTPUT_HZ;

/// Where a card keeps its own test sound.
pub const TEST_SOUND: &str = "System/eq-test.wav";

/// The test sound, interleaved stereo at `OUTPUT_HZ`.
pub fn test_sound(root: &Path) -> Vec<i16> {
    std::fs::read(root.join(TEST_SOUND))
        .ok()
        .and_then(|bytes| wav(&bytes))
        .unwrap_or_else(made)
}

/// A wav file's samples, when it is 16-bit stereo PCM at `OUTPUT_HZ`.
fn wav(bytes: &[u8]) -> Option<Vec<i16>> {
    let u16_at = |i: usize| {
        bytes
            .get(i..i + 2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
    };
    let u32_at = |i: usize| {
        bytes
            .get(i..i + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };
    if bytes.get(0..4)? != b"RIFF" || bytes.get(8..12)? != b"WAVE" {
        return None;
    }
    let mut at = 12;
    let mut format_ok = false;
    while at + 8 <= bytes.len() {
        let id = &bytes[at..at + 4];
        let len = u32_at(at + 4)? as usize;
        let body = at + 8;
        if id == b"fmt " {
            format_ok = u16_at(body)? == 1
                && u16_at(body + 2)? == 2
                && u32_at(body + 4)? == OUTPUT_HZ
                && u16_at(body + 14)? == 16;
        } else if id == b"data" && format_ok {
            let data = bytes.get(body..(body + len).min(bytes.len()))?;
            let samples: Vec<i16> = data
                .chunks_exact(2)
                .map(|b| i16::from_le_bytes([b[0], b[1]]))
                .collect();
            return (!samples.is_empty()).then_some(samples);
        }
        at = body + len + (len & 1);
    }
    None
}

/// Two bars at 120 bpm, made from simple waves.
fn made() -> Vec<i16> {
    let rate = OUTPUT_HZ as f32;
    let beat = rate / 2.0;
    let frames = (beat * 8.0) as usize;
    let tau = std::f32::consts::TAU;
    let bass = [55.0, 55.0, 65.41, 49.0];
    let chords = [[220.0, 261.63, 329.63], [196.0, 246.94, 293.66]];
    let lead = [
        659.25, 587.33, 523.25, 587.33, 659.25, 783.99, 659.25, 523.25,
    ];
    let mut noise = 0x1234_5678u32;
    let mut out = Vec::with_capacity(frames * 2);
    for i in 0..frames {
        let t = i as f32 / rate;
        let b = i as f32 / beat;
        let in_beat = b.fract() * 0.5;
        let kick = (tau * (50.0 + 90.0 * (-in_beat * 30.0).exp()) * in_beat).sin()
            * (-in_beat * 9.0).exp();
        let bass_hz = bass[(b / 2.0) as usize % bass.len()];
        let bass = (tau * bass_hz * t).sin().signum() * 0.18 * (1.0 - b.fract() * 0.6);
        let chord = chords[(b / 4.0) as usize % chords.len()]
            .iter()
            .map(|hz| (tau * hz * t).sin() * 0.07)
            .sum::<f32>();
        let lead_hz = lead[b as usize % lead.len()];
        let saw = 2.0 * (lead_hz * t).fract() - 1.0;
        let lead = saw * 0.08 * (-b.fract() * 3.0).exp();
        noise ^= noise << 13;
        noise ^= noise >> 17;
        noise ^= noise << 5;
        let off_beat = (b + 0.5).fract() * 0.5;
        let hat = (noise as f32 / u32::MAX as f32 - 0.5) * 0.25 * (-off_beat * 60.0).exp();
        let mid = kick * 0.5 + bass + chord;
        let left = (mid + lead * 0.8 + hat * 0.6) * 0.8;
        let right = (mid + lead * 0.6 + hat) * 0.8;
        out.push((left.clamp(-1.0, 1.0) * 30_000.0) as i16);
        out.push((right.clamp(-1.0, 1.0) * 30_000.0) as i16);
    }
    out
}
