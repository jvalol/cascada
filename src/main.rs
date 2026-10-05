//! cascada: dominoes on a floor. See `specs/`.

mod domino;
mod knock;
mod pattern;
mod pips;
mod run;

use blitzkit::camera::Camera;
use blitzkit::collision::Ray;
use blitzkit::geometry::Geometry;
use blitzkit::keyboard::{KeyboardInput, KeyboardKey, KeyboardKeyState};
use blitzkit::mesh::{MeshData, Transform};
use blitzkit::mouse::{MouseButton, MouseInput};
use blitzkit::notice;
use blitzkit::physics::Shape;
use blitzkit::renderer::render_text::{RenderText, TextRenderer};
use blitzkit::renderer::scene::{MeshId, Scene};
use blitzkit::renderer::Renderer;
use blitzkit::sound::SoundSystem;
use blitzkit::{start, Game};
use glam::{vec2, vec3, vec4, Vec2, Vec3};
use run::{Phase, Run};

const FLOOR: f32 = 30.0;

/// Whether this run is only here to be photographed, for `refresh-screenshots`
/// in the project above.
///
/// A game's opening frame rarely shows what the game is about, and cascada's
/// never does: it opens on a figure standing still, which is every run before
/// anything has happened. The picture wants the wave partway through, and that
/// is a state nobody can reach by pressing a key at the right moment.
fn staged() -> bool {
    std::env::args().any(|arg| arg == "--screenshot")
}

/// How far into a run the picture is taken, in seconds.
///
/// Measured rather than guessed, and the first guess was nine seconds, by
/// which time it has been over for five and a half. The whole run is 3.5
/// seconds: at two, 62 of the 136 are down and 14 are still moving, so the
/// near half reads as fallen and the far half as still to come.
const PHOTOGENIC: f32 = 2.0;

/// How far an arrow turns the ghost. A twenty fourth of the way round, so a
/// quarter turn is six presses and the figure's own bends are reachable.
const TURN: f32 = std::f32::consts::TAU / 24.0;

/// Where a ray meets the floor, which is where a click lays a domino.
fn on_the_floor(ray: &Ray) -> Option<Vec3> {
    if ray.direction.y > -1e-4 {
        return None;
    }

    let at = ray.at(-ray.origin.y / ray.direction.y);

    (at.x.abs() < FLOOR * 0.5 && at.z.abs() < FLOOR * 0.5).then_some(at)
}

struct Cascada {
    domino_mesh: Option<MeshId>,
    /// One per tile of a double six set, so a domino can wear its own face.
    tiles: Vec<blitzkit::renderer::scene::TextureId>,
    floor_mesh: Option<MeshId>,
    run: Run,
    cursor: Vec2,
    /// Where on the floor the cursor is pointing, worked out in `draw` where the
    /// camera is.
    aimed: Option<Vec3>,
    /// And the ray it was worked out from, which is what picks a domino out.
    pointing: Option<Ray>,
    /// Which domino the cursor is over, worked out in `draw` with the camera
    /// and kept so the click and the readout agree with what is lit up.
    over: Option<usize>,
    /// Which way a domino laid now would face, per spec 0005. A world heading:
    /// the arrows turn it, and nothing else does. It starts pointing away from
    /// where the camera opens, which is where a first domino wants to fall.
    aim: f32,
    /// The one just laid, while the cursor has not left it yet.
    ///
    /// One button, and what is under the cursor decides what it means. Laying
    /// one puts it under the cursor, so the next click read as a push and there
    /// was no obvious way to lay a second. A domino is not pushable until the
    /// cursor has been off it once.
    just_laid: Option<usize>,
    /// Where the camera is and which way it faces, so the ears follow it.
    ears: Option<(Vec3, Vec3)>,
    /// How much sound is already queued and not yet played, in seconds.
    waiting: f32,
    /// Whether this run is held still to be photographed.
    ///
    /// Staging the state is not enough on its own. The run goes on stepping
    /// every frame, so the two seconds set up here ran to the end while the
    /// camera was still being pointed, and the picture came out of a finished
    /// run twice over.
    held: bool,
    camera_angle: f32,
    camera_up: f32,
    turning: bool,
    distance: f32,
    quitting: bool,
}

impl Cascada {
    fn new() -> Self {
        let mut run = Run::new();

        if staged() {
            // the physics is the game's own and needs no window, so the wave
            // is run forward here rather than waited for after one opens
            let dt = 1.0 / 60.0;
            run.push();
            for _ in 0..(PHOTOGENIC / dt) as usize {
                run.step(dt);
            }
        }

        Self {
            domino_mesh: None,
            tiles: Vec::new(),
            floor_mesh: None,
            held: staged(),
            run,
            cursor: Vec2::ZERO,
            aimed: None,
            pointing: None,
            over: None,
            aim: 2.5 + std::f32::consts::PI,
            just_laid: None,
            ears: None,
            waiting: 0.0,
            camera_angle: 2.5,
            camera_up: 0.55,
            turning: false,
            distance: 18.5,
            quitting: false,
        }
    }

    /// Which way the ghost is pointing.
    fn aimed_way(&self) -> Vec3 {
        vec3(self.aim.sin(), 0.0, self.aim.cos())
    }
}

impl Game for Cascada {
    fn load(&mut self, renderer: &mut Renderer) {
        self.domino_mesh = Some(renderer.add_mesh(&pips::tile_mesh()));
        self.tiles = pips::tiles()
            .iter()
            .map(|tile| renderer.add_texture(tile))
            .collect();
        self.floor_mesh = Some(renderer.add_mesh(&MeshData::plane()));
    }

    fn initialize(
        &mut self,
        _geometry: &mut Geometry,
        _text_renderer: &mut TextRenderer,
        _sound_system: &SoundSystem,
        _window_size: (f32, f32),
    ) {
    }

    fn update(
        &mut self,
        dt: f32,
        geometry: &mut Geometry,
        text_renderer: &mut TextRenderer,
        sound_system: &SoundSystem,
    ) {
        if !self.held {
            self.run.step(dt);
        }
        self.waiting = (self.waiting - dt).max(0.0);

        if let Some((at, facing)) = self.ears {
            sound_system.set_listener(at, facing, Vec3::Y);

            for hit in self.run.knocks() {
                // The engine plays what it is given one sound after another, so
                // a run going over handed across whole is still being heard once
                // everything has stopped.
                if !knock::room_for_another(self.waiting) {
                    break;
                }
                self.waiting += knock::SECONDS;

                // a stride from the ears rather than where it happened, since
                // the engine's spatial sound fades with the distance squared
                let towards = (hit.at - at).normalize_or_zero();
                sound_system.queue_spatial(
                    knock::knock(
                        knock::loudness(hit.force),
                        knock::colour_of(hit.which),
                        knock::seed_of(hit.which, hit.force),
                    ),
                    (at + towards * knock::EARSHOT).to_array(),
                );
            }
        }

        // What a click would do, said for whatever the cursor is actually over,
        // so the one rule of the hand does not have to be discovered.
        let saying = match (self.over, self.run.phase()) {
            (Some(_), _) => String::from("Click a tile to push it over."),
            (None, Phase::Falling) => String::from("Over it goes. Whee!"),
            (None, Phase::Over) => format!(
                "{} of {} went over. Click to lay another, or press space to start again.",
                self.run.fallen(),
                self.run.dominoes().len()
            ),
            (None, _) if self.run.left() == 0 => {
                String::from("You have no tiles left. Press space to start again.")
            }
            (None, _) => String::from("To lay a new tile, click. Rotate it with the arrow keys."),
        };

        text_renderer.reset();
        let lines: Vec<RenderText> = vec![
            format!(
                "{} laid, {} left, {} moving",
                self.run.dominoes().len(),
                self.run.left(),
                self.run.moving()
            ),
            saying,
            String::from("right-drag turns and tilts, scroll zooms"),
        ]
        .into_iter()
        .enumerate()
        .map(|(line, text)| RenderText {
            position: vec2(20.0, 20.0 + line as f32 * 24.0),
            text,
            size: 14.0,
            ..Default::default()
        })
        .collect();

        // the readout goes on a panel, so it reads over the scene rather than
        // into it. See blitzkit's spec 0038.
        geometry.reset();
        if let Some(frame) = notice::framing_all(&lines) {
            for quad in frame.iter() {
                geometry.push_quad(quad);
            }
        }

        for line in lines {
            text_renderer.push_render_text(line);
        }
    }

    fn draw(&mut self, scene: &mut Scene, camera: &mut Camera) {
        let (domino_mesh, floor) = match (self.domino_mesh, self.floor_mesh) {
            (Some(domino_mesh), Some(floor)) => (domino_mesh, floor),
            _ => return,
        };

        scene.push_colored(
            floor,
            &Transform::at(Vec3::ZERO).with_scale(Vec3::splat(FLOOR)),
            vec4(0.16, 0.17, 0.20, 1.0),
        );

        for (which, body) in self.run.dominoes().iter().enumerate() {
            let Shape::Block { half } = body.shape else {
                continue;
            };

            let placed = Transform::at(body.position)
                .with_rotation(body.orientation)
                .with_scale(half * 2.0);

            match self.tiles.get(pips::worn_by(which)) {
                Some(tile) => {
                    scene.push_textured(domino_mesh, *tile, &placed, vec4(1.0, 1.0, 1.0, 1.0), 48.0)
                }
                None => {
                    scene.push_material(domino_mesh, &placed, vec4(0.92, 0.90, 0.86, 1.0), 64.0)
                }
            }
        }

        // where the next one would go, so a run can be aimed before it is laid
        if self.run.phase() == Phase::Laying && self.run.left() > 0 && self.over.is_none() {
            if let Some(at) = self.aimed {
                let ghost = domino::standing(at, self.aimed_way());

                scene.push_material(
                    domino_mesh,
                    &Transform::at(ghost.position)
                        .with_rotation(ghost.orientation)
                        .with_scale(domino::HALF * 2.0),
                    vec4(0.5, 0.62, 0.45, 0.45),
                    16.0,
                );
            }
        }

        camera.target = vec3(0.0, domino::TALL, 0.0);
        camera.position = camera.target
            + vec3(
                self.camera_angle.sin() * self.camera_up.cos() * self.distance,
                self.camera_up.sin() * self.distance,
                self.camera_angle.cos() * self.camera_up.cos() * self.distance,
            );
        self.ears = Some((camera.position, camera.target - camera.position));

        let ray = camera.ray_through(self.cursor);
        self.aimed = on_the_floor(&ray);
        self.pointing = Some(ray);
        let under = self.run.under(ray.origin, ray.direction);
        if under != self.just_laid {
            self.just_laid = None;
        }
        self.over = under.filter(|which| Some(*which) != self.just_laid);
    }

    fn process_keyboard(&mut self, input: KeyboardInput) {
        let held = input.state == KeyboardKeyState::Pressed;
        match input.key {
            KeyboardKey::Space if held => match self.run.phase() {
                Phase::Laying => self.run.push(),
                // While it is still going as well, not only once it is over,
                // so a run nobody wants to watch out does not have to be.
                Phase::Over | Phase::Falling => self.run = Run::new(),
            },
            KeyboardKey::Left if held => self.aim += TURN,
            KeyboardKey::Right if held => self.aim -= TURN,
            KeyboardKey::Escape => self.quitting = held,
            _ => (),
        }
    }

    fn process_mouse(&mut self, input: MouseInput) {
        match input.button {
            MouseButton::Right => self.turning = input.is_pressed(),
            // One button, and whatever is under the cursor decides what it
            // means: a domino gets knocked over, bare floor gets a new one.
            // The cursor was read in `draw`, where the camera is, so this
            // agrees with what is lit up and what the readout says.
            MouseButton::Left if input.is_pressed() => match (self.over, self.pointing) {
                (Some(which), Some(ray)) => {
                    self.run.shove(which, ray.direction);
                }
                _ => {
                    if let Some(at) = self.aimed {
                        if self.run.lay(at, self.aimed_way()) {
                            self.just_laid = Some(self.run.dominoes().len() - 1);
                        }
                    }
                }
            },
            _ => (),
        }
    }

    fn cursor_moved(&mut self, position: Vec2) {
        self.cursor = position;
    }

    fn mouse_motion(&mut self, delta: Vec2) {
        if self.turning {
            self.camera_angle += delta.x * 0.005;
            self.camera_up = (self.camera_up - delta.y * 0.004).clamp(0.12, 1.3);
        }
    }

    fn mouse_wheel(&mut self, delta: Vec2) {
        self.distance = (self.distance - delta.y * 0.05).clamp(4.0, 30.0);
    }

    fn is_quitting(&self) -> bool {
        self.quitting
    }

    fn focus_changed(&mut self, _focus: bool) {}
}

fn main() {
    start("cascada", Box::new(Cascada::new()));
}
