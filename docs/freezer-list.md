# The freezer list — decisions so far

The freezer counterpart to [`print-layout.md`](print-layout.md): the calls
made about what the freezer version is and what its printed sheet must do,
in the user's own terms. Input-side facts live in
[`freezer-format.md`](freezer-format.md).

**Status: built.** The loader work (F2, F3), the freezer page (F6–F9) and
the Check step's kind override (F10) are in; what stays open is listed at
the bottom. The sample freezer day prints as 15 routes over 21 sheets —
the same day through the bread layout was 24.

## Decided

**F1 — The freezer sheet is a checking list, not a picking list.**
(2026-08-27)
The bread driver picks a route out of a heap, packs and labels crates. The
freezer worker's box is **already packed**: they find their freezer box and
check that everything on the list is in it. Same list-shaped job, opposite
direction — the sheet confirms contents rather than directing work. It can
be simpler than the bread sheet, and rests on the same data spine.

**F2 — One loader, two kinds, told apart by the filename.** (2026-08-27)
The freezer export is the same file format in every structural respect, so
there is one reader and one validation pass. `PSR-BREAD-` / `PSR-FREEZER-`
is the only mark of kind and decides only what counts as *familiar* (each
list's own suppliers; words-only route nicknames are ordinary on the
freezer list). A renamed file validates as bread — the stricter reading,
and the kind every check was first derived from. Both prefixes are found by
the no-argument CLI and both carry the delivery date the same way.

**F3 — `Position` is optional, product-keyed, and carried but not
printed.** (2026-08-27)
On the freezer list it is a pick slot in the wholesaler's warehouse, absent
for 17 of 113 products (the bread list's `Position` was the supplier again,
D4). A slot in asko's warehouse means nothing to someone checking a packed
box, so it stays off the page — but the loader keeps it, because a later
call could print it small as a where-to-look-first hint. Whether it earns
that place is open.

**F4 — Suppliers are a lookup cue, nothing more.** (2026-08-27)
The user's call: *suppliers are not important here beyond being something
you can look for when checking if everything is in the crate.* So the
supplier appears in small print on the line — it helps identify an item in
a full box — and nothing groups, sorts, totals or draws columns by it. The
bread sheet's per-supplier route totals (D15) exist to check a bakery
delivery that arrives in a heap; the freezer flow has no such heap, so no
totals. This sidesteps the whole 7-supplier layout question.

**F5 — Customer and department stay the block structure.** (2026-08-27)
The user's call: *customer / department is still needed.* One block per
order, customer name with the department under it, exactly as D7/D16/D19
settled for bread — the checker must know **whose** items they are
confirming, and the freezer sample leans on it the same way (Customer 012
Department 35: one address, eight department orders on one route).

**F6 — Boxes are not modelled. The sheet lists the items.** (2026-08-27)
The user's call: *you don't need to track boxes or any of that — just the
items.* Nothing in the file names a box and nothing on the sheet pretends
to: the customer/department blocks (F5) are the only grouping, and the
route-per-sheet structure transfers from the bread page whole.

**F7 — The freezer page is the bread page minus the picking machinery.**
(2026-08-27)
Everything the bread sheet settled carries over unchanged — masthead,
blocks in delivery order, the department box, the substitute marker, the
unsequenced flag, and the `P` / `M` / `F` tick boxes on every line, which
were already a checklist waiting for this job. What changes:

- **No crate glyphs** and no slot arithmetic anywhere (F4): a checker
  counts nothing into crates.
- **No route total**: it was a receiving check against the bakeries'
  unsorted delivery (D15), and the freezer goods arrive packed.
- **The legend reads `P Packed`** instead of `P Picked`, drops the crate
  key, and its supplier key names the wholesalers *on this route* (code
  and name; codes alone if a crowded route runs out of band) rather than
  the two house bakeries.
- **The page note says `check list`** where the bread page says `in full`,
  so the two sheets cannot be mistaken for one another in a stack.

On the sample day this takes route 13's Customer 012 page from crate-glyph rows
and a 19-line total down to eight clean blocks.
*Partly superseded the same day: **F8** reshapes the line (the `F` box is
gone and `P` became `C`) and **F9** brings a total back in flat form — the
legend now reads `C Checked · M Missing` and the day is 21 sheets.*

**F8 — The check line: checked box, dotted note field, missing box.**
(2026-08-27)
The user's call: *there doesn't need to be a missing and fixed box any
more — a checked/confirmed box on the left, a missing box on the right,
and a dotted comment field in between.* So the freezer line is
`C` box · quantity · supplier code · product name · a leader of full stops
for the checker's pen (how many short, what was substituted) · `M` box at
the right edge. The bread line keeps its `P` / `F` / `M` boxes untouched.
A name long enough to leave no room simply has no field — nothing wraps.

**F9 — Totals come back, flat.** (2026-08-27)
The user's call: *at the bottom have totals.* Each route closes with
`Route N total — {types} · {units} · most to least`: one list in two
balanced columns, read down the first then down the second. No bakery
columns and no ten-dots (those are receiving-check machinery, D15), and no
supplier code either — that cue lives on the stop lines, and the longest
freezer names need the room a totals half-column has.

**F10 — The kind stays automatic, with a visible override on the Check
step.** (2026-08-27)
The user's call: *keep it auto, but have a visible, obvious button on the
check segment in case it's the wrong name or a custom name.* So the
filename keeps deciding (F2), and the Check step carries a `TREATED AS`
bar — `BREAD` / `FREEZER`, the current answer solid — that flips the open
file to the other list. Flipping re-runs the validation on the spot, since
what counts as familiar depends on the kind, and re-paginates the sheets.
The override is per-file: the next file opened is read from its own name
again. The CLI has no flag for this — a renamed file there still validates
as bread — and gets one only if someone actually renames files at a
terminal.

## Open

- **Whether `Position` prints** as a small where-to-look-first hint (see
  F3). The loader carries it; the page currently does not show it.
- The wording of the unsequenced-stops notice — "no position in their
  route" now collides with the `Position` column meaning a pick slot.
- The Configure step still offers the crate-size table when a freezer file
  is loaded. Harmless — nothing on a freezer sheet reads the sizes — but
  it could hide itself once the step knows the kind.
