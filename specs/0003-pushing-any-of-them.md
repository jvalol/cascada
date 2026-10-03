# 0003 Pushing any of them

**Status:** implemented
**Date:** 2026-10-03

## Goal

Any domino on the floor can be pushed over by clicking it, including the ones
the player stood up themselves, and at any point in a run.

## Why

Spec 0001 gave one push, on the first domino laid, and spec 0002 moved it to the
figure's own start. Both are a single button that sets off a single thing. The
supply left over after the figure is laid is 38 dominoes, and the only thing a
player can currently do with them is add to a line and hope the wave reaches it.
Anything they build on their own is unreachable: there is no way to set it off.

## Behavior

**Clicking a domino pushes it over.** Any of them, the figure's or the player's.
It is struck near the top and with the same shove the figure's own start gets.

**It falls away from the click.** A domino can only go two ways, along its thin
axis, and which one is decided by where the click came from. The ray is
flattened onto the floor and taken against the domino's thin axis, so a click
from the near side sends it away and a click from behind sends it back.

**Clicking the floor still lays one.** A click that meets nothing standing is a
click on the floor, which is what spec 0001 made it.

**At any point in a run.** While laying, while a wave is still running, and after
everything has stopped. A run that had come to rest and is poked again is
running again, and what falls afterwards counts with the rest.

**Space still sets the figure off**, unchanged, so the figure keeps a start that
takes no aim.

**And a click out on its own starts a line of its own.** Spec 0001 stood the very
first domino square and turned it to face the second when that arrived, because
the first has nothing to face. That was enough while the floor began empty.
Since spec 0002 lays a figure first, the player's first click turns to face
whatever the figure happened to end on, which is a direction nothing meant, and
being able to push it is what made that visible: the domino goes over across the
line the player was drawing rather than along it.

So the rule is generalised from "the run is empty" to "this one is not
continuing anything". A click further from the last than any spacing carries is
the head of a new chain, stood square until its second arrives.

## Acceptance criteria

- Clicking a standing domino knocks it over. — `run::tests::a_click_pushes_what_it_hits`
- It falls away from the click rather than towards it. — `run::tests::it_falls_away_from_the_click`
- A domino the player laid can be pushed like any other. — `run::tests::one_the_player_laid_can_be_pushed`
- A click that meets nothing standing lays one instead. — `run::tests::a_click_on_the_floor_still_lays`
- A run that has come to rest can be set going again. — `run::tests::a_finished_run_can_be_poked`
- A click out on its own starts a line of its own rather than facing the last thing laid. — `run::tests::they_stand_across_the_line`
- What falls after a second push counts with the rest. — `run::tests::a_later_push_adds_to_the_count`

### Verified by hand

- Picking out one domino of the figure and dropping it inwards rather than along.
- Standing a few off to one side and setting them off without touching the figure.

## Out of scope

Dragging a domino, which is cairn's hand and a different game. Picking one up
again once it is laid. Aiming the push harder or softer. Undo.
