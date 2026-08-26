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
*Settled by the design pass: quiet `want substitute: true`, and for false an
inverted badge plus a heavy left rule — see `print-spec.md` §5.*

**D9 — A stop is never split across a page break.** (2026-08-26)
If a block doesn't fit in the remaining space, the whole block moves to the
next page. Paper is cheaper than a half-picked crate.

**D10 — Nothing beyond D5 on the page, but the fields are the user's choice.**
(2026-08-26)
No address, comment or date on the default printout. Since the app is a
formatting tool, the user picks which fields make it onto the page — the D5
set is the default, not a fixed list.
*Second half superseded by **D11**: only the order ID ended up toggglable.*

**D11 — Only the order ID is toggglable.** (2026-08-26)
Everything else on the page is fixed: the user can show or hide `Order ID` and
nothing more. This overrides the design handoff's "any of the 15 columns on top
of six mandatory ones" — the page is a fixed form, not a column picker.

**D12 — Route labels print the nickname verbatim.** (2026-08-26)
`hau 1` and `hau 2` are ordinary routes that simply don't run in Stavanger.
Nothing about them is special *except* that the page must say `hau 1`, not
`1` — the masthead renders the nickname as it appears in the file rather than
the integer parsed out of it. (The integer is still what sorts them, D2.)
The unlabelled `Stavanger` column carries no meaning for the printout.

**D13 — Design mono, allow colour to reinforce.** (2026-08-26)
The printer at the bakery is sometimes a colour one and sometimes not, and the
app can't know which. So every distinction must be fully carried in black and
white; colour may only add redundancy on top.

**D14 — Product names print exactly as the file has them.** (2026-08-26)
No stripping of the trailing bakery name, even though it repeats what the
supplier column says. The picker knows the bread by its full name.

**D15 — Route totals stay.** (2026-08-26)
Each route's last page closes with a per-bread, per-supplier total, ordered
most to least, with a dot per full ten inside a single order. This reverses the
old "no summaries" non-goal — the bakery gets a cross-check, at a cost of about
5 of the 24 sheets in a full day.

**D3 reaffirmed — the unsequenced flag stays.** (2026-08-26)
The design pass deleted it; that deletion is rejected. Without the flag a
driver cannot tell "no position assigned" from "last delivery of the day", and
route 5 has five such stops in a row. `print-spec.md` §6 overrides the design
handoff on this one point.

## Open

Nothing. The design handoff's eight overrides are all settled: six adopted as
written, route totals adopted as **D15**, and the deletion of the unsequenced
flag rejected — **D3** stands.

## Next

The design pass is done; its output is in
`Printer page formatting application/design_handoff_breadify/`, which is the
source of truth for how the page looks. [`print-spec.md`](print-spec.md) is
the reconciled specification of what it must say and do, and records the one
place where it overrides the handoff (the unsequenced flag). This file stays
the decision log — when a decision changes, change it here and reflect it
there.

What is left is building it: a Rust desktop app that reads the export,
validates it (§6 of `excel-format.md`), and prints the page `print-spec.md`
describes.
