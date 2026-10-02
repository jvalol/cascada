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
        Quat::from_rotation_y(flat.x.atan2(flat.z))
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

    #[test]
    fn it_faces_the_way_it_is_sent() {
        let along_x = standing(Vec3::ZERO, Vec3::X);
        let faces = along_x.orientation * Vec3::Z;

        // its face looks along the run, so the run's direction comes back out
        assert!(faces.dot(Vec3::X).abs() > 0.99, "it faces {}", faces);
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
