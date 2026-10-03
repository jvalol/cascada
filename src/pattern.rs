//! The yin and yang that is standing when a run begins. Spec 0002.
//!
//! Points on a floor, in the order they are laid. Nothing here knows about
//! dominoes beyond how far apart they want to be, so all of it can be checked
//! without a physics step.
//!
//! The figure is a tree, which is the whole of its design. One push, on the
//! ring. The S branches off the ring at the top, and a spur branches off each
//! half of the S and winds in to that half's dot. Everything is reached from
//! the one push and no path ever rejoins another, so the wave only ever spreads
//! and never meets itself coming back.

use crate::domino::TALL;
use glam::{vec3, Vec3};
use std::f32::consts::PI;

/// How big the whole thing is: the radius of the outer ring.
///
/// Measured rather than chosen. A bend turns `step / radius` between two
/// dominoes, and the whole figure's bends scale with this, so a small figure is
/// a tight one. At 3.6 across the S turns 21 degrees between neighbours and
/// stalls; at 5.2 it turns 14 and goes over whole.
pub const ROUND: f32 = 5.2;

/// How far apart they stand, as a fraction of their own height. Spec 0001
/// measured the range that carries at six to nine tenths, all of them only in
/// the lower half of it.
pub const APART: f32 = 0.65;

/// How far to either side of its own line a domino going over reaches.
/// Measured, a run of twenty splitting into two of fourteen, each branch set
/// this far off the line of the last domino of the stem: ```text 0.26 to 0.40
/// both branches go over whole, at every radius tried 0.46 one branch goes,
/// the other is missed 0.55 neither ``` Which is the thing this spec first had
/// wrong. It had measured that a domino knocks over nothing more than fifteen
/// degrees off straight ahead, which is true of a single domino set to one
/// side, and read it as a wave being unable to split. A pair standing abreast
/// is a different question and the answer is different.
pub const REACH: f32 = 0.43;

/// How far aside a path stands where it branches off another, and how far
/// along.
///
/// A branching domino has to be nearer the stem than the stem's own next one,
/// or that one shields it. Measured, a straight run carrying on and a second
/// path leaving it:
///
/// ```text
/// half a step ahead, 0.25 to 0.45 aside    both paths go over whole
/// a whole step ahead, same aside           the branch never starts
/// a step and a quarter ahead               it goes again, taken by the
///                                          stem's next instead of the stem
/// ```
///
/// So a branch is not a road forking. It is one domino standing closer in, and
/// the path it belongs to leaving from there. Nothing about the path it leaves
/// has to change, which is why the ring is a plain circle and the S is two
/// plain half circles.
pub const ASIDE: f32 = REACH * 0.8;
pub const ALONG: f32 = 0.5;

/// How far apart two paths stand where neither is meant to touch the other.
/// Outside a domino's reach, with something to spare.
pub const CLEAR: f32 = 0.7;

/// How far the S reaches at each end. The top end stands a branch inside the
/// ring, which is how the wave gets off the ring and into the S. The bottom
/// end stops clear of it. Were it a branch at both ends the figure would hold
/// a loop, and a loop fills from both sides at once, leaving an arch of three
/// standing where they meet.
pub const TOP: f32 = ROUND - ASIDE;
pub const BOTTOM: f32 = ROUND - CLEAR;

/// The radius of each half of the S, which is half of what that half spans.
pub const HIGH: f32 = TOP * 0.5;
pub const LOW: f32 = BOTTOM * 0.5;

/// How big the two dots are. How big the two dots are. Small, which took
/// measuring. The tightest ring a wave gets round at all: ```text 0.8, 0.9
/// nothing goes over 1.0 7 dominoes, 51 degrees apiece, all 7 1.2 9 dominoes,
/// 40 degrees apiece, all 9 1.7 14 dominoes, 26 degrees apiece, all 14 ```
/// This spec had carried 36 degrees as the point where a ring jams. That was
/// measured before bends had their spacing corrected for the pinch at the
/// inside, and is no longer true of anything. Believing it had the dots at a
/// third of the outer ring, nothing like a yin and yang draws them, and left
/// no room for the spur that feeds them. A dot of 1.2 clears the S by more
/// than a domino's reach and is reached across that gap in a single step.
pub const DOT: f32 = 1.2;

/// The step along a straight path, in units of the world rather than of a
/// domino.
pub fn step() -> f32 {
    TALL * APART
}

/// The step along a bend of this radius, which is further than the straight
/// one. What decides whether a domino reaches the next is the gap at the
/// inside of the bend, which is less than the gap between their middles. A
/// domino is half a width across, so a bend turning `t` radians between two of
/// them closes the inner gap by about `w·t/2`. Stepping by the same amount
/// everywhere therefore pinches every curve, and pinches a tight one hardest.
/// Solving `inner = step·(1 − w / 2r)` for the step that leaves the inner gap
/// where it was wanted. On the outer ring it is a few percent; on half an S it
/// is a tenth; on a dot it is more again.
pub fn step_round(radius: f32) -> f32 {
    let wide = crate::domino::HALF.z * 2.0;
    // inner_gap read the other way about: it takes a step and says what is left
    // at the inside, and this takes what is wanted there and says the step
    let pinch = (inner_gap(1.0, 1.0 / radius, wide)).max(0.35);

    step() / pinch
}

/// How many dominoes go round a bend of this radius and angle.
fn count_for(radius: f32, through: f32) -> usize {
    (radius * through.abs() / step_round(radius))
        .round()
        .max(1.0) as usize
}

/// The whole figure: the paths in the order they are laid, and which domino of
/// the first of them is pushed.
pub struct Figure {
    pub parts: Vec<Vec<Vec3>>,
    pub pushed: usize,
}

/// Where the S branches off the ring, and where each spur branches off its half
/// of the S, as a fraction of the way along that half.
const SPURS_AT: f32 = 0.5;

/// How far round the spur turns as it winds in from the S to its dot.
pub const WINDS: f32 = PI * 0.5;

pub fn figure() -> Figure {
    figure_with(WINDS)
}

pub fn figure_with(winds: f32) -> Figure {
    // The ring: a plain circle, counted to a multiple of four and turned back
    // by half a step, so that the domino a quarter of the way round stands half
    // a step short of the top. The S's first one stands at the top, which puts
    // it half a step ahead of that one and a branch inside it.
    let round = count_for(ROUND, PI * 2.0).div_ceil(4) * 4;
    let apiece = PI * 2.0 / round as f32;
    let ring = turning(Vec3::ZERO, ROUND, ROUND, -apiece * ALONG, PI * 2.0, round);

    // The S, from the ring downwards, which is the way the wave travels it.
    let up = count_for(HIGH, PI);
    let down = count_for(LOW, PI);
    let mut ess = turning(vec3(0.0, 0.0, HIGH), HIGH, HIGH, PI * 0.5, PI, up);
    ess.extend(turning(vec3(0.0, 0.0, -LOW), LOW, LOW, PI * 0.5, -PI, down));
    ess.push(vec3(0.0, 0.0, -BOTTOM));

    // and a spur off the middle of each half of it, half a step further on than
    // the S's own next domino is, and a branch inside it
    let parts = vec![
        ring,
        ess,
        dot(vec3(0.0, 0.0, HIGH), HIGH, PI * 0.5, PI, up, 1.0, winds),
        dot(vec3(0.0, 0.0, -LOW), LOW, PI * 0.5, -PI, down, -1.0, winds),
    ];

    // the push goes at the right of the ring, so the wave climbs a quarter of
    // the way round before the S branches away and the two run on together
    Figure { parts, pushed: 0 }
}

pub fn all_of_it() -> Vec<Vec<Vec3>> {
    figure().parts
}

pub fn pushed_at() -> usize {
    figure().pushed
}

/// A dot, with the one domino that reaches it. Not a winding spur, which is
/// what this first tried. A spur that comes in turning alongside the ring it
/// feeds is a spiral, and the ring then goes the whole way round and back
/// underneath it. At the join their gap is nothing, by definition, so some
/// stretch of the ring always stands inside the spur. No size of figure fixes
/// that, because the gap closes to nothing at the join whatever the scale.
/// With a dot small enough to sit a clear gap inside the S, none of that is
/// needed. One domino stands off the S, a branch inside it and half a step
/// along, turned to face straight in rather than to follow anything. It takes
/// the blow on its wide face, falls inwards, and lands on the dot's ring. The
/// ring then goes round once and ends on its own fallen start.
#[allow(clippy::too_many_arguments)]
fn dot(
    middle: Vec3,
    arc: f32,
    from: f32,
    through: f32,
    count: usize,
    way: f32,
    winds: f32,
) -> Vec<Vec3> {
    let apiece = through / count as f32;
    let leaves = from + apiece * ((count as f32 * SPURS_AT).round() + ALONG);
    let out = arc - ASIDE;
    let winding = winds * way;

    // one more rather than one fewer: a spur is diving inwards as well as
    // turning, and the step that is merely near enough on a plain bend is the
    // one that leaves it stranded after a domino
    let along = winds * ((out + DOT) * 0.5).hypot((DOT - out) / winds);
    let mut path = turning(
        middle,
        out,
        DOT,
        leaves,
        winding,
        (along / step_round(DOT)).ceil().max(1.0) as usize,
    );
    let round = PI * 2.0 * way - winding;
    path.extend(turning(
        middle,
        DOT,
        DOT,
        leaves + winding,
        round,
        count_for(DOT, round),
    ));

    path
}

/// Walks a turn whose radius may be closing, which is what a spur does and
/// which an arc is the flat case of.
///
/// The last point is left off: on a ring it is the first one again, and on a
/// spur it is where the ring it feeds begins.
fn turning(
    middle: Vec3,
    from_radius: f32,
    to_radius: f32,
    from: f32,
    through: f32,
    count: usize,
) -> Vec<Vec3> {
    (0..count)
        .map(|n| {
            let how_far = n as f32 / count as f32;
            let radius = from_radius + (to_radius - from_radius) * how_far;
            let angle = from + through * how_far;

            middle + vec3(angle.cos() * radius, 0.0, angle.sin() * radius)
        })
        .collect()
}

/// How far two dominoes standing on a bend are from each other on the inside of
/// it, which is less than their middles are by about half a width a radian.
pub fn inner_gap(apart: f32, turned: f32, wide: f32) -> f32 {
    apart - wide * 0.5 * turned.abs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domino::HALF;

    fn flat(a: Vec3, b: Vec3) -> f32 {
        vec3(a.x - b.x, 0.0, a.z - b.z).length()
    }

    fn ring() -> Vec<Vec3> {
        all_of_it()[0].clone()
    }

    fn ess() -> Vec<Vec3> {
        all_of_it()[1].clone()
    }

    fn dots() -> Vec<Vec<Vec3>> {
        all_of_it()[2..].to_vec()
    }

    #[test]
    fn it_is_a_loop_an_ess_and_two_dots() {
        let parts = all_of_it();

        assert_eq!(parts.len(), 4);
        assert!(ring().len() > 25, "the ring is {} long", ring().len());
        assert!(ess().len() > 12, "the S is {} long", ess().len());
        for dot in dots() {
            assert!(dot.len() >= 8, "a dot of {} is not a ring", dot.len());
        }
    }

    /// Every step is one a domino carries over, wherever on the figure it is.
    /// Measured from the path itself rather than from what drew it: the turn
    /// between one step and the next, and the gap that leaves at the inside of
    /// the bend. Spec 0001 measured what carries on the straight at six to
    /// nine tenths of a domino's height, and a bend eats into that.
    #[test]
    fn every_step_is_one_that_carries() {
        let wide = HALF.z * 2.0;

        for part in all_of_it() {
            for three in part.windows(3) {
                let (one, two) = (three[1] - three[0], three[2] - three[1]);
                let turned = one.normalize().dot(two.normalize()).clamp(-1.0, 1.0).acos();
                let inner = inner_gap(two.length(), turned, wide);

                assert!(
                    (0.4..=0.9).contains(&inner),
                    "a gap of {} at the inside of a turn of {} degrees, at {}",
                    inner,
                    turned.to_degrees(),
                    three[1]
                );
            }
        }
    }

    /// The S leaves the ring as a branch: its first domino stands nearer the
    /// stem than the stem's own next one does, and inside a domino's reach of
    /// the line the stem is on.
    #[test]
    fn the_ess_branches_off_the_ring() {
        let ring = ring();
        let ess = ess();

        assert!(
            branches(&ring, ring.len() / 4, ess[0]),
            "the S's first domino is not placed where the ring will knock it over"
        );
    }

    /// And each spur leaves the S the same way.
    #[test]
    fn each_spur_branches_off_the_ess() {
        let ess = ess();

        for dot in dots() {
            assert!(
                (1..ess.len() - 1).any(|stem| branches(&ess, stem, dot[0])),
                "no domino of the S is placed to knock a spur over"
            );
        }
    }

    /// What a branch has to be, measured: the branching domino nearer the stem
    /// than the stem's own next one, and inside a reach of its line. Further
    /// off and nothing takes it; further along and the stem's own next one
    /// stands in front of it.
    fn branches(parent: &[Vec3], stem: usize, off: Vec3) -> bool {
        let way = (parent[stem] - parent[stem - 1]).normalize();
        let across = vec3(-way.z, 0.0, way.x);

        let near = flat(parent[stem], off);
        let on = flat(parent[stem], parent[stem + 1]);

        near < on
            && near > 0.4
            && (off - parent[stem]).dot(across).abs() < REACH
            && (off - parent[stem]).dot(way) > 0.0
    }

    /// And the bottom of the S stops clear of the ring, so the figure holds no
    /// loop. A loop fills from both sides at once and leaves an arch standing
    /// where the two sides meet.
    #[test]
    fn the_ess_does_not_rejoin_the_ring() {
        let ring = ring();
        let ess = ess();
        let end = ess[ess.len() - 1];

        let nearest = ring
            .iter()
            .map(|on| flat(end, *on))
            .fold(f32::INFINITY, f32::min);
        assert!(
            nearest > REACH,
            "the S comes back within {} of the ring",
            nearest
        );

        // and it still reaches most of the way, so the figure reads right
        assert!(
            flat(end, Vec3::ZERO) > ROUND * 0.82,
            "the S only reaches {} of {}",
            flat(end, Vec3::ZERO),
            ROUND
        );
    }

    /// Nothing stands near anything except where it branches off it, so every
    /// junction in the figure is one that was drawn on purpose. Which is why a
    /// dot's ring stops where its spur came in rather than closing. A ring
    /// that goes the whole way round passes back underneath the spur that
    /// feeds it, and at the join their gap is nothing. The spur lies across
    /// the opening, so it still reads closed.
    #[test]
    fn nothing_stands_near_anything_it_does_not_branch_from() {
        let parts = all_of_it();
        // which path each one leaves: the S leaves the ring, a dot leaves the S
        let parent = [None, Some(0usize), Some(1), Some(1)];
        // and how many of its dominoes are still alongside it while it does
        const LEAVING: usize = 3;

        for (n, part) in parts.iter().enumerate() {
            for (m, other) in parts.iter().enumerate().skip(n + 1) {
                for (k, at) in part.iter().enumerate() {
                    for (j, on) in other.iter().enumerate() {
                        if parent[m] == Some(n) && j < LEAVING {
                            continue;
                        }
                        if parent[n] == Some(m) && k < LEAVING {
                            continue;
                        }

                        assert!(
                            flat(*at, *on) > CLEAR,
                            "two paths stand {} apart at {} and {}",
                            flat(*at, *on),
                            at,
                            on
                        );
                    }
                }
            }
        }
    }

    /// The step round a bend and the gap it leaves at the inside of it are the
    /// same rule read each way, so stepping by one gives back the other.
    #[test]
    fn the_step_round_a_bend_leaves_the_gap_it_meant_to() {
        let wide = HALF.z * 2.0;

        for radius in [ROUND, HIGH, DOT, 1.2, 0.9] {
            let stepped = step_round(radius);
            let left = inner_gap(stepped, stepped / radius, wide);

            assert!(
                (left - step()).abs() < 1e-3,
                "a bend of {} stepped at {} left {} where {} was wanted",
                radius,
                stepped,
                left,
                step()
            );
        }

        // a tighter bend is stepped further apart, which is the whole point
        assert!(step_round(DOT) > step_round(ROUND));
        assert!(step_round(ROUND) > step());
    }
}
