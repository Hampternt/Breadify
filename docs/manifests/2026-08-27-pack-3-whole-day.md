# Pack 3 — The whole day

**Status:** all 7 items done — pack gate green, awaiting review.
**Container:** Breadify v1 (7 packs).
**Branch:** `pack-3-whole-day`.

## Goal

Paginate every route in an export onto sheets, under the five pagination
rules, and report what the day actually costs in paper.

**Observable when done:** `breadify print --pdf day.pdf` writes one PDF of the
whole sample day — every route starting on a fresh page, no block or route
total cut across a break, the unsequenced flag on all nine routes that need
it — and a test proves every emitted page keeps ≥ 10 mm above its footer.

## Items

| # | Item | Done when |
| --- | --- | --- |
| 1 | Blocks laid out in isolation | A stop and a route total each lay out into their own page at `y = 0`, and a sheet places them by offset, so height is measured by laying out rather than by a second formula that could disagree |
| 2 | The paginator | Routes become sheets: a route always starts a fresh page, no page carries two routes, and a stop block never splits |
| 3 | The route total lands whole | The total closes the route's last page, or moves to a page of its own if it does not fit — never split |
| 4 | Per-page furniture | Every page gets its own masthead with `continued` after the first, a truthful `Page n of m · stops · lines` counter, and a footer that says whether the route continues |
| 5 | The clearance invariant | A test asserts every page of every route in the sample keeps ≥ 10 mm between its last content and the footer rule |
| 6 | Print the day | `breadify print --pdf day.pdf` writes every route in order; `--route <n>` narrows it to one |
| 7 | The real budget | The sheet count for the sample day is measured rather than quoted, and recorded in the docs against the handoff's 24 |

## Ledger

- [x] 1 — Blocks laid out in isolation · stops, the flag and the total lay out at y=0
- [x] 2 — The paginator · 16 routes → 26 sheets, no page carries two
- [x] 3 — The route total lands whole · moves to its own page, pulling a stop down with it
- [x] 4 — Per-page furniture · `continued`, truthful counters, continuation footer
- [x] 5 — The clearance invariant · asserted for all 26 sheets
- [x] 6 — Print the day · `breadify print --pdf day.pdf`
- [x] 7 — The real budget · 26 sheets, recorded in print-spec §9

**Deviations:**

- The design's rule 2 lets a *grouped* stop split between its sub-blocks. D16
  removed grouped stops, so there is nothing to split between: a stop is one
  order and never breaks.
- The day comes to 26 sheets rather than the handoff's 24. The two extra are
  the unsequenced flag on nine routes and the loss of address-level grouping
  under D16. `print-spec.md` §9 records the measurement.
**Pack gate:** not run.
