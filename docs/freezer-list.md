# The freezer list — decisions so far

The freezer counterpart to [`print-layout.md`](print-layout.md): the calls
made about what the freezer version is and what its printed sheet must do,
in the user's own terms. Input-side facts live in
[`freezer-format.md`](freezer-format.md).

**Status: built.** The loader work (F2, F3) and the freezer page (F6, F7)
are both in; what stays open is listed at the bottom. The sample freezer
day prints as 15 routes over 17 sheets — the same day through the bread
layout was 24.

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
and a 19-line total down to eight clean blocks, and the day from 24 sheets
to 17.

## Open

- **Whether `Position` prints** as a small where-to-look-first hint (see
  F3). The loader carries it; the page currently does not show it.
- The wording of the unsequenced-stops notice — "no position in their
  route" now collides with the `Position` column meaning a pick slot.
- The Configure step still offers the crate-size table when a freezer file
  is loaded. Harmless — nothing on a freezer sheet reads the sizes — but
  it could hide itself once the step knows the kind.
