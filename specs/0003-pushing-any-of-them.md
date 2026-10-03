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
It is struck near the top, and harder than the figure's own start.

Harder because the two are doing different jobs. A start only has to put the
first one past its balance, and the next one along is square on and tips over
its own thickness. A click is a finger, and what it is pushed into may be
anything: a domino standing across the line is hit on its narrow end and has to
rock over a base two and a half times wider. Measured, a domino pushed into the
side of another at three spacings:

```text
2.2 to 4.0   the one across stands, every spacing
5.0 to 8.0   it goes over, every spacing
```

A straight run goes twenty of twenty at all of those, so nothing is given up by
the harder click. This spec said they were the same shove, and that is what left
a clicked tile leaning against its neighbour with the wave stopped.

**It falls away from the click.** A domino can only go two ways, along its thin
axis, and which one is decided by where the click came from. The ray is
flattened onto the floor and taken against the domino's thin axis, so a click
from the near side sends it away and a click from behind sends it back.

**Clicking the floor still lays one.** A click that meets nothing standing is a
click on the floor, which is what spec 0001 made it.

**And one just laid is not pushable until the cursor has left it.** Laying a
domino puts it under the cursor, so with one button and the thing underneath
deciding, the next click read as a push and there was no obvious way to lay a
second. Clicking again lays the next one; move off it and back and it is a
domino like any other.

**At any point in a run.** While laying, while a wave is still running, and after
everything has stopped. A run that had come to rest and is poked again is
running again, and what falls afterwards counts with the rest.

**And a shove that takes nothing with it is not a run.** A domino pushed into
the narrow end of another leans on it and stops, which is what a real one does:
toppling that way means rocking over a base two and a half times wider than the
one it tips over going forwards. The game called that a finished run and printed
"0 of 137 went over" a fifth of a second after the click. If nothing has gone
over since the push, laying carries on where it left off.

**Space still sets the figure off**, unchanged, so the figure keeps a start that
takes no aim.

**Which way a laid domino faces is spec 0005's**, and was briefly this spec's. A
click out on its own was made the head of a new chain, stood square until its
second arrived, because being able to push one made it visible that the player's
first click faced whatever the figure happened to end on. That was still the
game choosing. The player aims it now.

## Acceptance criteria

- Clicking a standing domino knocks it over. — `run::tests::a_click_pushes_what_it_hits`
- And one standing across its path goes over too. — `run::tests::a_click_takes_the_one_across_its_path`
- It falls away from the click rather than towards it. — `run::tests::it_falls_away_from_the_click`
- A domino the player laid can be pushed like any other. — `run::tests::one_the_player_laid_can_be_pushed`
- A click that meets nothing standing lays one instead. — `run::tests::a_click_on_the_floor_still_lays`
- A run that has come to rest can be set going again. — `run::tests::a_finished_run_can_be_poked`
- A shove that takes nothing with it leaves the run laying. — `run::tests::a_shove_that_takes_nothing_is_not_a_run`
- What falls after a second push counts with the rest. — `run::tests::a_later_push_adds_to_the_count`

### Verified by hand

- Picking out one domino of the figure and dropping it inwards rather than along.
- Standing a few off to one side and setting them off without touching the figure.

## Out of scope

Dragging a domino, which is cairn's hand and a different game. Picking one up
again once it is laid. Aiming the push harder or softer. Undo.
