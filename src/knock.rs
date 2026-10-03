//! The sound a domino makes when it hits something. Spec 0001.
//!
//! Noise through a few heavily damped resonances, which is wood landing on a
//! table. Written out sample by sample, and nothing here needs an audio device
//! so all of it can be checked without one.
//!
//! The same shape as cairn's, which took four tries to arrive at, with the
//! resonances carried up because a domino is a tenth of the weight of one of
//! those blocks and sounds like it.

use rodio::buffer::SamplesBuffer;
use rodio::{ChannelCount, SampleRate};

const RATE: u32 = 48_000;

/// How long one lasts. A clack, not a ring.
pub const SECONDS: f32 = 0.055;

/// How fast the noise that drives it dies away. Most of it is in the first few
/// milliseconds, which is the moment of contact.
const BURST_FADE: f32 = 230.0;

/// The resonances the noise is heard through, in hertz, with how tight each is
/// and how loud.
///
/// Low for the floor taking the weight, middle for the body of the domino,
/// high for the edge of it. Higher than cairn's, because a domino is small and
/// light and sounds like it. Tight enough to colour the noise and loose enough
/// not to ring. A resonance held any closer to one is a note, and a note is
/// what the three attempts before this turned into.
const RINGS: [f32; 3] = [340.0, 1100.0, 2400.0];
const TIGHTNESS: [f32; 3] = [0.90, 0.85, 0.80];
const LOUDNESS: [f32; 3] = [1.0, 0.75, 0.40];

/// How much of the raw noise is left in, unresonated. The scrape of one surface
/// on another, which is most of what stops it sounding synthetic.
const SCRAPE: f32 = 0.30;

/// What the whole thing is scaled by so a loud one reaches full and no louder.
/// Measured from the loudest unscaled one across all forty eight blocks, which
/// came out at 8.70, with a little left over. Left alone every knock clipped,
/// and a clipped knock is a different noise altogether.
const GAIN: f32 = 0.105;

/// How far ahead of itself the sound may get, in seconds.
///
/// The engine plays queued sounds one after another rather than over each
/// other, so a collapse that hands it eighty knocks is eighty knocks long. Six
/// seconds of clattering after everything has stopped. Nothing is queued while
/// there is already more than this waiting, which bounds how far behind the
/// sound can fall to about a tenth of a second.
pub const LEAD: f32 = 0.11;

/// Whether there is room to play another, given how much is already waiting.
pub fn room_for_another(waiting: f32) -> bool {
    waiting < LEAD
}

/// How far from the ears a knock is placed, in world units.
///
/// The engine's spatial sound fades with one over the distance squared, so at
/// one it is heard whole. Where the block really is would be eighteen units
/// away and three thousandths of itself.
pub const EARSHOT: f32 = 1.0;

/// The quietest a knock is allowed to be, against one for the hardest landing
/// there is.
///
/// A clack is fifty milliseconds of noise, and fifty milliseconds of noise at a
/// third of full is a thing you can miss entirely. Something that was worth
/// making a sound about is worth hearing.
pub const SOFTEST: f32 = 0.45;

/// How loud a landing of this force is heard.
pub fn loudness(force: f32) -> f32 {
    SOFTEST + force.clamp(0.0, 1.0) * (1.0 - SOFTEST)
}

/// How far a block's resonances may sit from the middle, as a multiplier. Not a
/// scale: blocks are not tuned, and forty eight of them at exactly the same
/// frequencies read as one block hit forty eight times.
pub const SPREAD: f32 = 0.16;

/// Where this block's resonances sit. Its own number, so the same block sounds
/// the same every time and its neighbours do not.
pub fn colour_of(block: usize) -> f32 {
    // the golden ratio, which spreads a sequence of small integers about as
    // evenly over a range as anything does
    let along = (block as f32 * 0.618_034).fract();

    1.0 + (along - 0.5) * 2.0 * SPREAD
}

/// The noise of the contact. A hash rather than a random number, so a knock is
/// the same knock every time it is asked for.
///
/// Seeded, which it was not. Without a seed the burst is the same few hundred
/// samples in every knock, and a hundred of those down a run is one click
/// played a hundred times at slightly different pitches. Wood never makes the
/// same noise twice.
fn grit(n: u32, seed: u32) -> f32 {
    let mut x = n
        .wrapping_add(seed.wrapping_mul(2_654_435_761))
        .wrapping_mul(1_664_525)
        .wrapping_add(1_013_904_223);
    x ^= x >> 15;
    x = x.wrapping_mul(2_246_822_519);

    ((x >> 8) & 0xffff) as f32 / 32767.5 - 1.0
}

/// Which grit a knock is made of, from the domino and how hard it landed.
///
/// Both, so the same domino hit twice in a run is not the same noise twice, and
/// so a knock is still the same knock every time it is asked for.
pub fn seed_of(block: usize, volume: f32) -> u32 {
    (block as u32).wrapping_mul(2_246_822_519) ^ (volume * 4096.0) as u32
}

/// The samples of one knock.
pub fn samples(volume: f32, colour: f32, seed: u32) -> Vec<f32> {
    shaped(volume, colour, seed, TIGHTNESS, SCRAPE)
}

/// The same thing with the shape handed in, so it can be measured.
fn shaped(volume: f32, colour: f32, seed: u32, tightness: [f32; 3], scrape: f32) -> Vec<f32> {
    let volume = volume.clamp(0.0, 1.0);
    let count = (RATE as f32 * SECONDS) as usize;

    // a two pole resonator per ring: y = x + a1 y' + a2 y''
    let mut was = [[0.0f32; 2]; RINGS.len()];
    let steps: Vec<(f32, f32)> = (0..RINGS.len())
        .map(|ring| {
            let hz = (RINGS[ring] * colour).min(RATE as f32 * 0.45);
            let turn = std::f32::consts::TAU * hz / RATE as f32;
            let tight = tightness[ring];

            (2.0 * tight * turn.cos(), -tight * tight)
        })
        .collect();

    let mut out = Vec::with_capacity(count);
    for n in 0..count {
        let t = n as f32 / RATE as f32;
        let burst = grit(n as u32, seed) * (-t * BURST_FADE).exp();

        let mut sample = burst * scrape;
        for ring in 0..RINGS.len() {
            let (a1, a2) = steps[ring];
            let now = burst + a1 * was[ring][0] + a2 * was[ring][1];
            was[ring][1] = was[ring][0];
            was[ring][0] = now;

            // a resonator's gain goes up as it is held tighter, so what comes
            // out is scaled back by how tight it is
            sample += now * LOUDNESS[ring] * (1.0 - tightness[ring]) * 4.0;
        }

        out.push((sample * volume * GAIN).clamp(-1.0, 1.0));
    }

    out
}

/// The sound itself, ready to be played.
pub fn knock(volume: f32, colour: f32, seed: u32) -> SamplesBuffer {
    SamplesBuffer::new(
        ChannelCount::new(1).expect("one channel"),
        SampleRate::new(RATE).expect("the sample rate is not zero"),
        samples(volume, colour, seed),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loudest(of: &[f32]) -> f32 {
        of.iter().fold(0.0f32, |a, b| a.max(b.abs()))
    }

    /// How much the sound repeats itself, at whatever interval it repeats best.
    /// One is a pure tone and nothing is noise.
    ///
    /// Each lag is normalised by the energy of the two stretches being
    /// compared, which matters more than it sounds. Without it, any two
    /// exponentially decaying things correlate well at every lag, because the
    /// envelope does it for them. The first version of this measure gave a
    /// clack 0.85.
    fn periodic(of: &[f32]) -> f32 {
        // from 80 hz up to 3 khz, which is every pitch anything here could have
        (RATE as usize / 3000..RATE as usize / 80)
            .filter(|lag| *lag < of.len() / 2)
            .map(|lag| {
                let (mut matched, mut here, mut there) = (0.0f32, 0.0f32, 0.0f32);
                for n in 0..of.len() - lag {
                    matched += of[n] * of[n + lag];
                    here += of[n] * of[n];
                    there += of[n + lag] * of[n + lag];
                }

                if here * there <= 0.0 {
                    return 0.0;
                }

                matched / (here * there).sqrt()
            })
            .fold(0.0f32, f32::max)
    }

    /// Spec 0004: the sound cannot fall behind what is happening. Played one
    /// after another, a collapse's worth of knocks outlasts the collapse.
    #[test]
    fn the_sound_cannot_fall_behind() {
        let mut waiting = 0.0f32;
        let mut played = 0;

        // two seconds of a tower coming down, offering a knock every frame
        for _ in 0..240 {
            waiting = (waiting - 1.0 / 120.0).max(0.0);
            if room_for_another(waiting) {
                waiting += SECONDS;
                played += 1;
            }

            assert!(
                waiting <= LEAD + SECONDS,
                "the sound is {}s behind",
                waiting
            );
        }

        // and it is not so cautious that a collapse goes quiet
        assert!(
            played > 15,
            "only {} knocks were played in two seconds",
            played
        );
        assert!(played < 120, "{} knocks is one a frame", played);
    }

    /// Spec 0004: the softest knock there is, is still a knock.
    #[test]
    fn the_quietest_landing_can_still_be_heard() {
        assert_eq!(loudness(0.0), SOFTEST);
        assert_eq!(loudness(1.0), 1.0);
        assert!(loudness(0.3) > loudness(0.1), "it does not rise with force");

        let quietest = loudest(&samples(loudness(0.0), 1.0, 0));
        assert!(quietest > 0.3, "the quietest one peaks at {}", quietest);
    }

    #[test]
    fn a_block_keeps_its_colour() {
        assert_eq!(colour_of(11), colour_of(11));
        assert_ne!(colour_of(0), colour_of(1), "neighbours sound the same");

        for block in 0..48 {
            let colour = colour_of(block);
            assert!((1.0 - SPREAD..=1.0 + SPREAD).contains(&colour));
        }
    }

    /// The one this is for. A block landing on a table has no pitch, and the
    /// three attempts before this one all did.
    ///
    /// Measured this way a pure sine comes out at 1.00 and plain noise at
    /// 0.05. This sits at about 0.52, and most of what is left is the low
    /// resonance being a lowpass. The measure answers to smoothness as well as
    /// pitch, since neighbouring samples of anything dull are alike. It
    /// catches a note and it does not pretend to be a spectrum.
    #[test]
    fn it_is_not_a_note() {
        let clack = periodic(&samples(1.0, 1.0, 0));

        let tone: Vec<f32> = (0..(RATE as f32 * SECONDS) as usize)
            .map(|n| (n as f32 / RATE as f32 * 400.0 * std::f32::consts::TAU).sin())
            .collect();
        let hiss: Vec<f32> = (0..(RATE as f32 * SECONDS) as usize)
            .map(|n| grit(n as u32, 0))
            .collect();

        assert!(
            periodic(&tone) > 0.9 && periodic(&hiss) < 0.2,
            "the measure itself is wrong: a sine came out at {} and noise at {}",
            periodic(&tone),
            periodic(&hiss)
        );
        assert!(
            clack < 0.65,
            "it repeats itself {} of the way, which is a note",
            clack
        );
    }

    #[test]
    fn it_is_over_quickly() {
        let samples = samples(1.0, 1.0, 0);
        assert_eq!(samples.len(), (RATE as f32 * SECONDS) as usize);

        let tenth = samples.len() / 10;
        assert!(
            loudest(&samples[samples.len() - tenth..]) < loudest(&samples[..tenth]) * 0.25,
            "it is still going at the end"
        );
    }

    /// Most of it is at the front. A block landing is a moment, not a swell.
    #[test]
    fn it_is_loudest_at_the_moment_of_contact() {
        let samples = samples(1.0, 1.0, 0);
        let energy = |of: &[f32]| of.iter().map(|s| s * s).sum::<f32>();

        let fifth = samples.len() / 5;
        assert!(
            energy(&samples[..fifth]) > energy(&samples[fifth..]) * 1.5,
            "the sound arrives after the impact"
        );
    }

    #[test]
    fn a_quiet_knock_is_quieter_and_nothing_clips() {
        assert!(loudest(&samples(0.2, 1.0, 0)) < loudest(&samples(1.0, 1.0, 0)) * 0.5);
        assert_eq!(loudest(&samples(0.0, 1.0, 0)), 0.0, "silence is silent");

        for block in 0..48 {
            assert!(
                loudest(&samples(1.0, colour_of(block), seed_of(block, 1.0))) <= 1.0,
                "it clipped"
            );
        }
    }

    /// And it is the same knock every time, which a random burst would not be.
    #[test]
    fn the_same_knock_twice_is_the_same_knock() {
        let seed = seed_of(9, 0.7);
        assert_eq!(samples(0.7, 1.05, seed), samples(0.7, 1.05, seed));
    }

    /// Spec 0001: and two knocks are not the same noise.
    ///
    /// Without this the burst was the same few hundred samples in every one of
    /// them, so a run was one click played over and over at slightly different
    /// pitches. Jake called it robotic.
    #[test]
    fn two_knocks_are_different_noises() {
        let one = samples(0.7, 1.0, seed_of(3, 0.7));
        let other = samples(0.7, 1.0, seed_of(4, 0.7));

        assert_eq!(one.len(), other.len());
        let apart: f32 = one
            .iter()
            .zip(&other)
            .map(|(a, b)| (a - b).abs())
            .sum::<f32>()
            / one.len() as f32;
        let loud: f32 = one.iter().map(|a| a.abs()).sum::<f32>() / one.len() as f32;
        assert!(
            apart > loud * 0.5,
            "two knocks differ by {} against a loudness of {}",
            apart,
            loud
        );

        // and the same domino landing harder is not the same noise either
        assert_ne!(seed_of(3, 0.7), seed_of(3, 0.9));
    }
}
