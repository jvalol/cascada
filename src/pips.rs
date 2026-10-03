//! The face of a tile: two counts, a bar between them, and the mesh that keeps
//! all of it off the edges. Spec 0004.
//!
//! Drawn rather than loaded, like cairn's grain and marble's checker, so the
//! game ships nothing but code.

use blitzkit::mesh::{MeshData, Vertex};
use blitzkit::texture::TextureData;

/// How big a drawn tile is. Twice as tall as it is wide, because the face of a
/// domino is, and the two halves are then square.
pub const WIDE: u32 = 128;
pub const TALL: u32 = 256;

/// A strip of plain ivory under the face, for the four narrow sides to wear.
/// `MeshData::cube` hands every face the whole texture, so without somewhere
/// blank to point them the edges of a tile come out wearing stretched spots.
pub const BLANK: u32 = 16;

/// The highest count on a half, so a set is every pair from nought to this.
pub const MOST: u8 = 6;

/// The ivory of the tile, the near black of a spot, and the bar between the
/// halves. Whole bytes rather than fractions, because that is what goes into a
/// texture.
const IVORY: [u8; 3] = [235, 231, 219];
const SPOT: [u8; 3] = [38, 34, 30];
const BAR: [u8; 3] = [168, 162, 148];

/// How big a spot is and how far the face is inset, as a fraction of a half.
const SPOT_ACROSS: f32 = 0.15;
const EDGE: f32 = 0.22;

/// Every tile of a double six set, in the order they are counted out.
pub fn set() -> Vec<(u8, u8)> {
    let mut tiles = Vec::new();
    for high in 0..=MOST {
        for low in 0..=high {
            tiles.push((high, low));
        }
    }

    tiles
}

/// Which tile a domino wears. Its own number, so the same domino wears the same
/// face every time the figure is laid.
pub fn worn_by(domino: usize) -> usize {
    domino % set().len()
}

/// Where the spots of a count sit inside a half, as fractions of it.
///
/// The arrangement a real tile uses: the odd counts carry a middle spot and the
/// even ones do not, and everything else is the corners and the sides.
pub fn spots(count: u8) -> Vec<(f32, f32)> {
    let (near, far, mid) = (EDGE, 1.0 - EDGE, 0.5);
    let corners = [(near, near), (far, far), (far, near), (near, far)];
    let sides = [(near, mid), (far, mid)];

    let mut out = Vec::new();
    match count {
        0 => {}
        1 => out.push((mid, mid)),
        2 => out.extend([corners[0], corners[1]]),
        3 => out.extend([corners[0], (mid, mid), corners[1]]),
        4 => out.extend(corners),
        5 => {
            out.extend(corners);
            out.push((mid, mid));
        }
        _ => {
            out.extend(corners);
            out.extend(sides);
        }
    }

    out
}

/// A tile drawn as pixels: the high half at the top, the low at the bottom, the
/// bar across the middle, and the blank strip under both.
pub fn face(high: u8, low: u8) -> Vec<u8> {
    let mut pixels = vec![0u8; (WIDE * (TALL + BLANK) * 4) as usize];
    let radius = SPOT_ACROSS * WIDE as f32;
    let half = TALL / 2;

    for y in 0..TALL + BLANK {
        for x in 0..WIDE {
            // which half this pixel is in, and where in that half
            let colour = if y >= TALL {
                IVORY
            } else if y.abs_diff(half) < 2 {
                BAR
            } else {
                let count = if y < half { high } else { low };
                let down = if y < half { y } else { y - half };
                let at = (x as f32 + 0.5, down as f32 + 0.5);

                let inked = spots(count).into_iter().any(|(u, v)| {
                    let middle = (u * WIDE as f32, v * half as f32);
                    (at.0 - middle.0).hypot(at.1 - middle.1) < radius
                });

                if inked {
                    SPOT
                } else {
                    IVORY
                }
            };

            let at = ((y * WIDE + x) * 4) as usize;
            pixels[at..at + 3].copy_from_slice(&colour);
            pixels[at + 3] = 255;
        }
    }

    pixels
}

/// Every tile of the set, ready for the renderer.
pub fn tiles() -> Vec<TextureData> {
    set()
        .into_iter()
        .map(|(high, low)| TextureData::from_pixels(WIDE, TALL + BLANK, face(high, low)))
        .collect()
}

/// A cube that wears the drawn tile on its two wide faces and the blank strip
/// on the four narrow ones.
///
/// `MeshData::cube` gives every face the whole texture, which on a tile means
/// spots smeared down the edges. A domino is thin along x, so the faces at
/// x = ±1/2 are the ones with a face on them.
pub fn tile_mesh() -> MeshData {
    // where the drawn face ends and the blank strip begins
    let drawn = TALL as f32 / (TALL + BLANK) as f32;
    let blank = [
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, drawn + 0.001],
        [0.0, drawn + 0.001],
    ];
    let shown = [[0.0, drawn], [1.0, drawn], [1.0, 0.0], [0.0, 0.0]];

    let faces: [([f32; 3], [[f32; 3]; 4], bool); 6] = [
        (
            [1.0, 0.0, 0.0],
            [
                [0.5, -0.5, 0.5],
                [0.5, -0.5, -0.5],
                [0.5, 0.5, -0.5],
                [0.5, 0.5, 0.5],
            ],
            true,
        ),
        (
            [-1.0, 0.0, 0.0],
            [
                [-0.5, -0.5, -0.5],
                [-0.5, -0.5, 0.5],
                [-0.5, 0.5, 0.5],
                [-0.5, 0.5, -0.5],
            ],
            true,
        ),
        (
            [0.0, 1.0, 0.0],
            [
                [-0.5, 0.5, 0.5],
                [0.5, 0.5, 0.5],
                [0.5, 0.5, -0.5],
                [-0.5, 0.5, -0.5],
            ],
            false,
        ),
        (
            [0.0, -1.0, 0.0],
            [
                [-0.5, -0.5, -0.5],
                [0.5, -0.5, -0.5],
                [0.5, -0.5, 0.5],
                [-0.5, -0.5, 0.5],
            ],
            false,
        ),
        (
            [0.0, 0.0, 1.0],
            [
                [-0.5, -0.5, 0.5],
                [0.5, -0.5, 0.5],
                [0.5, 0.5, 0.5],
                [-0.5, 0.5, 0.5],
            ],
            false,
        ),
        (
            [0.0, 0.0, -1.0],
            [
                [0.5, -0.5, -0.5],
                [-0.5, -0.5, -0.5],
                [-0.5, 0.5, -0.5],
                [0.5, 0.5, -0.5],
            ],
            false,
        ),
    ];

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for (normal, corners, wide) in faces {
        let began = vertices.len() as u32;
        let uvs = if wide { shown } else { blank };
        for (corner, uv) in corners.iter().zip(uvs) {
            vertices.push(Vertex::new(*corner, normal, uv));
        }
        indices.extend([began, began + 1, began + 2, began, began + 2, began + 3]);
    }

    MeshData::new(vertices, indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What a pixel is, so a test can say "a spot was drawn here".
    fn at(pixels: &[u8], x: u32, y: u32) -> [u8; 3] {
        let n = ((y * WIDE + x) * 4) as usize;
        [pixels[n], pixels[n + 1], pixels[n + 2]]
    }

    /// Spec 0004: a double six set is every pair of counts once.
    #[test]
    fn a_set_is_every_pair_once() {
        let tiles = set();

        assert_eq!(tiles.len(), 28, "a double six set is twenty eight");
        for (high, low) in &tiles {
            assert!(
                high >= low,
                "({}, {}) is the same tile the other way up",
                high,
                low
            );
            assert!(*high <= MOST);
            assert_eq!(
                tiles.iter().filter(|pair| *pair == &(*high, *low)).count(),
                1,
                "({}, {}) is in the set twice",
                high,
                low
            );
        }
    }

    /// Spec 0004: and a domino keeps the one it was dealt.
    #[test]
    fn a_domino_keeps_its_face() {
        for which in [0usize, 1, 27, 28, 135] {
            assert_eq!(worn_by(which), worn_by(which));
            assert!(worn_by(which) < set().len());
        }

        // the set repeats rather than running out
        assert_eq!(worn_by(0), worn_by(28));
        assert_ne!(worn_by(0), worn_by(1));
    }

    /// Spec 0004: each half carries as many spots as it says.
    #[test]
    fn a_half_has_the_spots_it_claims() {
        for count in 0..=MOST {
            assert_eq!(
                spots(count).len(),
                count as usize,
                "a half of {} drew {} spots",
                count,
                spots(count).len()
            );

            // and none of them sit on top of another
            let places = spots(count);
            for (n, one) in places.iter().enumerate() {
                for other in &places[n + 1..] {
                    let apart = (one.0 - other.0).hypot(one.1 - other.1);
                    assert!(
                        apart > SPOT_ACROSS,
                        "two spots of {} are {} apart",
                        count,
                        apart
                    );
                }
            }
        }
    }

    /// Spec 0004: the bar runs across the middle and neither half crosses it.
    #[test]
    fn the_bar_divides_it() {
        let pixels = face(MOST, MOST);
        let half = TALL / 2;

        for x in 0..WIDE {
            assert_eq!(at(&pixels, x, half), BAR, "the bar has a gap at {}", x);
        }

        // and the spots of the top half stay in the top half
        let highest = (0..half)
            .filter(|y| (0..WIDE).any(|x| at(&pixels, x, *y) == SPOT))
            .max()
            .expect("a six has spots");
        assert!(highest < half - 2, "a spot reaches the bar at {}", highest);
    }

    /// Spec 0004: the spots are on the wide faces and the edges are blank.
    #[test]
    fn only_the_wide_faces_are_drawn_on() {
        let mesh = tile_mesh();
        let drawn = TALL as f32 / (TALL + BLANK) as f32;

        for vertex in &mesh.vertices {
            let wide = vertex.normal[0].abs() > 0.5;
            if wide {
                assert!(
                    vertex.uv[1] <= drawn + 1e-6,
                    "a wide face reaches into the blank strip at {:?}",
                    vertex.uv
                );
            } else {
                assert!(
                    vertex.uv[1] > drawn,
                    "a narrow face reaches onto the drawn tile at {:?}",
                    vertex.uv
                );
            }
        }

        // and the blank strip really is blank
        let pixels = face(MOST, MOST);
        for y in TALL..TALL + BLANK {
            for x in 0..WIDE {
                assert_eq!(
                    at(&pixels, x, y),
                    IVORY,
                    "the strip is marked at {},{}",
                    x,
                    y
                );
            }
        }
    }

    /// Spec 0004: both halves are drawn from the same side, so a tile is never
    /// upside down against its own bar.
    #[test]
    fn both_halves_read_from_the_same_side() {
        // a one and a blank: the single spot has to be in the half that claims
        // it, and the other half has to be empty
        let pixels = face(1, 0);
        let half = TALL / 2;

        let top = (0..half).any(|y| (0..WIDE).any(|x| at(&pixels, x, y) == SPOT));
        let bottom = (half..TALL).any(|y| (0..WIDE).any(|x| at(&pixels, x, y) == SPOT));

        assert!(top, "the high half of a one and a blank has no spot");
        assert!(!bottom, "the low half of a one and a blank has a spot");
    }
}
