# The freezer list — decisions so far

The freezer counterpart to [`print-layout.md`](print-layout.md): the calls
made about what the freezer version is and what its printed sheet must do,
in the user's own terms. Input-side facts live in
[`freezer-format.md`](freezer-format.md).

**Status: accumulating.** The loader work (F2, F3) is built; the printed
sheet is not — a freezer export currently prints through the bread layout,
which is legible but says more than this job needs.

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

## Open

- **What is a box?** Nothing in the file names one. If one box = one route,
  the bread sheet's route-per-page structure transfers whole; if boxes are
  packed per customer, the sheet needs per-box boundaries beyond the
  customer blocks. Needs the warehouse's answer, not the file's.
- **The checklist rendering** — whether lines carry a tick box, and what
  else changes against the bread block (crate glyphs and slot arithmetic
  presumably go; `Accept alternatives` presumably stays, since a checker
  meeting a substitute needs to know whether one was allowed).
- **Whether `Position` prints** as a small hint (see F3).
- The wording of the unsequenced-stops notice — "no position in their
  route" now collides with the `Position` column meaning a pick slot.

## Next

The loader accepts freezer exports (see `freezer-format.md` §4 for what
that means); the checklist page is the next pack, and starts by settling
the box question above.
