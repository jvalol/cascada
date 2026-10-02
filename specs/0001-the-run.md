# 0001 The run

**Status:** draft
**Date:** 2026-10-02

## Goal

Stand dominoes up on a floor, one at a time, wherever you like. Then push the
first one. The number at the end is how many fell.

## Why this game exists

Blitzkit's spec 0036 built sleeping, and with it a rule that a body touching one
that has woken wakes too, outwards through the contacts. A line of dominoes is
that rule and almost nothing else: a hundred of them standing still, costing
nothing, until one is pushed and the waking runs down the line a little ahead of
the falling.

It is the most watchable thing the engine can do and it exists today only in a
test called `waking_spreads_through_contacts`. Cairn is boxes resting on each
other. This is boxes knocking each other over, which is the same contacts asked a
different question.

It needs nothing new from the engine, which is the other reason to build it.

## Behavior

**A supply of dominoes, and they run out.** A fixed number to lay, so where they
go is a decision rather than a habit. The supply is on screen.

**One click, one domino.** It stands at the point on the floor the cursor names,
turned across the line from the one before it, so a run follows the clicks. The
first one faces whichever way the second click asks for.

**Spacing is the player's and the physics decides what works.** Too close and a
falling domino has nowhere to swing before it meets the next, and the chain
stalls. Too far and it falls short. Nothing in the game enforces a spacing: the
range that works is a thing to find, and the numbers it works over are measured
and written down rather than chosen.

**A domino that will not stand is refused.** One placed inside another cannot be
put there, and saying so is better than dropping it in and watching the pair
explode.

**Nothing moves until it is pushed.** Dominoes are laid, they settle, they fall
asleep. Pushing is a separate thing the player does once, to the first one, and
from then on it is out of their hands.

**The count is how many fell**, not how many were laid. A domino counts as fallen
when it is further over than it could stand.

**How many are awake is on screen.** It is the wave made visible: the number
climbs as the fall spreads and drops back to nothing as the far end settles, and
it is also the thing the engine is being shown off for.

## What it asks of blitzkit

- Blocks, orientation and the inertia tensor, per spec 0034. A domino falling is
  a block turning about the edge it stands on, and the tensor is why it looks
  like a domino rather than a cube.
- Box against box and box against the world, per spec 0035.
- A `Solver` kept across frames, per spec 0036, for the sleeping. Without it a
  hundred standing dominoes are a hundred bodies being solved forever.
- `Camera::ray_through`, per spec 0025, to find where on the floor a click
  lands.
- `Body::strike` and `Body::wake`, per spec 0032 and 0036, for the push.
- Spatial sound, per spec 0019, and a knock of cairn's shape.

## What it will not have

No undo. No saving a run. No branching the chain by hand, though a run that
crosses itself will do what it does. No dominoes of different sizes. No table
edge to fall off that is not the floor's own.

## The numbers are measured, not reasoned

The three that matter are all the physics', not mine, and none of them is settled
until it has been run:

- How far apart two dominoes may stand and still knock each other over. There is
  a near end where the fall jams and a far end where it falls short.
- How hard the first one has to be pushed, and whether a push that is too hard
  throws it over the second rather than into it.
- Whether a hundred of them standing is still the quiet nothing spec 0036
  promised, or whether a hundred bodies in one line is where that stops being
  true.

## Acceptance criteria

- A new run has a full supply and nothing on the floor. — `run::tests::a_new_run_is_empty`
- Laying one takes it off the supply. — `run::tests::laying_one_spends_it`
- A domino may not be laid inside another. — `run::tests::they_do_not_overlap`
- Nor when the supply is gone. — `run::tests::an_empty_supply_lays_nothing`
- Each one stands across the line from the one before. — `run::tests::they_stand_across_the_line`
- A laid run settles and goes to sleep. — `run::tests::a_laid_run_sleeps`
- Pushing the first one wakes the second before it has touched it. — `run::tests::the_waking_runs_ahead`
- A run at a spacing that works knocks all of them down. — `run::tests::a_good_spacing_carries`
- One too far apart does not. — `run::tests::too_far_apart_falls_short`
- The count is of those that fell, not those that were laid. — `run::tests::the_count_is_what_fell`
- Nothing falls until it is pushed. — `run::tests::nothing_falls_on_its_own`

### Verified by hand

- A line of them goes over like a line of dominoes.
- The number awake climbs with the wave and comes back to nothing.
- A curve works, and a tight enough curve does not.

## Out of scope

Dominoes with spots. Standing one on top of another. Ramps, jumps, or anything
that is not a domino. A second player. A target to hit. Scoring beyond the
count.
