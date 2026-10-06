//! A run: the dominoes laid, the push, and what fell. Spec 0001.

use crate::domino::{self, HALF};
use blitzkit::collision::{obbs_meet, Aabb, Obb};
use blitzkit::physics::{Body, Shape, Solver};
use glam::{vec3, Quat, Vec3};

/// How many there are to lay. The figure spends most of them and the rest are
/// the player's.
pub const SUPPLY: usize = 180;

/// How hard the first one is pushed, and where on it.
///
/// Near the top, which is where a finger goes, and only hard enough to put it
/// past its own balance: the rest is gravity.
///
/// Measured, a run of twenty at six tenths of their height:
///
/// ```text
/// 0.8            nothing moves
/// 1.0 to 100     all twenty go over
/// 200            two
/// ```
///
/// Spec 0001 wondered whether a shove hard enough throws the first one over
/// the top of the second rather than into it. This said yes: past four times
/// what was needed, one went over and nothing else moved. That was the engine
/// rather than the dominoes. A sleeping body was a wall to the solver for the
/// step it was run into, so the first one sailed over a second that could not
/// be moved. Blitzkit wakes what a moving body is touching before the passes
/// now, and a hard shove carries like any other until 200, which is ninety
/// times what is needed.
pub const PUSH: f32 = 0.5 * QUICKER;
pub const PUSHED_AT: f32 = 0.8;

/// How hard a click pushes, which is harder than the figure's own start.
///
/// A start only has to put the first one past its balance and let gravity do
/// the rest, and the next one along is square on and tips over its own
/// thickness. A click is a finger, and what it is pushed into may be anything.
/// A domino standing across the line is hit on its narrow end and rocks over a
/// base two and a half times wider.
///
/// Measured, a domino pushed into the side of another at three spacings, and
/// the same shove used to set off a straight run of twenty:
///
/// ```text
/// 2.2 to 4.0   the one across stands, every spacing. the run goes 20/20
/// 5.0 to 8.0   the one across goes over, every spacing. the run goes 20/20
/// ```
///
/// So a start and a click are not the same shove, which is what this spec said
/// first. Nothing is lost at the top: spec 0001 measured a straight run
/// carrying anything up to a hundred.
pub const SHOVE: f32 = 1.35 * QUICKER;

/// How long everything has to be still before a run is over, and how still.
///
/// Its own measure, which it needed and now barely does.
///
/// Sleeping in the engine is a property of a whole group of touching bodies.
/// Nothing sleeps while anything it touches is moving, so a fallen figure is
/// one connected group of ninety, and the clock restarts whenever any of them
/// stirs. At blitzkit's old eight solver passes 75 of the 92 went on stirring
/// long after the figure had stopped, the clock never reached half a second,
/// and it never slept at all.
///
/// That was the solver rather than the bookkeeping, and blitzkit's passes are
/// thirty two now. None of them stir, and it sleeps at 20.2 seconds, against the
/// 20.5 this measure calls it over at. So this is near enough redundant, and it
/// is kept because it does not depend on what the engine's defaults happen to
/// be.
pub const RESTS_FOR: f32 = 0.8 / QUICKER;
pub const RESTS_UNDER: f32 = 0.1 * QUICKER;

/// How big one of this world's units is, in metres.
///
/// A domino here is one unit tall. At the engine's own 9.81 that makes it a
/// metre tall, and it topples like a metre of concrete. Measured, a wave went
/// down a straight run at two and a half dominoes a second.
///
/// A real domino is about five centimetres, so gravity is what a five
/// centimetre unit would feel. Time then goes as the square root of that, so
/// everything here measured as a speed is `QUICKER` times what it was and
/// everything measured as a duration is that much less. Measured down a
/// straight run of twenty:
///
/// ```text
/// as it was    0.389s a domino,  2.6 a second
/// x4           0.195s            5.1
/// x16          0.102s            9.8
/// x20          0.089s           10.5
/// x36          0.070s           14.3
/// ```
pub const UNIT: f32 = 0.05;
pub const GRAVITY: Vec3 = vec3(0.0, -9.81 / UNIT, 0.0);
/// The square root of one over `UNIT`, which is what every speed here is
/// multiplied by. Written out because a square root is not a constant.
pub const QUICKER: f32 = 4.472;

/// How many solver steps a frame is cut into.
///
/// Gravity here is twenty times the engine's, and the engine's sleep threshold
/// is the speed gravity gives a body in a couple of frames. At 196 and a
/// hundred and twentieth of a second that is 3.27, a third of the speed the
/// wave travels at, so a domino part way over counted as still and was slept
/// leaning. Smaller steps bring it down in proportion.
///
/// Measured, a domino pushed into the side of another at nine angles and
/// spacings, and then left for a minute after the run said it was over:
///
/// ```text
/// 1 slice    a tile moved 84.9 degrees more
/// 2          2.6
/// 4          0.0
/// ```
///
/// It is bought with solver work: 3411 microseconds of a frame on the whole
/// figure against a budget of 8333, where one slice is 880.
pub const SLICES: usize = 4;
pub const GROUND: f32 = 30.0;

/// How much speed a domino has to lose in a step to count as having hit
/// something, and the loss that counts as flat out.
pub const MIN_KNOCK: f32 = 0.4 * QUICKER;
pub const LOUDEST_KNOCK: f32 = 3.0 * QUICKER;
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
    /// How many had gone over when the last push was made, so a push that took
    /// nothing with it can be told from one that did.
    fell_before: usize,
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
            fell_before: 0,
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

    /// Stands one up at a point on the floor, facing the way it is given. Says
    /// whether it went down.
    ///
    /// The direction is the player's, per spec 0005. Spec 0001 worked it out
    /// from the domino before and spec 0003 added a rule for a click out on
    /// its own. Between them a heading was never the player's, and one already
    /// on the floor could be turned by the next arriving.
    ///
    /// At any point in a run, also per spec 0005. Spec 0003 made pushing
    /// something that can be done whenever, and laying had been left where spec
    /// 0001 put it: before the first push and never again.
    pub fn lay(&mut self, at: Vec3, way: Vec3) -> bool {
        if self.left() == 0 {
            return false;
        }

        let laid = domino::standing(at, way);
        if self.dominoes.iter().any(|other| overlap(&laid, other)) {
            return false;
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

        self.fell_before = self.fallen();
        for (first, second) in std::mem::take(&mut self.starts) {
            let from = self.dominoes[first];
            let way = (self.dominoes[second].position - from.position).normalize_or_zero();
            let at = from.position + Vec3::Y * HALF.y * PUSHED_AT - way * HALF.x;

            self.dominoes[first].strike(way * PUSH, at);
        }

        self.phase = Phase::Falling;
        self.still = 0.0;
    }

    /// Which domino a ray meets first, if any. The slab test in each one's own
    /// frame, which is the only way to pick a thing that is lying at an angle.
    pub fn under(&self, from: Vec3, way: Vec3) -> Option<usize> {
        let mut nearest: Option<(usize, f32)> = None;

        for (which, one) in self.dominoes.iter().enumerate() {
            let Some(far) = hit(from, way, one.position, one.orientation, HALF) else {
                continue;
            };
            if nearest.is_none_or(|(_, best)| far < best) {
                nearest = Some((which, far));
            }
        }

        nearest.map(|(which, _)| which)
    }

    /// Knocks one over, any of them and at any point in a run. Spec 0003.
    ///
    /// Away from whoever asked: a domino goes two ways along its thin axis, so
    /// the ray is flattened onto the floor and taken against that axis to pick
    /// the sign.
    ///
    /// Struck near the top, and harder than the figure's own start: see `SHOVE`.
    pub fn shove(&mut self, which: usize, way: Vec3) -> bool {
        let Some(one) = self.dominoes.get(which) else {
            return false;
        };

        let flat = vec3(way.x, 0.0, way.z).normalize_or_zero();
        let thin = one.orientation * Vec3::X;
        let thin = vec3(thin.x, 0.0, thin.z).normalize_or_zero();
        if flat == Vec3::ZERO || thin == Vec3::ZERO {
            return false;
        }

        let going = thin * thin.dot(flat).signum();
        let at = one.position + Vec3::Y * HALF.y * PUSHED_AT - going * HALF.x;
        self.dominoes[which].strike(going * SHOVE, at);

        if self.phase != Phase::Falling {
            self.fell_before = self.fallen();
            self.phase = Phase::Falling;
        }
        self.still = 0.0;
        true
    }

    pub fn step(&mut self, dt: f32) {
        self.step_sliced(dt, SLICES)
    }

    pub fn step_sliced(&mut self, dt: f32, slices: usize) {
        if self.dominoes.is_empty() {
            return;
        }

        let was: Vec<f32> = self
            .dominoes
            .iter()
            .map(|one| one.velocity.length())
            .collect();

        // Several smaller steps rather than one of the frame's length. Gravity
        // here is twenty times the engine's, and its sleep threshold is the
        // speed gravity gives a body in a couple of frames. At 196 and a 120th
        // of a second that is 3.27, a third of the speed the wave travels at.
        // Dominoes were being put to sleep part way over and freezing at 25
        // degrees. Smaller steps bring it back down in proportion.
        let dt = dt / slices as f32;
        for _ in 0..slices {
            self.solver
                .step(&mut self.dominoes, &self.ground, GRAVITY, dt);
        }

        self.listen(&was);

        if self.phase == Phase::Falling {
            if self.moving() == 0 {
                self.still += dt;
                if self.still > RESTS_FOR {
                    // A shove that took nothing with it is not a run that is
                    // over, it is a shove that did nothing. A domino pushed
                    // into the side of another leans on it and stops, which is
                    // what a real one does. Saying "0 of 137 went over" a
                    // fifth of a second after the click reads as the game
                    // being finished.
                    self.phase = if self.fallen() > self.fell_before {
                        Phase::Over
                    } else {
                        Phase::Laying
                    };
                }
            } else {
                self.still = 0.0;
            }
        }
    }

    /// Writes down what hit something. An impact is a loss of speed, and only
    /// the loudest few of a step are kept. A run going over is dozens at once,
    /// and all of them together is one bang.
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

/// Where a ray first meets a block: the slab test, in the block's own frame.
///
/// The same few lines as cairn's, which is two games now and an argument for
/// the engine owning it.
fn hit(from: Vec3, way: Vec3, middle: Vec3, turn: Quat, half: Vec3) -> Option<f32> {
    let back = turn.inverse();
    let began = back * (from - middle);
    let along = back * way;

    let mut entry = f32::NEG_INFINITY;
    let mut exit = f32::INFINITY;

    for n in 0..3 {
        if along[n].abs() < 1e-6 {
            if began[n].abs() > half[n] {
                return None;
            }
            continue;
        }

        let (near, far) = (
            (-half[n] - began[n]) / along[n],
            (half[n] - began[n]) / along[n],
        );
        entry = entry.max(near.min(far));
        exit = exit.min(near.max(far));
    }

    (exit >= entry.max(0.0)).then_some(entry.max(0.0))
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
                run.lay(vec3(n as f32 * apart, 0.0, 0.0), Vec3::X),
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

    /// Spec 0002: the figure comes to rest and the run ends, which the
    /// engine's own sleeping never does on a figure this size.
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
        // Measured at 3.5 seconds of wave and 4.2 to being called over. The
        // ring is what takes the time: it is one wave going the whole way
        // round, and the S and the dots branch off it and run beside it rather
        // than after it.
        //
        // It read twenty seconds until 2026-10-04, measured before gravity
        // became a unit of five centimetres and the frame was cut into four
        // solver steps. Both made the wave faster and neither touched this
        // number.
        assert!(
            ticks < 960,
            "it took {} seconds to stop",
            ticks as f32 / 120.0
        );
        assert_eq!(run.moving(), 0);
    }

    /// Spec 0002: nothing laid stands inside anything else.
    ///
    /// Not a claim about the pattern's points, which hug each other where the
    /// S meets the ring. Two circles tangent at a place run alongside each
    /// other near it, and that is the shape. The laying is what resolves it,
    /// by leaving out whatever will not fit.
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
    /// A ray aimed at a domino from a given way off, level with its top, which
    /// is roughly where a click from the camera arrives.
    fn aimed_at(run: &Run, which: usize, from: Vec3) -> (Vec3, Vec3) {
        let at = run.dominoes()[which].position;
        let eye = at + from;

        (eye, (at - eye).normalize())
    }

    /// Spec 0003: clicking a domino knocks it over.
    #[test]
    fn a_click_pushes_what_it_hits() {
        let mut run = Run::new();
        for _ in 0..360 {
            run.step(1.0 / 120.0);
        }

        // one out on the ring, well away from the figure's own start
        let which = run.dominoes().len() / 4;
        let (eye, way) = aimed_at(&run, which, vec3(4.0, 3.0, 4.0));
        assert_eq!(run.under(eye, way), Some(which), "the ray missed it");
        assert!(run.shove(which, way));
        assert_eq!(run.phase(), Phase::Falling);

        until_settled(&mut run, 9000);
        assert!(
            domino::has_fallen(&run.dominoes()[which]),
            "it was pushed and stayed up"
        );
    }

    /// Spec 0003: and it goes away from whoever pushed it.
    #[test]
    fn it_falls_away_from_the_click() {
        for from in [vec3(4.0, 3.0, 4.0), vec3(-4.0, 3.0, -4.0)] {
            let mut run = Run::new();
            for _ in 0..360 {
                run.step(1.0 / 120.0);
            }

            let which = 24;
            let stood = run.dominoes()[which].position;
            let (_, way) = aimed_at(&run, which, from);
            assert!(run.shove(which, way));
            until_settled(&mut run, 9000);

            let went = run.dominoes()[which].position - stood;
            let pushed = vec3(way.x, 0.0, way.z).normalize();
            assert!(
                vec3(went.x, 0.0, went.z).normalize_or_zero().dot(pushed) > 0.0,
                "pushed {} and it went {}",
                pushed,
                went
            );
        }
    }

    /// Spec 0003: one the player laid is pushed like any other.
    #[test]
    fn one_the_player_laid_can_be_pushed() {
        let mut run = Run::new();
        // off on its own, clear of the figure
        let mine = vec3(11.0, 0.0, 0.0);
        let apart = domino::TALL * 0.65;
        for n in 0..3 {
            assert!(run.lay(mine + vec3(0.0, 0.0, n as f32 * apart), Vec3::Z));
        }
        let which = run.dominoes().len() - 3;
        for _ in 0..360 {
            run.step(1.0 / 120.0);
        }

        let (eye, way) = aimed_at(&run, which, vec3(0.0, 3.0, -4.0));
        assert_eq!(run.under(eye, way), Some(which));
        assert!(run.shove(which, way));
        until_settled(&mut run, 9000);

        assert!(
            domino::has_fallen(&run.dominoes()[which]),
            "the one the player laid stayed up"
        );
        assert!(
            domino::has_fallen(&run.dominoes()[which + 1]),
            "and it did not take the next one with it"
        );
    }

    /// Spec 0003: a ray that meets nothing standing is a click on the floor.
    #[test]
    fn a_click_on_the_floor_still_lays() {
        let mut run = Run::new();
        let was = run.dominoes().len();

        let bare = vec3(12.0, 0.0, 12.0);
        let eye = bare + vec3(0.0, 8.0, 0.0);
        assert_eq!(
            run.under(eye, (bare - eye).normalize()),
            None,
            "it found something standing on bare floor"
        );

        assert!(run.lay(bare, Vec3::X));
        assert_eq!(run.dominoes().len(), was + 1);
    }

    /// Spec 0003: and a run that has come to rest can be set going again.
    #[test]
    fn a_finished_run_can_be_poked() {
        let mut run = Run::new();
        for _ in 0..360 {
            run.step(1.0 / 120.0);
        }
        let standing = vec3(11.0, 0.0, 0.0);
        assert!(run.lay(standing, Vec3::X));
        let mine = run.dominoes().len() - 1;

        run.push();
        until_settled(&mut run, 12000);
        assert_eq!(run.phase(), Phase::Over);
        assert!(
            !domino::has_fallen(&run.dominoes()[mine]),
            "it fell on its own"
        );

        let (_, way) = aimed_at(&run, mine, vec3(0.0, 3.0, -4.0));
        assert!(run.shove(mine, way));
        assert_eq!(run.phase(), Phase::Falling, "it would not start again");

        until_settled(&mut run, 9000);
        assert_eq!(run.phase(), Phase::Over);
        assert!(domino::has_fallen(&run.dominoes()[mine]));
    }

    /// Spec 0003: and what goes over afterwards counts with the rest.
    /// Spec 0003: which way the figure is pushed decides how much of it goes,
    /// and the lesser answer is allowed to happen.
    ///
    /// A branch is a domino half a step ahead of the stem and inside it,
    /// turned to topple along the branch. Struck from behind it goes; struck
    /// from the front it falls backwards, away from the branch. So a wave
    /// running the ring the designed way takes the S and the dots with it and
    /// a wave running it the other way takes the ring alone.
    ///
    /// Held here so that a change which quietly makes a click run the figure's
    /// way, or which breaks the forward case, is caught rather than noticed in
    /// a screenshot.
    #[test]
    fn which_way_the_figure_is_pushed_decides_how_much_goes() {
        let ring = crate::pattern::all_of_it()[0].len();

        let run_it = |against: bool| {
            let mut run = Run::new();
            let thin = run.dominoes()[0].orientation * Vec3::X;
            run.shove(0, if against { -thin } else { thin });

            let dt = 1.0 / 60.0;
            let mut at = 0.0f32;
            while run.phase() != Phase::Over && at < 30.0 {
                run.step(dt);
                at += dt;
            }
            (run.fallen(), run.dominoes().len())
        };

        let (with, all) = run_it(false);
        assert_eq!(
            with, all,
            "pushed its own way, {} of {} went over",
            with, all
        );

        let (against, _) = run_it(true);
        assert!(
            against > ring / 2 && against < all,
            "pushed against it, {} of {} went over, with a ring of {}",
            against,
            all,
            ring
        );
    }

    #[test]
    fn a_later_push_adds_to_the_count() {
        let mut run = Run::new();
        for _ in 0..360 {
            run.step(1.0 / 120.0);
        }
        let apart = domino::TALL * 0.65;
        for n in 0..4 {
            assert!(run.lay(vec3(11.0, 0.0, n as f32 * apart), Vec3::Z));
        }
        let mine = run.dominoes().len() - 4;

        run.push();
        until_settled(&mut run, 12000);
        let figure = run.fallen();

        let (_, way) = aimed_at(&run, mine, vec3(0.0, 3.0, -4.0));
        assert!(run.shove(mine, way));
        until_settled(&mut run, 9000);

        assert_eq!(
            run.fallen(),
            figure + 4,
            "the four laid aside did not join the count"
        );
    }

    /// Spec 0005: a domino is laid at the heading it was given, and nothing
    /// works it out from anything else.
    #[test]
    fn it_is_laid_facing_where_it_was_aimed() {
        let mut run = Run::new();

        for (at, way) in [
            (vec3(11.0, 0.0, 0.0), Vec3::X),
            (vec3(-9.0, 0.0, 4.0), Vec3::Z),
            (vec3(0.0, 0.0, 12.0), vec3(1.0, 0.0, 1.0).normalize()),
        ] {
            assert!(run.lay(at, way), "it would not lay one at {}", at);

            let stood = run.dominoes().last().expect("one was laid");
            let thin = stood.orientation * Vec3::X;
            assert!(
                thin.dot(way.normalize()) > 0.999,
                "aimed {} and it stands {}",
                way,
                thin
            );
        }
    }

    /// Spec 0005: and laying another never turns it afterwards.
    #[test]
    fn laying_one_never_turns_another() {
        let mut run = Run::bare();
        assert!(run.lay(Vec3::ZERO, Vec3::X));
        let was = run.dominoes()[0].orientation;

        // one beside it, pointing somewhere else entirely
        assert!(run.lay(vec3(0.0, 0.0, domino::TALL * 0.65), Vec3::Z));
        assert_eq!(
            run.dominoes()[0].orientation,
            was,
            "the first one turned when the second arrived"
        );

        // and a third, far off, which spec 0003 would have made a fresh head
        assert!(run.lay(vec3(9.0, 0.0, 9.0), Vec3::X));
        assert_eq!(run.dominoes()[0].orientation, was);
    }

    /// Spec 0005: laying works at any point in a run.
    #[test]
    fn laying_works_at_any_point() {
        let mut run = Run::new();
        for _ in 0..600 {
            run.step(1.0 / 120.0);
        }

        let was = run.dominoes().len();
        run.push();
        assert_eq!(run.phase(), Phase::Falling);
        assert!(
            run.lay(vec3(12.0, 0.0, 12.0), Vec3::X),
            "it would not lay one while the wave was running"
        );

        until_settled(&mut run, 12000);
        assert_eq!(run.phase(), Phase::Over);
        assert!(
            run.lay(vec3(12.0, 0.0, -12.0), Vec3::X),
            "it would not lay one after everything had stopped"
        );
        assert_eq!(run.dominoes().len(), was + 2);
    }

    /// Spec 0001: when a run says it is over, it is over.
    ///
    /// A domino toppling onto the side of another leans against it and comes to
    /// rest there, which is a real thing for a tile hit on its narrow end. What
    /// was not real was it being called finished and then keeling over a minute
    /// later, which is what a frame too coarse for this gravity did.
    #[test]
    fn nothing_moves_after_it_is_over() {
        let mut worst: f32 = 0.0;

        for degrees in [60.0f32, 90.0, 120.0] {
            for apart in [0.6f32, 0.7, 0.8] {
                let angle = degrees.to_radians();
                let mut run = Run::bare();
                run.dominoes.push(domino::standing(Vec3::ZERO, Vec3::X));
                let out = vec3(angle.cos(), 0.0, angle.sin()) * apart;
                let way = -out.normalize();
                run.dominoes.push(domino::standing(out, way));
                run.solver.forget();
                for _ in 0..360 {
                    run.step(1.0 / 120.0);
                }

                let from = run.dominoes[1];
                let at = from.position + Vec3::Y * HALF.y * PUSHED_AT - way * HALF.x;
                run.dominoes[1].strike(way * PUSH, at);
                run.phase = Phase::Falling;
                until_settled(&mut run, 3600);

                let tilt = |run: &Run, n: usize| {
                    (run.dominoes()[n].orientation * Vec3::Y)
                        .dot(Vec3::Y)
                        .clamp(-1.0, 1.0)
                        .acos()
                        .to_degrees()
                };
                let was = [tilt(&run, 0), tilt(&run, 1)];
                for _ in 0..(20 * 120) {
                    run.step(1.0 / 120.0);
                }

                worst = worst
                    .max((tilt(&run, 0) - was[0]).abs())
                    .max((tilt(&run, 1) - was[1]).abs());
            }
        }

        assert!(
            worst < 1.0,
            "a tile moved {} degrees after the run said it was over",
            worst
        );
    }

    /// Spec 0003: a shove that takes nothing with it leaves the run where it
    /// was, rather than ending it with a score of nothing.
    ///
    /// Clicking one that is already down is the plainest case of it. The game
    /// used to call that a finished run a fifth of a second later and print
    /// "0 of 137 went over".
    #[test]
    fn a_shove_that_takes_nothing_is_not_a_run() {
        let mut run = Run::bare();
        run.dominoes.push(domino::standing(Vec3::ZERO, Vec3::X));
        run.solver.forget();

        // put it down first, well clear of anything
        assert!(run.shove(0, Vec3::X));
        for _ in 0..600 {
            run.step(1.0 / 120.0);
        }
        assert_eq!(run.phase(), Phase::Over, "the one that fell did not count");
        assert_eq!(run.fallen(), 1);

        // and now shove the one that is already lying there
        assert!(run.shove(0, Vec3::X));
        assert_eq!(run.phase(), Phase::Falling, "it did not start");
        for _ in 0..900 {
            run.step(1.0 / 120.0);
        }

        assert_eq!(run.fallen(), 1, "something else went over");
        assert_eq!(
            run.phase(),
            Phase::Laying,
            "a shove that took nothing with it ended the run"
        );
        assert!(
            run.lay(vec3(6.0, 0.0, 0.0), Vec3::X),
            "it would not lay another"
        );
    }

    /// Spec 0003: and a click takes the one standing across its path with it.
    ///
    /// Which is what a click is for. A domino hit on its narrow end has to
    /// rock over a base two and a half times wider than the one it tips over
    /// going forwards, and the figure's own start has nowhere near enough. A
    /// clicked tile leant against its neighbour and the wave stopped there.
    #[test]
    fn a_click_takes_the_one_across_its_path() {
        for gap in [0.6f32, 0.7, 0.8] {
            let mut run = Run::bare();
            run.dominoes.push(domino::standing(Vec3::ZERO, Vec3::X));
            run.dominoes
                .push(domino::standing(vec3(0.0, 0.0, gap), -Vec3::Z));
            run.solver.forget();
            for _ in 0..360 {
                run.step(1.0 / 120.0);
            }

            assert!(run.shove(1, -Vec3::Z));
            for _ in 0..1200 {
                run.step(1.0 / 120.0);
            }

            assert_eq!(
                run.fallen(),
                2,
                "at {} apart only {} of the two went over",
                gap,
                run.fallen()
            );
        }
    }

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

        assert!(run.lay(Vec3::ZERO, Vec3::X));
        assert_eq!(run.dominoes().len(), 1);
        assert_eq!(run.left(), SUPPLY - 1);
    }

    #[test]
    fn they_do_not_overlap() {
        let mut run = Run::bare();
        assert!(run.lay(Vec3::ZERO, Vec3::X));

        assert!(!run.lay(Vec3::ZERO, Vec3::X), "one went inside another");
        assert!(
            !run.lay(vec3(0.05, 0.0, 0.0), Vec3::X),
            "one went inside another"
        );
        assert!(
            run.lay(vec3(domino::TALL, 0.0, 0.0), Vec3::X),
            "a clear one was refused"
        );
    }

    #[test]
    fn an_empty_supply_lays_nothing() {
        let mut run = Run::bare();
        for n in 0..SUPPLY {
            assert!(run.lay(vec3(n as f32 * domino::TALL, 0.0, 0.0), Vec3::X));
        }

        assert_eq!(run.left(), 0);
        assert!(
            !run.lay(vec3(1000.0, 0.0, 0.0), Vec3::X),
            "it laid one it did not have"
        );
    }

    /// Spec 0005: a domino stands thin way along the way it was aimed, which is
    /// what makes a run a run rather than a wall.
    #[test]
    fn they_stand_across_the_line() {
        let mut run = Run::bare();
        run.lay(Vec3::ZERO, Vec3::Z);
        run.lay(vec3(0.0, 0.0, domino::TALL * 0.65), Vec3::Z);

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
