# 0006 A picture of a run going over

**Status:** implemented
**Date:** 2026-11-02

## Goal

The frame this game shows the world is held against a picture of what it
should be, so that a change which breaks it fails a test rather than waiting
for somebody to look at the readme.

## Why

`--screenshot` has staged a run two seconds into its wave since the readme
needed a picture, because this game's opening frame is a figure standing
still, which is every run before anything has happened. That staging is
already the one thing here worth photographing, and nothing has ever checked
that it still comes out.

The engine's spec 0076 made the check one call. Before it, taking a picture
meant assembling an offscreen renderer, an empty overlay, a `block_on` and an
`Err` turned into a panic, which is why fourteen games were built on that
engine and one of them had pictures.

## Behavior

**The picture is the staged run and not a pose of its own.** A game built
with the staging on, loaded, updated once with the clock at nought and drawn:
the same state `refresh-screenshots` photographs. So what this guards is the
picture already on show rather than a second arrangement that could drift
from it.

**It is whatever size the engine says**, `blitzkit::picture::SIZE`, because
this game has no reason of its own. It was given twice that at first on the
argument that a domino is about five pixels at the engine's size and its face
under one, so every pip spec 0004 put on them could come off without the
comparison moving. Measured rather than assumed, by taking the faces off and
asking how far the picture went: 7.7% at the engine's size against 7.3% at
twice it, where the tolerance is 0.2%. A face that is gone takes its domino's
whole shading with it, which is not a detail at any size, and the bigger
picture cost four times the bytes to notice slightly less.

**The reference lives in `tests/pictures/` and is looked at before it is
committed**, which is the engine's rule and is one no code can enforce.

**A machine with no adapter checks nothing and says so.** The engine writes
which picture went unchecked and hands back that it skipped, and nothing here
turns that into a failure.

## Acceptance criteria

- A run two seconds into its wave matches the picture of
  one. — `tests::a_run_going_over_looks_right`

### Verified by hand

- **The control**, which is the same shot with the staging off. It differs by
  4.1% of its pixels against a tolerance of 0.2%, so the reference is of the
  staged state rather than of anything at all. Set `Cascada::new(false)` in
  the test to see it.
- **What a lost face costs**, which is what the size was settled on: clear
  `tiles` after `load` and the picture moves 7.7%.
- **The reference itself**, looked at before it was committed: a yin and yang
  of dominoes with the near arc going over, which is what the readme's own
  screenshot shows.

## Out of scope

**Failing a run that drew nothing.** A machine with no adapter skips, and
deciding that a run with no pictures in it is a failed run belongs to
whatever runs the gate rather than to this game.

**A picture per state.** One frame, and the one already staged. Pictures of
laying, aiming and pushing would each need a pose invented for them, and an
invented pose is a second account of what this game looks like.
