# 0005 Laying it your way

**Status:** implemented
**Date:** 2026-10-03

## Goal

A domino goes down facing where you aimed it, stays that way, and can be laid
at any point in a run.

## Why

Three complaints, and all three are the same mistake: the game decided things
the player should be deciding.

Spec 0001 worked out which way a domino faced from the one before it, because a
run was a line being drawn and that is what a line wants. Spec 0003 added a rule
for a click out on its own, and the two together mean a domino's direction is
never the player's: it is inferred, and when the second of a pair arrives the
first is turned to face it. A domino already on the floor moves by itself.

And laying stopped the moment anything was pushed. Spec 0003 made pushing
something you can do at any point in a run; laying was left where spec 0001 put
it, before the first push and never again.

## Behavior

**A domino faces where it is aimed.** The left and right arrows turn the ghost
under the cursor, and the domino goes down at exactly that heading. It starts
pointing away from the camera, which is where a first domino wants to fall.

**Nothing already laid is ever turned.** Spec 0001 stood the first one square
and turned it when the second arrived, and spec 0003 generalised that to the
head of any chain. Both are gone. What is on the floor stays as it was put
there.

**Laying works at any point in a run.** While the figure stands, while a wave is
running, and after everything has stopped. The supply is the only limit, and it
is the same supply.

**The ghost shows the heading**, so the aim is visible before the click rather
than after.

## Acceptance criteria

- A domino is laid at the heading it was given. — `run::tests::it_is_laid_facing_where_it_was_aimed`
- Laying another does not turn the first. — `run::tests::laying_one_never_turns_another`
- A domino can be laid while a wave is running and after it has stopped. — `run::tests::laying_works_at_any_point`
- The supply is still the only limit on laying. — `run::tests::an_empty_supply_lays_nothing`

### Verified by hand

- Turning the ghost with the arrows and watching it swing under the cursor.
- Laying a curve by turning between clicks.

## Out of scope

Dragging to aim, which is cairn's hand. Picking a domino up once laid. Turning
one that is already down. Snapping the aim to a neighbour.
