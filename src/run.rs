//! A run: the dominoes laid, the push, and what fell. Spec 0001.

use crate::domino::{self, HALF};
use blitzkit::collision::{obbs_meet, Aabb, Obb};
use blitzkit::physics::{Body, Shape, Solver};
use glam::{vec3, Vec3};

/// How many there are to lay, which the pattern spends most of.
/// How many there are to lay. The figure spends most of them and the rest are
/// the player's.
pub const SUPPLY: usize = 130;

/// How hard the first one is pushed, and where on it.
///
/// Near the top, which is where a finger goes, and only hard enough to put it
/// past its own balance: the rest is gravity.
///
/// Measured, a run of twenty at six tenths of their height:
///
/// ```text
/// 0.10, 0.20   nothing moves
/// 0.35 to 2.0  all twenty go over
/// 5.00         one goes over
/// ```
///
/// The top end is the thing spec 0001 wondered about and it is real: shoved
/// hard enough the first one leaves over the top of the second rather than
/// into it, and nothing else moves. Half is in the middle of what works.
pub const PUSH: f32 = 0.5;
pub const PUSHED_AT: f32 = 0.8;

/// How long everything has to be still before a run is over, and how still.
///
/// Its own measure rather than the engine's sleeping, which is a property of a
/// whole group of touching bodies: one of them twitching resets the timer for
/// all of them, and a heap of fifty four fallen dominoes took between sixty and
/// eighty seconds to go quiet enough all at once. Cairn's tower of forty does it
/// in a second and a half, so this is about the heap rather than the number.
pub const RESTS_FOR: f32 = 0.8;
pub const RESTS_UNDER: f32 = 0.1;

pub const GRAVITY: Vec3 = vec3(0.0, -9.81, 0.0);
pub const GROUND: f32 = 30.0;

/// How much speed a domino has to lose in a step to count as having hit
/// something, and the loss that counts as flat out.
pub const MIN_KNOCK: f32 = 0.4;
pub const LOUDEST_KNOCK: f32 = 3.0;
/// How many are heard out of any one step.
pub const AT_ONCE: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phase {
    /// Dominoes may be laid.
    Laying,
    /// One has been pushed and it is out of the player's hands.
    Falling,
    /// Everything has come to rest.
    Over,
}

/// Something hitting something, for the window to make a noise about.
#[derive(Debug, Clone, Copy)]
pub struct Knock {
    pub which: usize,
    pub at: Vec3,
    pub force: f32,
}

pub struct Run {
    dominoes: Vec<Body>,
    ground: Vec<Aabb>,
    solver: Solver,
    phase: Phase,
    knocks: Vec<Knock>,
    /// How long nothing has moved, so a run can be called over.
    still: f32,
    /// Which domino starts each separate piece, and which one it faces, so a
    /// push can set every piece off.
    starts: Vec<(usize, usize)>,
}

impl Default for Run {
    fn default() -> Self {
        Self::new()
    }
}

impl Run {
    pub fn new() -> Self {
        let mut run = Self::bare();
        for (n, part) in crate::pattern::all_of_it().iter().enumerate() {
            let began = run.lay_path(part);

            // one push, on the ring, and everything else is reached from it
            if n == 0 {
                let at = began + crate::pattern::pushed_at();
                run.starts.push((at, at + 1));
            }
        }

        run
    }

    /// A floor with nothing on it, which only the pattern and the tests want.
    pub fn bare() -> Self {
        Self {
            dominoes: Vec::new(),
            ground: vec![Aabb::from_center_size(
                vec3(0.0, -1.0, 0.0),
                vec3(GROUND, 2.0, GROUND),
            )],
            solver: Solver::new(),
            phase: Phase::Laying,
            knocks: Vec::new(),
            still: 0.0,
            starts: Vec::new(),
        }
    }

    /// Stands a whole path up at once, each facing the next along it. Says
    /// where in the run it began.
    ///
    /// Not by calling `lay` over and over: that turns each one to face the last
    /// thing laid, which is right within a path and wrong at the seam between
    /// two of them. The last of a path faces the way the path was going.
    pub fn lay_path(&mut self, path: &[Vec3]) -> usize {
        let began = self.dominoes.len();

        for (n, at) in path.iter().enumerate() {
            if self.left() == 0 {
                return began;
            }

            let way = if n + 1 < path.len() {
                path[n + 1] - *at
            } else if path.len() > 1 {
                *at - path[n - 1]
            } else {
                Vec3::X
            };

            let laid = domino::standing(*at, way);
            if self.dominoes.iter().any(|other| overlap(&laid, other)) {
                continue;
            }

            self.dominoes.push(laid);
        }

        self.solver.forget();
        began
    }

    pub fn dominoes(&self) -> &[Body] {
        &self.dominoes
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// How many are left to lay.
    pub fn left(&self) -> usize {
        SUPPLY - self.dominoes.len()
    }

    /// How many have gone over.
    pub fn fallen(&self) -> usize {
        self.dominoes
            .iter()
            .filter(|one| domino::has_fallen(one))
            .count()
    }

    /// How far a domino laid here would stand from the last one, in units of
    /// their own height. Nothing in the game enforces a spacing and this does
    /// not either: it says what the gap is, and what works is still a thing to
    /// find out by trying it.
    pub fn gap_to(&self, at: Vec3) -> Option<f32> {
        let last = self.dominoes.last()?.position;

        Some(vec3(at.x - last.x, 0.0, at.z - last.z).length() / domino::TALL)
    }

    /// How many are actually moving, which is the wave made visible.
    ///
    /// Not the awake count, which is what this first showed. Awake is the
    /// engine's word for a body it has not yet put aside, and a heap of fallen
    /// dominoes stays awake for a minute after the last of them has stopped. A
    /// number that climbs with the wave and never comes back down is not the
    /// wave.
    pub fn moving(&self) -> usize {
        self.dominoes
            .iter()
            .filter(|one| one.velocity.length() > RESTS_UNDER)
            .count()
    }

    /// What has hit something since this was last asked. Taken rather than
    /// read, so nothing is heard twice.
    pub fn knocks(&mut self) -> Vec<Knock> {
        std::mem::take(&mut self.knocks)
    }

    /// Stands one up at a point on the floor, across the line from the one
    /// before it. Says whether it went down.
    ///
    /// The first has nothing to face, so it is stood square and turned to face
    /// the second when that arrives: a run of one has no direction yet.
    pub fn lay(&mut self, at: Vec3) -> bool {
        if self.phase != Phase::Laying || self.left() == 0 {
            return false;
        }

        let way = match self.dominoes.last() {
            Some(last) => at - last.position,
            None => Vec3::X,
        };

        let laid = domino::standing(at, way);
        if self.dominoes.iter().any(|other| overlap(&laid, other)) {
            return false;
        }

        if self.dominoes.len() == 1 {
            let first = self.dominoes[0].position;
            self.dominoes[0] = domino::standing(first, at - first);
        }

        self.dominoes.push(laid);
        if self.dominoes.len() == 2 && self.starts.is_empty() {
            self.starts.push((0, 1));
        }
        self.solver.forget();
        true
    }

    /// Sets the figure off, which is the last thing the player does.
    ///
    /// One domino. The figure is drawn so that everything else is reached from
    /// it: the S branches off the ring and each dot's spur branches off the S.
    pub fn push(&mut self) {
        if self.phase != Phase::Laying || self.dominoes.len() < 2 {
            return;
        }

        for (first, second) in std::mem::take(&mut self.starts) {
            let from = self.dominoes[first];
            let way = (self.dominoes[second].position - from.position).normalize_or_zero();
            let at = from.position + Vec3::Y * HALF.y * PUSHED_AT - way * HALF.x;

            self.dominoes[first].strike(way * PUSH, at);
        }

        self.phase = Phase::Falling;
        self.still = 0.0;
    }

    pub fn step(&mut self, dt: f32) {
        if self.dominoes.is_empty() {
            return;
        }

        let was: Vec<f32> = self
            .dominoes
            .iter()
            .map(|one| one.velocity.length())
            .collect();

        self.solver
            .step(&mut self.dominoes, &self.ground, GRAVITY, dt);

        self.listen(&was);

        if self.phase == Phase::Falling {
            if self.moving() == 0 {
                self.still += dt;
                if self.still > RESTS_FOR {
                    self.phase = Phase::Over;
                }
            } else {
                self.still = 0.0;
            }
        }
    }

    /// Writes down what hit something. An impact is a loss of speed, and only
    /// the loudest few of a step are kept: a run going over is dozens of these
    /// at once and all of them together is one bang.
    fn listen(&mut self, was: &[f32]) {
        let mut heard: Vec<Knock> = self
            .dominoes
            .iter()
            .enumerate()
            .filter_map(|(which, one)| {
                let lost = was[which] - one.velocity.length();
                if lost < MIN_KNOCK {
                    return None;
                }

                Some(Knock {
                    which,
                    at: one.position,
                    force: ((lost - MIN_KNOCK) / (LOUDEST_KNOCK - MIN_KNOCK)).clamp(0.0, 1.0),
                })
            })
            .collect();

        heard.sort_by(|one, other| other.force.total_cmp(&one.force));
        heard.truncate(AT_ONCE);
        self.knocks.extend(heard);
    }
}

/// Whether two dominoes standing in these places would be inside each other.
///
/// Asked of the engine's own shape test rather than worked out here, so what
/// the game refuses and what the physics would have done agree by construction.
fn overlap(one: &Body, other: &Body) -> bool {
    let boxy = |body: &Body| match body.shape {
        Shape::Block { half } => Some(Obb::new(body.position, body.orientation, half)),
        Shape::Sphere { .. } => None,
    };

    match (boxy(one), boxy(other)) {
        (Some(one), Some(other)) => obbs_meet(&one, &other).is_some(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A straight run at a spacing, laid and settled.
    fn run_of(count: usize, apart: f32) -> Run {
        let mut run = Run::bare();
        for n in 0..count {
            assert!(
                run.lay(vec3(n as f32 * apart, 0.0, 0.0)),
                "it would not lay number {} at {} apart",
                n,
                apart
            );
        }

        for _ in 0..240 {
            run.step(1.0 / 120.0);
        }

        run
    }

    fn until_settled(run: &mut Run, most: usize) -> usize {
        let mut ticks = 0;
        while run.phase() != Phase::Over && ticks < most {
            run.step(1.0 / 120.0);
            ticks += 1;
        }

        ticks
    }

    /// Spec 0001: the gap is reported, in units of a domino's height, and it
    /// is the plain distance rather than any judgement about it.
    #[test]
    fn the_gap_is_said_in_dominoes() {
        let mut run = Run::bare();
        assert_eq!(
            run.gap_to(Vec3::ZERO),
            None,
            "there is nothing to measure from"
        );

        run.lay(Vec3::ZERO);
        let gap = run
            .gap_to(vec3(domino::TALL * 0.7, 0.0, 0.0))
            .expect("one is laid");
        assert!((gap - 0.7).abs() < 1e-4, "it said {}", gap);

        // and across, not only along
        let gap = run
            .gap_to(vec3(0.0, 0.0, domino::TALL * 1.5))
            .expect("one is laid");
        assert!((gap - 1.5).abs() < 1e-4, "it said {}", gap);
    }

    /// Spec 0002: a new run is the pattern, standing, with some of the supply
    /// left over.
    /// Spec 0002: the figure comes to rest and the run ends, rather than the
    /// minute the engine's own sleeping takes on a heap this size.
    #[test]
    fn the_figure_comes_to_rest() {
        let mut run = Run::new();
        for _ in 0..480 {
            run.step(1.0 / 120.0);
        }
        run.push();

        let mut ticks = 0;
        while run.phase() != Phase::Over && ticks < 9000 {
            run.step(1.0 / 120.0);
            ticks += 1;
        }
        assert_eq!(run.phase(), Phase::Over, "it never came to rest");
        // Measured at twenty seconds, nearly all of it the wave still
        // running. The ring is what takes the time: it is one wave going the
        // whole way round, and the S and the dots branch off it and run beside
        // it rather than after it.
        assert!(
            ticks < 2880,
            "it took {} seconds to stop",
            ticks as f32 / 120.0
        );
        assert_eq!(run.moving(), 0);
    }

    /// Spec 0002: nothing laid stands inside anything else.
    ///
    /// Not a claim about the pattern's points, which hug each other where the
    /// S meets the ring: two circles tangent at a place run alongside each
    /// other near it, and that is the shape rather than a mistake. The laying
    /// is what resolves it, by leaving out whatever will not fit.
    #[test]
    fn nothing_laid_overlaps() {
        let run = Run::new();

        for (n, one) in run.dominoes().iter().enumerate() {
            for other in &run.dominoes()[n + 1..] {
                assert!(
                    !overlap(one, other),
                    "two of them stand inside each other at {} and {}",
                    one.position,
                    other.position
                );
            }
        }
    }

    #[test]
    fn a_new_run_is_already_laid() {
        let mut run = Run::new();

        // every point gets one: no two pieces come near enough to crowd each
        // other out
        let wanted: usize = crate::pattern::all_of_it()
            .iter()
            .map(|part| part.len())
            .sum();
        assert_eq!(
            run.dominoes().len(),
            wanted,
            "only {} of {} points got one",
            run.dominoes().len(),
            wanted
        );
        assert!(run.left() > 10, "only {} left to play with", run.left());

        for _ in 0..360 {
            run.step(1.0 / 120.0);
        }
        assert_eq!(
            run.fallen(),
            0,
            "{} of it fell over on its own",
            run.fallen()
        );
        assert_eq!(run.moving(), 0, "it never settled");
    }

    /// Spec 0002: and starting again lays it again.
    #[test]
    fn starting_again_lays_it_again() {
        let one = Run::new();
        let other = Run::new();

        assert_eq!(one.dominoes().len(), other.dominoes().len());
        for (a, b) in one.dominoes().iter().zip(other.dominoes()) {
            assert_eq!(a.position, b.position);
            assert_eq!(a.orientation, b.orientation);
        }
    }

    /// Spec 0002: the whole reason for that shape. One push, and all of it
    /// goes over.
    ///
    /// Not most of it, and not four pushes. The figure is a tree: the S
    /// branches off the ring and each dot's spur branches off the S, so the
    /// wave spreads from the one domino that is pushed. No path rejoins
    /// another, so it never meets itself coming back, which is the other way a
    /// run strands dominoes.
    #[test]
    fn the_whole_figure_goes_over() {
        let mut run = Run::new();
        for _ in 0..360 {
            run.step(1.0 / 120.0);
        }

        let laid = run.dominoes().len();
        run.push();
        until_settled(&mut run, 9000);

        assert_eq!(
            run.fallen(),
            laid,
            "{} of {} were left standing",
            laid - run.fallen(),
            laid
        );
    }

    /// Scratch: can a wave split in two?
    #[test]
    fn a_new_run_is_empty() {
        let run = Run::bare();

        assert_eq!(run.dominoes().len(), 0);
        assert_eq!(run.left(), SUPPLY);
        assert_eq!(run.fallen(), 0);
        assert_eq!(run.phase(), Phase::Laying);
    }

    #[test]
    fn laying_one_spends_it() {
        let mut run = Run::bare();

        assert!(run.lay(Vec3::ZERO));
        assert_eq!(run.dominoes().len(), 1);
        assert_eq!(run.left(), SUPPLY - 1);
    }

    #[test]
    fn they_do_not_overlap() {
        let mut run = Run::bare();
        assert!(run.lay(Vec3::ZERO));

        assert!(!run.lay(Vec3::ZERO), "one went inside another");
        assert!(!run.lay(vec3(0.05, 0.0, 0.0)), "one went inside another");
        assert!(
            run.lay(vec3(domino::TALL, 0.0, 0.0)),
            "a clear one was refused"
        );
    }

    #[test]
    fn an_empty_supply_lays_nothing() {
        let mut run = Run::bare();
        for n in 0..SUPPLY {
            assert!(run.lay(vec3(n as f32 * domino::TALL, 0.0, 0.0)));
        }

        assert_eq!(run.left(), 0);
        assert!(
            !run.lay(vec3(1000.0, 0.0, 0.0)),
            "it laid one it did not have"
        );
    }

    /// Each stands across the line from the one before, and the first turns to
    /// face the second when it arrives.
    #[test]
    fn they_stand_across_the_line() {
        let mut run = Run::bare();
        run.lay(Vec3::ZERO);
        run.lay(vec3(0.0, 0.0, 2.0));

        for one in run.dominoes() {
            let thin = one.orientation * Vec3::X;
            assert!(
                thin.dot(Vec3::Z) > 0.99,
                "it stands {} where the run goes +z",
                thin
            );
        }
    }

    #[test]
    fn a_laid_run_sleeps() {
        let run = run_of(10, 0.6);

        assert_eq!(run.moving(), 0, "a run standing still is moving");
        assert_eq!(run.fallen(), 0);
    }

    #[test]
    fn nothing_falls_on_its_own() {
        let mut run = run_of(10, 0.6);
        for _ in 0..600 {
            run.step(1.0 / 120.0);
        }

        assert_eq!(run.fallen(), 0, "it went over by itself");
        assert_eq!(
            run.phase(),
            Phase::Laying,
            "it started without being pushed"
        );
    }

    /// Spec 0001: the whole reason for the game. The push wakes the first, and
    /// what it touches wakes before it has been knocked over.
    #[test]
    fn the_waking_runs_ahead() {
        let mut run = run_of(20, 0.6);
        run.push();

        let mut ahead = 0;
        for _ in 0..240 {
            run.step(1.0 / 120.0);
            ahead = ahead.max(run.moving().saturating_sub(run.fallen()));
        }

        assert!(
            ahead >= 2,
            "only {} were ever awake and still standing",
            ahead
        );
    }

    #[test]
    fn a_good_spacing_carries() {
        let mut run = run_of(20, 0.6);
        run.push();
        until_settled(&mut run, 3600);

        assert_eq!(run.fallen(), 20, "only {} of 20 went over", run.fallen());
    }

    #[test]
    fn too_far_apart_falls_short() {
        let mut run = run_of(20, domino::TALL * 1.3);
        run.push();
        until_settled(&mut run, 3600);

        assert!(
            run.fallen() < 4,
            "{} went over at a spacing further than they are tall",
            run.fallen()
        );
    }

    /// Spec 0001 asked whether a push that is too hard throws the first one
    /// over the second rather than into it. It does: past about four times
    /// what is needed, the first one leaves and nothing else moves.
    #[test]
    fn a_push_too_hard_goes_over_the_next() {
        let mut run = run_of(20, 0.6);

        let first = run.dominoes[0];
        let way = (run.dominoes[1].position - first.position).normalize_or_zero();
        let at = first.position + Vec3::Y * HALF.y * PUSHED_AT - way * HALF.x;
        run.dominoes[0].strike(way * 5.0, at);
        run.phase = Phase::Falling;
        until_settled(&mut run, 3600);

        assert!(
            run.fallen() < 4,
            "{} went over, so a shove of ten times is not too hard",
            run.fallen()
        );
    }

    #[test]
    fn the_count_is_what_fell() {
        let mut run = run_of(20, 0.6);
        assert_eq!(run.fallen(), 0, "they fell while being laid");

        run.push();
        until_settled(&mut run, 3600);

        assert_eq!(run.dominoes().len(), 20, "the count is of what was laid");
        assert_eq!(run.fallen(), 20);
    }
}
