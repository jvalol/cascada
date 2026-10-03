# 0004 Spots on them

**Status:** implemented
**Date:** 2026-10-03

## Goal

A domino has spots on it.

## Why

They are blank ivory slabs. Spec 0002 put spots out of scope and spec 0003 left
them there, and the figure got large enough to look at before anyone noticed
that the thing it is drawn with has no face.

It is also the one thing that says what these are. A white block standing on
edge is a block; the same block with six spots and a line across the middle is a
domino, and a run of them is a domino run rather than a wall falling over.

## Behavior

**Each tile carries two halves and a bar between them**, as a real one does: a
count of nought to six at each end, and a line across the middle separating
them.

**The spots are on the two wide faces and nowhere else.** A tile is thin, and
the four narrow sides of a real one are blank. `MeshData::cube` gives every face
the whole texture, so this takes a mesh of its own: the wide faces take the
drawn tile and the edges take a blank corner of it.

**Both wide faces of a tile carry the same count**, which a real one does not do
and is what the player sees. A tile standing shows one face and a tile lying
shows the other, and a wave passing is the moment both are visible.

**Every tile of a set, and then round again.** A double six set is twenty eight
tiles, the figure lays about a hundred and thirty, so the set repeats. Which
tile a domino wears is its own number, so the same domino wears the same face
every time the figure is laid.

**Drawn rather than loaded**, like cairn's grain and marble's checker, so the
game ships nothing but code.

## Acceptance criteria

- A double six set is twenty eight tiles, each pair of counts appearing once. — `pips::tests::a_set_is_every_pair_once`
- A tile's count is its own number, so it is the same every time. — `pips::tests::a_domino_keeps_its_face`
- Each half carries as many spots as it says. — `pips::tests::a_half_has_the_spots_it_claims`
- The bar runs across the middle and the halves do not cross it. — `pips::tests::the_bar_divides_it`
- The spots sit on the wide faces and the edges are blank. — `pips::tests::only_the_wide_faces_are_drawn_on`
- A tile is drawn the way up it is held, so a face is never upside down against its own bar. — `pips::tests::both_halves_read_from_the_same_side`

### Verified by hand

- A standing run reads as dominoes rather than as blocks.
- The spots are legible at the distance the camera sits at.

## Out of scope

Matching ends, which is the other game dominoes are for. Choosing which tile to
lay. Colour on the spots. Wear, chips, or a wooden back.
