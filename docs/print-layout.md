# Printed list — decisions so far

Output side: what gets printed for the drivers. Companion to
[`excel-format.md`](excel-format.md), which covers the input file only.

**Status: accumulating.** These are the calls made so far, in the user's own
terms. Nothing here is implemented yet, and the open questions at the bottom
are still open.

## Decided

**D1 — Paper is A4, one route per sheet of paper.** (2026-08-26)
A route that doesn't fit runs onto further pages; a big route can be several
sheets. **No page ever carries two routes** — a route always starts on a fresh
page, even when the previous one ended a third of the way down. Page breaks
are therefore driven by route boundaries first and by overflow second.

**D2 — Sort: route, then delivery sequence.** (2026-08-26)
Routes in natural order (`1`..`14`, then `hau 1`, `hau 2`), and within a route
by `Route ordering` ascending — lower numbers are delivered first. Ties break
on address → department → order ID so two runs of the same file print
identically. See §4 of `excel-format.md`.

**D3 — `Route ordering = 0` means unsequenced: print those stops last, with a
flag.** (2026-08-26)
`0` is not position zero — five different stops on route 5 share it. Those
stops go after every sequenced stop on their route and are visibly marked, so
the driver knows the position is theirs to choose. See §3 of
`excel-format.md`.

**D4 — `Position` is just the supplier, not a pick location.** (2026-08-26)
Nothing is grouped, split or ordered by it. Orders that mix both bakeries
(37 of 148 in the sample) stay one stop on one list.

**D5 — What each stop must communicate.** (2026-08-26)
The picker works off: **quantity**, **bread type**, **customer**, and
**accept-alternatives as a visible true/false marker**. **Supplier** is
helpful and should be shown too. Order ID may appear but is not important —
keep it small and out of the way.

**D6 — `Route ordering` is not printed.** (2026-08-26)
The order of the list already says what to do first and last, so the number
itself is noise. It drives the sort (D2/D3) and then disappears; the
unsequenced flag from D3 stays, since that one carries information the
sequence cannot.

**D7 — A department is its own customer, and the department is shown.**
(2026-08-26)
Where `Department` is filled in, that order prints as its own block, not folded
into a shared customer heading, and the department name is visible on the
block. The picker labels crates by it: one customer can have several
departments each ordering several bread types, and the crates have to be told
apart. Customer 012 is the worst case — 9 departments, 9 orders, one
address, all at the same delivery position.

**D8 — Accept-alternatives must be communicated, presentation is free.**
(2026-08-26)
Every block has to make clear whether that customer wants replacement wares or
not. Colour, a true/false column, a word — the channel doesn't matter as long
as it is understood at a glance and survives a black-and-white printer.

**D9 — A stop is never split across a page break.** (2026-08-26)
If a block doesn't fit in the remaining space, the whole block moves to the
next page. Paper is cheaper than a half-picked crate.

**D10 — Nothing beyond D5 on the page, but the fields are the user's choice.**
(2026-08-26)
No address, comment or date on the default printout. Since the app is a
formatting tool, the user picks which fields make it onto the page — the D5
set is the default, not a fixed list.

## Open

1. **Which fields the picker can toggle** — D10 says the field set is
   configurable. Is that any of the 15 input columns, or a fixed shortlist
   (address, comment, date, order ID) on top of the mandatory D5 set?
2. **`hau` routes and the unlabelled `Stavanger` column** — do they mean
   anything for the printout (a depot heading, a separate batch)?
3. **Printer reality** — always mono laser, or is colour available? The spec
   assumes mono, which is safe either way, but colour would open up a cheaper
   way to carry D8.
4. **Bakery name inside the product name** — `Grovbrød M/sirup Oppdelt Sandnes
   Bakeri` next to a `Sandnes Bakeri` supplier column says it twice, and the
   names run to 57 chars. Strip the trailing bakery name for the printout, or
   print the product name exactly as the file has it?

## Next

The decisions above are written up as a design brief in
[`print-spec.md`](print-spec.md) — that is the file to hand to a UI/design
pass. This file stays the decision log; when a decision changes, change it
here and reflect it there.
