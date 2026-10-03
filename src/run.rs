//! A run: the dominoes laid, the push, and what fell. Spec 0001.

use crate::domino::{self, HALF};
use blitzkit::collision::{obbs_meet, Aabb, Obb};
use blitzkit::physics::{Body, Shape, Solver};
use glam::{vec3, Vec3};

/// How many there are to lay. Enough for a long run or a short one with a
/// shape to it.
pub const SUPPLY: usize = 60;

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

/// How long everything has to be still before a run is over.
pub const RESTS_FOR: f32 = 1.0;

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
}

impl Default for Run {
    fn default() -> Self {
        Self::new()
    }
}

impl Run {
    pub fn new() -> Self {
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
        }
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

    /// How many are awake, which is the wave made visible.
    pub fn awake(&self) -> usize {
        self.dominoes.iter().filter(|one| !one.asleep).count()
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
        self.solver.forget();
        true
    }

    /// Pushes the first one over, which is the last thing the player does.
    pub fn push(&mut self) {
        if self.phase != Phase::Laying || self.dominoes.len() < 2 {
            return;
        }

        let first = self.dominoes[0];
        let way = (self.dominoes[1].position - first.position).normalize_or_zero();
        let at = first.position + Vec3::Y * HALF.y * PUSHED_AT - way * HALF.x;

        self.dominoes[0].strike(way * PUSH, at);
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
            if self.dominoes.iter().all(|one| one.asleep) {
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
        let mut run = Run::new();
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
        let mut run = Run::new();
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

    #[test]
    fn a_new_run_is_empty() {
        let run = Run::new();

        assert_eq!(run.dominoes().len(), 0);
        assert_eq!(run.left(), SUPPLY);
        assert_eq!(run.fallen(), 0);
        assert_eq!(run.phase(), Phase::Laying);
    }

    #[test]
    fn laying_one_spends_it() {
        let mut run = Run::new();

        assert!(run.lay(Vec3::ZERO));
        assert_eq!(run.dominoes().len(), 1);
        assert_eq!(run.left(), SUPPLY - 1);
    }

    #[test]
    fn they_do_not_overlap() {
        let mut run = Run::new();
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
        let mut run = Run::new();
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
        let mut run = Run::new();
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

        assert_eq!(run.awake(), 0, "a run standing still is awake");
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
            ahead = ahead.max(run.awake().saturating_sub(run.fallen()));
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
