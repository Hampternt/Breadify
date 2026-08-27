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

**D16 — A stop is one customer, or one department of a customer.**
(2026-08-27)
That is: **one stop = one order = one block = one crate label**. There is no
address-level stop. Route 11 is therefore 14 stops and route 14 is 10, and the
masthead's stop counter counts orders. Nothing needs address normalisation,
and the three `Street 17` spellings simply stay three stops.

Orders at the same address still sort adjacently (D2) and may share an address
heading as pure layout economy — but the count, the label and the crate are
per order. This narrows the design handoff's "grouped stops", which treated an
address as the stop; it costs roughly 3 sheets a day in repeated headings.

**D17 — Crate size is a per-bread modifier, not a small/large flag.**
(2026-08-27)
Each bread type carries a **size modifier**, default `1.0`, editable in the
app: rolls and small items below 1, bulky items above it. The arithmetic then
runs on *slots* rather than raw units:

```
slots  = ceil( Σ over the order's lines of  quantity × modifier )
tens   = slots / 10      rem = slots % 10
rem == 0     ->  tens × crate-of-10
rem in 1..5  ->  tens × crate-of-10  +  1 × crate-of-5
rem in 6..9  -> (tens + 1) × crate-of-10
```

With every modifier at `1.0` this reduces exactly to the flat-unit rule, so
the verified baseline still holds — Customer 012's nine departments are 13 crates
until modifiers are set. Crate capacities (10 and 5) stay configurable. This
replaces the design handoff's binary "14 of 35 marked small" list, which had
no formula behind it.

**D18 — v1 prints by handing the PDF to the system.** (2026-08-27)
The app writes the PDF and opens it in whatever the OS uses, so the user gets
their own printer picker, page-scaling control and print preview for free, on
both platforms, for about ten lines of code. A direct `lp -d … -o media=A4`
path on Linux comes after that if it earns its place. The Windows
`PrintDlgW` + rasterise route is explicitly **not** in v1: it is the largest
piece of platform-specific code in the project and it trades away vector text.
Revisit only if the bakery's own printer turns out to have no PDF handler.

The print step must tell the user to print at **actual size / 100 %, no
scaling** — a viewer's default "fit to printable area" shrinks A4 by ~4 % and
takes the 11 pt body below its floor.

**D19 — The department sits under the customer name, not beside it.**
(2026-08-27)
A heading is one crate label in two parts, so it is set as two lines: the
customer at 14 pt, and beneath it the `DPT` box at 10.4 pt against the
handoff's 11.9. Side by side the two read as equals and the eye has to work
out which is the site and which is the kitchen. This also frees the whole
right-hand end of the heading line for **D20**. It costs about 5.5 mm on each
of the blocks that carry a department; the sample day is 26 sheets either way.

**D20 — The crate glyphs belong to the right-hand group.** (2026-08-27)
They sit immediately left of the substitute marker rather than trailing the
customer name, so crates, marker and order id form one column the eye finds
once per block instead of a run that starts at a different place on every
line. Where a long customer name would reach them — five of the sample's 148
stops, the longest being 127 mm of a 194 mm column — the crates drop to the
department's line rather than collide.

**D21 — The words alone are the default no-substitutes marker.** (2026-08-27)
This closes the question deferred below. The handoff argued for two
independent channels — an inverted badge plus a heavy bar down the block — so
a refusal survives a photocopy, and that is still one click away on the
Configure step. It also makes one block in ten shout on a page that is read
top to bottom, and the bakery asked for the plainer sheet. The words print in
Archivo ExtraBold caps under every treatment, so nothing is lost by not
choosing.

**D22 — The crate rules persist; nothing else does.** (2026-08-27)
How many slots a crate holds and how much room each bread takes are facts
about the bakery, worked out once at the crates — so they are written to
`$XDG_CONFIG_HOME/breadify/crates.conf` (`%APPDATA%\breadify\crates.conf` on
Windows) whenever they change and read back at startup. The rest of the
settings — the order-ID toggle, the marker treatment, which routes are
selected — are decisions about today's print and start fresh every time.

The file is a few lines of text, not JSON, so the person who set the numbers
can open it and fix a typo without the app; each size carries its bread's name
as a label the app ignores. Anything unparseable is skipped rather than
refused — a settings file is never worth failing a print over.

## Open

Nothing blocking. The design handoff's eight overrides are settled (six adopted
as written, route totals kept as **D15**, the unsequenced flag reinstated by
**D3**), and D16–D18 close the three questions the build plan raised.

Two calls are deferred to the pack that needs them rather than open:

- The **actual size modifiers** per bread (D17 gives the mechanism and a
  default of 1.0; the numbers are the bakery's to set once someone looks at
  the crates).
- The **unsequenced flag's rendering** and English vs Norwegian page copy.
  (The default no-substitutes treatment was the third of these; **D21**
  settles it.)

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
