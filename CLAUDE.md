# cascada

Stand dominoes up on a floor, one at a time, wherever you like. Then push the
first one. The number at the end is how many fell.

The twelfth game on blitzkit. Cairn is boxes resting on each other; this is
boxes knocking each other over, which is the same contacts asked a different
question.

## Building and running

```
cargo run --release
```

Inside `blitzkit-project` this builds against the engine checkout rather than
the published crate, because of the `[patch.crates-io]` in
`blitzkit-project/.cargo/config.toml`. Cloned on its own it builds against
whatever is on crates.io, which is what a stranger gets.

```
cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check
```

Or `./check-all` from `blitzkit-project`, which runs that for every crate and
then checks the copy markers, the spec citations and the listings.

## The spec flow

A spec before the code, in `specs/`, numbered in the order written. Each
acceptance criterion names the test that proves it, and `check-specs` fails the
gate when an implemented spec cites a test that does not exist. A draft is not
checked, which is what makes it safe to write one before building it.

## Decisions worth defending

**It exists for blitzkit's spec 0036.** That spec built sleeping, and with it a
rule that a body touching one which has woken wakes too, outwards through the
contacts. A line of dominoes is that rule and almost nothing else, and until
this game it existed only in a test called `waking_spreads_through_contacts`.

**Spacing is the player's and nothing enforces it.** Too close and a falling
domino has nowhere to swing before it meets the next. Too far and it falls
short. The range that works is a thing to find, and the numbers are measured
rather than chosen.

**A `Solver` kept across frames, not the free `step`.** A hundred dominoes
standing still should cost nothing, and that is what sleeping is for. The free
step sleeps nothing.

**No broad phase exists.** Blitzkit tests every pair against every other, which
specs 0030, 0035 and 0036 all say outright. A hundred dominoes is 4,950 pairs a
step. Whether that holds up is one of the things this game is here to find out.

## User-facing text

Every string this game draws carries `[COPY - Jake]` until Jake rewrites it into
his own voice and strikes the marker himself. `check-all` reads `src/` as well
as the readmes and holds the gate while one remains.
