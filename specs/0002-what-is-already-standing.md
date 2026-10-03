# 0002 What is already standing

**Status:** implemented
**Date:** 2026-10-02

## Goal

A shape already laid when the game opens, rather than an empty floor and sixty
clicks.

## Why

Spec 0001 gave the player a supply and let them put it anywhere, which is the
right thing to be able to do and the wrong thing to have to do before anything
happens. Laying a run by hand is a minute of clicking before the first second of
the game, and the thing worth watching is at the end of it.

## Behavior

**A yin and yang, standing, when a run begins.** One push sets all of it off.
Nothing else about the game has to change for that to happen.

**The figure is a tree.** The ring is pushed; the S branches off the ring where
they pass at the top; a spur branches off each half of the S and runs in to that
half's dot. Everything is reached from the one domino that is pushed, and no
path ever rejoins another.

Both halves of that matter, and this spec got both of them wrong first.

**A wave does branch.** The first version of this measured that a domino knocks
over nothing more than fifteen degrees off straight ahead, which is true of a
single domino set off to one side, and read it as a wave not being able to split
at all. That reading made the figure four separate pieces with four pushes,
which is four short runs that happen to begin together rather than one run. A
pair standing abreast is a different question:

```text
branch set 0.26 to 0.40 off the stem's line   both go over whole, at every
                                              divergence tried
0.46                                          one goes, the other is missed
0.55                                          neither
```

So a domino reaches about 0.43 either side of its own line. But a branch is not
a road forking, because one path carrying straight on while the other leaves at
an angle cannot work: to be reached the second would have to stand closer than
that reach, and two dominoes that close are inside each other. What works is
that the branching domino stands **nearer the stem than the stem's own next one
does**:

```text
half a step ahead, 0.25 to 0.45 aside    both paths go over whole
a whole step ahead, same aside           the branch never starts: the path's
                                         own next domino shields it
a step and a quarter ahead               it goes again, taken by that one
                                         instead of by the stem
```

Which is a rule about one domino, so nothing about the path it leaves has to
change. The ring is a plain circle and the S is two plain half circles.

**And no loop can close.** The S meets the ring at the top as a branch and stops
clear of it at the bottom, and that asymmetry is the whole of it. When the S
joined at both ends, the ring filled from both sides and the two sides did not
finish each other off. They leant into each other:

```text
tilt  73.6  73.2  70.9  63.1  47.6  21.5   6.9  40.3  57.8  68.0
moved +.47  +.47  +.46  +.44  +.37  +.18  -.09  -.28  -.43  -.48
```

Each front props the other up and the last few stand as a little arch. That is
what real dominoes do, and no push got rid of it: all twenty seven ways of
starting the three paths, forwards, backwards or not at all, left exactly three.

**A dot is small, and it is reached across a gap rather than wound into.** A spur
that comes in turning alongside the ring it feeds is a spiral, and the ring then
goes the whole way round and back underneath it. At the join the gap between the
two is nothing, by definition, so some stretch of the ring always stands inside
the spur, and no size of figure fixes that because the gap closes to nothing at
the join whatever the scale. So a dot's ring stops where its spur came in rather
than closing. The spur lies across the opening and it still reads closed.

That is only affordable because a dot can be small, which took measuring. This
spec had carried a figure of 36 degrees between neighbours as where a ring jams.
That was measured before a bend had its spacing corrected for the pinch at the
inside of it, and it is no longer true of anything:

```text
0.8, 0.9    nothing goes over
1.0         7 dominoes, 51 degrees apiece, all 7
1.2         9 dominoes, 40 degrees apiece, all 9
1.7        14 dominoes, 26 degrees apiece, all 14
```

Believing the old number had the dots at a third of the outer ring, nothing like
what a yin and yang draws, and left no room between a dot and the S around it
for the spur that feeds it.

**What it looks like from the one push.** The ring carries the time, because it
is one wave going the whole way round, and everything else branches off it and
runs beside it rather than after it.

```text
ring   48 of 48, from 0.8s to 19.0s
S      22 of 22, branching at 6.5s
dot    11 of 11, branching at 9.9s
dot    11 of 11, branching at 14.0s
```

**The spacing is the measured one**, stepped along the path by arc length rather
than by angle: a step of fixed angle is a step of different length on a circle of
a different size, and the dots are a fifth of the radius of the outer ring.

**A curve costs something, and the spacing pays it.** On the inside of a bend
two dominoes stand closer than their centres do, by about half a domino's width
for every radian the path turns through between them. Stepping by the same
amount everywhere therefore pinches every curve and pinches a tight one hardest,
which is the near end of spec 0001's spacing arriving early and without warning.

So a bend is stepped further apart than a straight, by solving
`inner = step · (1 − w / 2r)` for the step that leaves the inner gap where it was
wanted. On the outer ring that is a few percent; on the S, half its radius, a
tenth; on a dot, more again. Before that correction the S stalled after three
dominoes and the figure was not worth looking at.

**The figure is big because a small one is a tight one.** Every bend scales with
the outer radius. At 3.6 across the S turns 21 degrees between neighbours and
stalls; at 5.2 it turns 14 and goes over whole.

**The supply is what is left.** The pattern is laid out of the same supply the
player draws on, and the count says what remains. More can be added anywhere, as
before.

**Starting again lays it again.** The pattern is the floor's natural state, not a
one-off at launch.

## Acceptance criteria

- A new run has the pattern standing on it, with some of the supply left over. — `run::tests::a_new_run_is_already_laid`
- The pattern is a ring, an S and two dots. — `pattern::tests::it_is_a_loop_an_ess_and_two_dots`
- Every step on it is one a domino carries over, bends and spurs included. — `pattern::tests::every_step_is_one_that_carries`
- The S leaves the ring as a branch. — `pattern::tests::the_ess_branches_off_the_ring`
- Each spur leaves the S as a branch. — `pattern::tests::each_spur_branches_off_the_ess`
- The S does not rejoin the ring, so the figure holds no loop. — `pattern::tests::the_ess_does_not_rejoin_the_ring`
- Nothing stands near anything it does not branch from. — `pattern::tests::nothing_stands_near_anything_it_does_not_branch_from`
- The step round a bend leaves the gap it meant to. — `pattern::tests::the_step_round_a_bend_leaves_the_gap_it_meant_to`
- Nothing laid stands inside anything else. — `run::tests::nothing_laid_overlaps`
- The figure comes to rest and the run ends. — `run::tests::the_figure_comes_to_rest`
- One push sends the whole figure over, not most of it. — `run::tests::the_whole_figure_goes_over`
- Starting again lays it again. — `run::tests::starting_again_lays_it_again`

### Verified by hand

- It reads as a yin and yang from above.
- The wave spreads: the ring first, then the S off it, then a dot off each half.
- Nothing is left standing when it stops.

## What it cost, measured

Ninety two dominoes laid out of a hundred and thirty, all of them go over from
one push, and it takes twenty seconds to come to rest, nearly all of it the wave
still running.

**A run ends by its own measure of stillness, which it needed and now barely
does.** The figure stops moving at 19.6 seconds and the run calls it over at
20.5.

Sleeping in the engine is a property of a whole group of touching bodies:
nothing sleeps while anything it touches is moving, so a fallen figure is one
connected group of ninety and the group's clock restarts whenever any one of them
stirs over the threshold. At blitzkit's old eight solver passes it never slept at
all. Long after the figure had stopped, 75 of the 92 were still crossing that
threshold, so the clock never reached half a second and 82 of them were awake at
forty seconds with nothing moving faster than 0.02.

That turned out to be the solver and not the bookkeeping, and it was the same
fault that left a pile of fifty four boxes moving for 58 seconds. Blitzkit's
passes are thirty two now. None of the 92 stir, and it sleeps at 20.2 seconds,
three tenths before this measure calls it over. So the measure is near enough
redundant, and it is kept because it does not depend on what the engine's
defaults happen to be.

**And what is on screen is how many are moving, not how many are awake.** Awake
is the engine's word for a body it has not finished with, and a number that
lingers after the wave has passed is not the wave.

## Out of scope

Other shapes. A shape the player chooses. Spots on the dominoes. Colouring the
two halves differently, which a yin and yang would want and which this engine
would do by tinting and which is spec 0003 if it is worth having.
