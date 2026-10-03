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
use blitzkit::physics::Shape;
use blitzkit::renderer::render_text::{RenderText, TextRenderer};
use blitzkit::renderer::scene::{MeshId, Scene};
use blitzkit::renderer::Renderer;
use blitzkit::sound::SoundSystem;
use blitzkit::{start, Game};
use glam::{vec2, vec3, vec4, Vec2, Vec3};
use run::{Phase, Run};

const FLOOR: f32 = 30.0;

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
    /// Where the camera is and which way it faces, so the ears follow it.
    ears: Option<(Vec3, Vec3)>,
    /// How much sound is already queued and not yet played, in seconds.
    waiting: f32,
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
            tiles: Vec::new(),
            floor_mesh: None,
            run: Run::new(),
            cursor: Vec2::ZERO,
            aimed: None,
            pointing: None,
            over: None,
            ears: None,
            waiting: 0.0,
            camera_angle: 2.5,
            camera_up: 0.55,
            turning: false,
            distance: 18.5,
            quitting: false,
        }
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
        _geometry: &mut Geometry,
        text_renderer: &mut TextRenderer,
        sound_system: &SoundSystem,
    ) {
        self.run.step(dt);
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
            (Some(_), _) => String::from("click this one to push it over"),
            (None, Phase::Laying) if self.run.dominoes().len() < 2 => {
                String::from("click the floor to stand one up")
            }
            (None, Phase::Laying) => match self.aimed.and_then(|at| self.run.gap_to(at)) {
                Some(gap) => format!(
                    "click to lay one, {:.2} of a domino from the last. space pushes the figure",
                    gap
                ),
                None => String::from(
                    "click a tile to push it, the floor to lay one. space pushes the figure",
                ),
            },
            (None, Phase::Falling) => String::from("over it goes. space to set it up again"),
            (None, Phase::Over) => format!(
                "{} of {} went over. click any that are left, or space to start again",
                self.run.fallen(),
                self.run.dominoes().len()
            ),
        };

        text_renderer.reset();
        for (line, text) in vec![
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
                let ghost = domino::standing(at, self.run.facing(at));

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
        self.over = self.run.under(ray.origin, ray.direction);
    }

    fn process_keyboard(&mut self, input: KeyboardInput) {
        let held = input.state == KeyboardKeyState::Pressed;
        match input.key {
            KeyboardKey::Space if held => match self.run.phase() {
                Phase::Laying => self.run.push(),
                // While it is still going as well, not only once it is over:
                // the wave takes twenty seconds to get round a figure this
                // size, and nobody should have to sit through the rest of it.
                Phase::Over | Phase::Falling => self.run = Run::new(),
            },
            KeyboardKey::Escape => self.quitting = held,
            _ => (),
        }
    }

    fn process_mouse(&mut self, input: MouseInput) {
        match input.button {
            MouseButton::Right => self.turning = input.is_pressed(),
            // One button, and whatever is under the cursor decides what it
            // means: a domino gets knocked over, bare floor gets a new one.
            MouseButton::Left if input.is_pressed() => {
                let under = self
                    .pointing
                    .and_then(|ray| self.run.under(ray.origin, ray.direction));

                match (under, self.pointing, self.aimed) {
                    (Some(which), Some(ray), _) => {
                        self.run.shove(which, ray.direction);
                    }
                    (None, _, Some(at)) => {
                        self.run.lay(at);
                    }
                    _ => (),
                }
            }
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
