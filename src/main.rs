//! cascada: dominoes on a floor. See `specs/`.
//!
//! This is the scaffolding the repo was born with: a line of them standing,
//! settled and asleep, and a camera to walk round it with. Laying them and
//! pushing them over is spec 0001 and is not built yet.

mod domino;

use blitzkit::camera::Camera;
use blitzkit::collision::Aabb;
use blitzkit::geometry::Geometry;
use blitzkit::keyboard::{KeyboardInput, KeyboardKey, KeyboardKeyState};
use blitzkit::mesh::{MeshData, Transform};
use blitzkit::mouse::{MouseButton, MouseInput};
use blitzkit::physics::{Body, Shape, Solver};
use blitzkit::renderer::render_text::{RenderText, TextRenderer};
use blitzkit::renderer::scene::{MeshId, Scene};
use blitzkit::renderer::Renderer;
use blitzkit::sound::SoundSystem;
use blitzkit::{start, Game};
use glam::{vec2, vec3, vec4, Vec2, Vec3};

const FLOOR: f32 = 30.0;
const GRAVITY: f32 = -9.81;

/// What the scaffolding stands up: a short line, at a spacing that looks like
/// one. What actually works is spec 0001's to measure.
const SHOWN: usize = 14;
const APART: f32 = domino::TALL * 0.6;

fn laid() -> Vec<Body> {
    (0..SHOWN)
        .map(|n| {
            let along = (n as f32 - (SHOWN as f32 - 1.0) * 0.5) * APART;

            domino::standing(vec3(along, 0.0, 0.0), Vec3::X)
        })
        .collect()
}

struct Cascada {
    domino_mesh: Option<MeshId>,
    floor_mesh: Option<MeshId>,
    dominoes: Vec<Body>,
    ground: Vec<Aabb>,
    /// Kept across frames. A line of dominoes standing still should cost
    /// nothing, and that is what blitzkit's sleeping is for; the free `step`
    /// sleeps nothing.
    solver: Solver,
    camera_angle: f32,
    camera_up: f32,
    turning: bool,
    distance: f32,
    quitting: bool,
}

impl Cascada {
    fn new() -> Self {
        Self {
            domino_mesh: None,
            floor_mesh: None,
            dominoes: laid(),
            ground: vec![Aabb::from_center_size(
                vec3(0.0, -1.0, 0.0),
                vec3(FLOOR, 2.0, FLOOR),
            )],
            solver: Solver::new(),
            camera_angle: 2.5,
            camera_up: 0.45,
            turning: false,
            distance: 9.0,
            quitting: false,
        }
    }
}

impl Game for Cascada {
    fn load(&mut self, renderer: &mut Renderer) {
        self.domino_mesh = Some(renderer.add_mesh(&MeshData::cube()));
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
        _geometry: &mut Geometry,
        text_renderer: &mut TextRenderer,
        _sound_system: &SoundSystem,
    ) {
        self.solver.step(
            &mut self.dominoes,
            &self.ground,
            vec3(0.0, GRAVITY, 0.0),
            dt,
        );

        let awake = self.dominoes.iter().filter(|one| !one.asleep).count();
        let fallen = self
            .dominoes
            .iter()
            .filter(|one| domino::has_fallen(one))
            .count();

        text_renderer.reset();
        for (line, text) in vec![
            format!(
                "[COPY - Jake] {} standing, {} fallen, {} awake",
                self.dominoes.len() - fallen,
                fallen,
                awake
            ),
            String::from("[COPY - Jake] laying them and pushing them over is spec 0001"),
            String::from("[COPY - Jake] right-drag turns and tilts, scroll zooms"),
        ]
        .into_iter()
        .enumerate()
        {
            text_renderer.push_render_text(RenderText {
                position: vec2(20.0, 20.0 + line as f32 * 24.0),
                text,
                size: 14.0,
                ..Default::default()
            });
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

        for body in self.dominoes.iter() {
            let Shape::Block { half } = body.shape else {
                continue;
            };

            scene.push_material(
                domino_mesh,
                &Transform::at(body.position)
                    .with_rotation(body.orientation)
                    .with_scale(half * 2.0),
                vec4(0.92, 0.90, 0.86, 1.0),
                64.0,
            );
        }

        camera.target = vec3(0.0, domino::TALL, 0.0);
        camera.position = camera.target
            + vec3(
                self.camera_angle.sin() * self.camera_up.cos() * self.distance,
                self.camera_up.sin() * self.distance,
                self.camera_angle.cos() * self.camera_up.cos() * self.distance,
            );
    }

    fn process_keyboard(&mut self, input: KeyboardInput) {
        let held = input.state == KeyboardKeyState::Pressed;
        match input.key {
            KeyboardKey::Space if held => {
                self.dominoes = laid();
                self.solver.forget();
            }
            KeyboardKey::Escape => self.quitting = held,
            _ => (),
        }
    }

    fn process_mouse(&mut self, input: MouseInput) {
        if input.button == MouseButton::Right {
            self.turning = input.is_pressed();
        }
    }

    fn mouse_motion(&mut self, delta: Vec2) {
        if self.turning {
            self.camera_angle += delta.x * 0.005;
            self.camera_up = (self.camera_up - delta.y * 0.004).clamp(0.08, 1.2);
        }
    }

    fn mouse_wheel(&mut self, delta: Vec2) {
        self.distance = (self.distance - delta.y * 0.03).clamp(3.0, 24.0);
    }

    fn is_quitting(&self) -> bool {
        self.quitting
    }

    fn focus_changed(&mut self, _focus: bool) {}
}

fn main() {
    start("cascada", Box::new(Cascada::new()));
}
