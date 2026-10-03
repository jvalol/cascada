//! What a domino is, and where one stands. Spec 0001.

use blitzkit::physics::Body;
use glam::{vec3, Quat, Vec3};

/// Half a domino. A real one is about nine by twenty five by fifty, which is
/// 1 : 2.8 : 5.6, and this is near enough that one standing looks like one.
pub const HALF: Vec3 = vec3(0.09, 0.50, 0.25);

/// How tall it stands, which is the number every spacing is measured against.
pub const TALL: f32 = HALF.y * 2.0;

/// How bouncy, which is not at all, and how much they grip.
pub const BOUNCE: f32 = 0.0;
pub const GRIP: f32 = 0.5;

/// One standing on the floor at a point, turned so its face looks along `way`.
///
/// A domino falls about the edge it stands on, so the way it faces is the way
/// it goes: laid across the line of the run, it topples along it.
pub fn standing(at: Vec3, way: Vec3) -> Body {
    let flat = vec3(way.x, 0.0, way.z);
    let facing = if flat.length_squared() < 1e-6 {
        Quat::IDENTITY
    } else {
        // Its thin way along the run, not its wide way. A domino in a line
        // shows its broad faces to the ones in front of and behind it, and
        // falls about the edge it stands on, which is across the run. Turned
        // the other way it would topple sideways out of its own line.
        Quat::from_rotation_y((-flat.z).atan2(flat.x))
    };

    Body::block(vec3(at.x, HALF.y, at.z), HALF, 1.0)
        .facing(facing)
        .with_restitution(BOUNCE)
        .with_friction(GRIP)
}

/// Whether one has fallen: further over than it could stand.
pub fn has_fallen(body: &Body) -> bool {
    (body.orientation * Vec3::Y).dot(Vec3::Y) < 0.7
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_stands_on_the_floor() {
        let it = standing(vec3(3.0, 0.0, -2.0), Vec3::X);

        assert_eq!(it.position, vec3(3.0, HALF.y, -2.0));
        assert!(!has_fallen(&it), "it was laid down flat");
    }

    /// Its thin way runs along the line and its wide way across it, which is
    /// what makes a line of them a line of dominoes rather than a wall.
    #[test]
    fn it_stands_thin_way_along_the_run() {
        for way in [Vec3::X, Vec3::Z, vec3(1.0, 0.0, 1.0), vec3(-2.0, 0.0, 0.7)] {
            let it = standing(Vec3::ZERO, way);
            let flat = vec3(way.x, 0.0, way.z).normalize();

            let thin = it.orientation * Vec3::X;
            let wide = it.orientation * Vec3::Z;

            assert!(
                thin.dot(flat) > 0.99,
                "its thin way points {} and the run goes {}",
                thin,
                flat
            );
            assert!(wide.dot(flat).abs() < 0.01, "its wide way is along the run");
            assert!(wide.y.abs() < 0.01, "it is leaning");
        }
    }

    /// Read off a body rather than off the constants, which clippy is right to
    /// say proves nothing: an assertion between two consts is decided at
    /// compile time and holds whatever the game then builds.
    #[test]
    fn a_domino_is_taller_than_it_is_thick() {
        let blitzkit::physics::Shape::Block { half } = standing(Vec3::ZERO, Vec3::X).shape else {
            unreachable!("a domino is a block")
        };

        assert!(half.y > half.z, "it is wider than it is tall: {:?}", half);
        assert!(half.z > half.x, "it is thicker than it is wide: {:?}", half);
        assert!(
            half.y * 2.0 > half.x * 8.0,
            "it is too stout to topple: {:?}",
            half
        );
    }
}
