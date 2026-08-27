# Pack 5 — Configure

**Status:** all 7 items done, reviewed, pack gate green. Merged.
**Container:** Breadify v1 (7 packs).
**Branch:** `pack-5-configure`.

## Goal

Build step 3's controls and wire every one of them to the paginator, with a
sample block that redraws from the same display list the PDF is drawn from.

**Observable when done:** change a setting and the sample block on the right
changes with it — and so does the printed sheet, because both are drawn from
one list that only the layout decides.

## Items

| # | Item | Done when |
| --- | --- | --- |
| 1 | Settings threaded through the layout | One `Settings` value carries the order-ID toggle, the no-substitutes treatment and the crate rules, and the paginator takes it — nothing reads a global default any more |
| 2 | The display list on screen | An egui renderer draws a `Page` into a `Ui` at a chosen scale, measuring nothing and laying out nothing |
| 3 | The field rail | Six always-printed fields shown as locked, `Order ID` as the one toggle (D11), `Route ordering` struck through with the reason |
| 4 | The marker treatment | The three no-substitutes treatments the design offers, each previewing itself, and the choice reaches the sheet |
| 5 | Crate rules | Crate capacities, and a size for each of the file's products as a percentage of a slot (D17), with the crate count updating as they change |
| 6 | The sample block | One real stop drawn at print size beside the controls, redrawing as settings change |
| 7 | Every control moves the paper | A test proves each setting changes the emitted primitives — a toggle nobody wired up is worse than no toggle |

## Ledger

- [x] 1 — Settings threaded through the layout · order ID, marker, crate rules
- [x] 2 — The display list on screen · `app::preview`, the second renderer
- [x] 3 — The field rail · six locked, one toggle, one struck
- [x] 4 — The marker treatment · three treatments, each reaching the sheet
- [x] 5 — Crate rules · capacities plus a size for every product
- [x] 6 — The sample block · a real stop, drawn from the same list
- [x] 7 — Every control moves the paper · four tests, one per control

**Deviations:**

- **Density is not built.** The design offers three densities (11.5 / 11 /
  10 pt) and they would need a scale threaded through every type size and gap
  in the layout. It buys paper, not correctness, and D11 made the field set
  fixed anyway; deferred rather than half-done.
- Paper size and label language are not offered either: A4 is the only paper
  the app prints (D1) and the page copy is English throughout.
- The design's small-items chip list is replaced by D17's per-product size
  percentage, which is the same idea with arithmetic behind it.
**Code review:** one pass at high effort. Two findings, both fixed:

1. `resettle` paginated the whole day on every change — and a dragged control
   changes on every frame, so dragging a crate size re-laid-out 148 blocks
   sixty times a second. The work now waits until the hand comes off the
   control.
2. The sample block was drawn 520 px wide in a column that only had 466,
   clipping the tick boxes and the order id off the right of the window. Found
   by looking at a screenshot, which is the only way that one shows up.

**Pack gate:** `./scripts/verify.sh` — fmt, clippy `-D warnings`, build, 89
tests across 14 files, all green. The step was opened and looked at, and the
sample block cropped from the screenshot to check it against the sheet.
