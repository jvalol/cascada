# 0001 The run

**Status:** implemented
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
range that works is a thing to find.

**But the gap is said out loud**, as a fraction of a domino's height, before the
click that would make it. That is a fact about what the cursor is pointing at
rather than advice about it, and without it a run that does nothing is a run with
no way of knowing which end of the range it fell off. Played a few times, the
range teaches itself.

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

## The numbers, measured

Twenty dominoes in a line, pushed, and how many went over.

**Spacing**, as a fraction of their own height:

```text
0.25, 0.35, 0.50   none of them. the fall jams: there is nowhere to swing
0.60, 0.70         all twenty
0.80               eighteen
0.90               seventeen
1.00 and past      one, which is the one that was pushed
```

So it carries from about six tenths to nine tenths of their height, and all
twenty only in the lower half of that. Both ends the spec guessed at are real
and the near one is sharper than expected: half their height is already too
close.

**The push**, at six tenths:

```text
0.10, 0.20   nothing moves
0.35 to 2.0  all twenty
5.00         one
```

The top end is the thing worth having asked about. Shoved hard enough the first
one leaves over the top of the second rather than into it and nothing else
moves, which looks exactly like a push that was too soft. It is set at half, in
the middle of what works.

**The cost**, against a budget of 8.33 milliseconds a frame:

```text
20 standing    0.01 ms a step
60 standing    0.06 ms
100 standing   0.06 ms
```

Which answers the third question: a hundred bodies with no broad phase and 4,950
pairs to test is nothing at all, because they are asleep. Spec 0036 earns its
keep here more plainly than it does in cairn.

## The numbers were measured, not reasoned

The three that mattered were all the physics', not mine, and all three are
above. The one the spec did not think to ask is that a domino stands with its
thin way along the run and its wide way across. Turned the other way it topples
sideways out of its own line, and the scaffolding this repo was born with had
it wrong.

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
- A push too hard goes over the next one rather than into it. — `run::tests::a_push_too_hard_goes_over_the_next`
- The gap is said in dominoes, and it is the plain distance. — `run::tests::the_gap_is_said_in_dominoes`
- A domino stands thin way along the run. — `domino::tests::it_stands_thin_way_along_the_run`

### Verified by hand

- A line of them goes over like a line of dominoes.
- The number awake climbs with the wave and comes back to nothing.
- A curve works, and a tight enough curve does not.

## Out of scope

Dominoes with spots. Standing one on top of another. Ramps, jumps, or anything
that is not a domino. A second player. A target to hit. Scoring beyond the
count.
