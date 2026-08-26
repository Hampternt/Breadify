# Printed pick list — design brief

The instruction spec for designing the printed page. Hand this file to a
UI/design pass; it says what the page must communicate and what it must not,
and leaves the visual decisions open where they are genuinely open.

Companion files: [`print-layout.md`](print-layout.md) is the decision log this
brief is derived from (decisions are cited as **D1**–**D10**);
[`excel-format.md`](excel-format.md) describes the input data.

---

## 1. What this page is

A Rust desktop app (Windows + Linux) reads a daily bread order export and
prints picking lists on **A4**. One page set per route; the driver takes their
route's pages and works down them.

Two people read the same paper:

- **The picker**, at the bakery, packing bread into crates and labelling each
  crate for a customer or department. They read top-to-bottom, hands full,
  glancing back at the sheet between crates.
- **The driver**, delivering those crates in order. They read it as a running
  order of stops.

Design for a working document that gets creased, carried, and read at arm's
length — not for a screen and not for an archive.

## 2. Hard constraints

These are settled. A design that breaks one of them is wrong, however good it
looks.

1. **A4, one route per page set.** A route that overflows continues onto more
   pages. **No page ever contains two routes** — a new route always starts on
   a fresh page, even if the previous one ended near the top. (**D1**)
2. **Order on the page is delivery order.** Stops appear in `Route ordering`
   ascending; the reader may assume top = first delivered. (**D2**)
3. **The sequence number itself is never printed.** The order of the blocks
   carries it. (**D6**)
4. **Unsequenced stops (`Route ordering = 0`) go last on their route, visibly
   flagged** as "no position assigned — driver's choice". They are not late
   stops; they are unplaced ones, and the flag must say so. (**D3**)
5. **A stop's block never splits across a page break.** If it doesn't fit, the
   whole block moves to the next page. (**D9**)
6. **Every block carries the accept-alternatives state**, unmistakably, in a
   way that survives a black-and-white printer. (**D8**)
7. **A department is its own block**, with its department name visible —
   never folded into a shared customer heading. (**D7**)

## 3. What each block says

A **block** is one order: one customer (or one department of one customer) and
the bread they get.

| Element | Source | Weight | Notes |
| --- | --- | --- | --- |
| Customer | `Customer` | **Loud** — block heading | Up to 46 chars; mixed case and ALL-CAPS both occur in the data |
| Department | `Department` | **Loud** — part of the heading | Present on ~26 % of orders; this is the crate label, so it must be as findable as the customer name |
| Accept alternatives | `Accept alternatives` | **Loud** — one per block | See §5 |
| Quantity | `Quantity` | **Loud** — the number the picker counts to | 1–48, at most 2 digits |
| Bread type | `Product Name` | **Loud** | Up to 57 chars; the long ones matter (`Kneipp Holdbar Skåret 750g Bakehuset (har Vært Fryst)`). Most names already embed the bakery name — see §7 |
| Supplier | `Supplier` | Quiet | `sandnes bakeri` / `bakehuset`; helpful, not critical. Lower-case in the data — title-case it |
| Order ID | `Order ID` | **Quietest** | 10 digits. Present for lookups, deliberately out of the way |
| Route | `Route nickname` | Page header | `1`–`14`, `hau 1`, `hau 2` |

Nothing else is on the page by default — no address, no customer comment, no
date. (**D10**) The app will let the user turn extra fields on; design the
block so an added line doesn't wreck it, but don't design *for* those fields.

## 4. Hierarchy

Three levels, in this order of prominence:

1. **Route** — the page's identity. A picker holding a stack must find route 11
   without reading body text.
2. **Customer + department** — where a crate goes. This is what gets copied
   onto a crate label, so it should be readable from the distance a person
   stands from a bench.
3. **Quantity + bread type** — the picking work. Quantity and product name are
   read as a pair; a design where the eye has to travel far between them
   invites miscounts.

Everything else — supplier, order ID — is reference material and should read
as such.

## 5. The accept-alternatives marker

The one piece of information that changes the picker's behaviour when a bread
is sold out: **may they substitute another product, or must the crate go
short?** 90 % of orders in the sample say yes, 10 % say no.

Requirements, not a design:

- Legible at a glance, per block, without cross-referencing another part of
  the page.
- **Assume mono.** It must survive a black-and-white laser printer and a
  photocopy. Colour may reinforce the distinction but never carry it alone.
  (If colour printing turns out to be available, revisit this section.)
- **Asymmetric.** The two states must not look equally important: "no
  substitutes" is the one that changes behaviour and appears on ~10 % of
  blocks, so it gets the weight. A page where 90 % of blocks repeat the same
  loud badge is a page where the tenth block stops standing out.
- Both states must still be *readable* — the picker should never have to infer
  "yes" from absence without being told. If the quiet state is rendered as
  blank or as small text, the page carries a one-line legend saying so.

Wording is free (`Alt: yes/no`, `SUBSTITUTES OK`, `IKKE BYTT`, an icon) — the
users are Norwegian, so Norwegian labels are welcome if they read faster.

## 6. Size envelope

Real numbers from the 2026-03-04 export (352 lines, 148 orders, 16 routes).
Design against the maxima, not the averages.

| Quantity | Typical | Max |
| --- | --- | --- |
| Blocks (orders) per route | 5–11 | **14** (route 11) |
| Lines per block | 1–3 | **7** |
| Lines per route | 12–34 | **34** (route 5) |
| Distinct bread types on one route | — | **14** |
| Blocks at one address | 1 | **9** (Customer 012, all at the same stop) |
| Product name length | ~35 | **57 chars** |
| Customer name length | ~25 | **46 chars** |
| Department length | ~12 | **38 chars** |
| Quantity | 1–4 | **48** |

A worst-case route is ~34 bread lines plus 14 headings, plus whatever the
marker and order ID take — call it 70 typeset rows. **Do not treat one page
per route as a target.** Size the type for reading at arm's length first
(body text no smaller than 11 pt, headings clearly above that) and let the
page count fall out; multi-page routes are normal, not a failure. D1 and D9
both spend paper rather than legibility.

## 7. Worked examples

Content, not layout — these show what the page has to carry.

**Route 8** — five blocks, two of them unsequenced. Note the last two: they
sit under a flag, not at a position. Product names are the literal strings
from the file; the marker is deliberately asymmetric per §5.

```
ROUTE 8

Customer 024                                              alt: ja
    1  Dansk Rugbrød Hel Sandnes Bakeri            Sandnes Bakeri
    1  Grovbrød M/sirup Oppdelt Sandnes Bakeri     Sandnes Bakeri
    3  Sekskornsbrød Oppdelt Sandnes Bakeri        Sandnes Bakeri
    2  Havrebrød Oppdelt Sandnes Bakeri            Sandnes Bakeri
    4  Ryfylkebrød Sandnes Oppdelt Bakeri          Sandnes Bakeri
    3  Sandnesbrød Oppdelt Sandnes Bakeri          Sandnes Bakeri
                                                        order 1000621240

Customer 041                                              alt: ja
    5  Grovbrød M/sirup Oppdelt Sandnes Bakeri     Sandnes Bakeri
    2  Havrebrød Oppdelt Sandnes Bakeri            Sandnes Bakeri
    2  Sandnesbrød Oppdelt Sandnes Bakeri          Sandnes Bakeri
                                                        order 1000622666

Customer 005                          alt: ja
   10  Barnehagebrødet - Oppskåret 750g            Bakehuset
                                                        order 1000620185

── ingen rekkefølge satt — sjåføren velger ────────────────────────────

Customer 054                                    ⛔ IKKE BYTT VARER
    4  Grovbrød M/sirup Oppdelt Sandnes Bakeri     Sandnes Bakeri
    4  Ryfylkebrød Sandnes Oppdelt Bakeri          Sandnes Bakeri
                                                        order 1000622461

Customer 070                                     alt: ja
    2  Sekskornsbrød Oppdelt Sandnes Bakeri        Sandnes Bakeri
                                                        order 1000622554
```

**Route 14, Customer 012** — nine departments at one address, one delivery
stop, nine crates. This is the case that decides whether the layout works: the
department is the only thing distinguishing one block from the next, so it
cannot be small print. It also shows the longest product names at full length.

```
Customer 012 — Department 11                                  alt: ja
    3  Franskbrød U/valmuefrø Oppd Sandnes Bakeri  Sandnes Bakeri
    4  Sekskornsbrød Oppdelt Sandnes Bakeri        Sandnes Bakeri
    2  Havrebrød Oppdelt Sandnes Bakeri            Sandnes Bakeri
    3  Kneippbrød Oppdelt, Sandnes Bakeri          Sandnes Bakeri
    3  Ryfylkebrød Sandnes Oppdelt Bakeri          Sandnes Bakeri
    3  Sandnesbrød Oppdelt Sandnes Bakeri          Sandnes Bakeri
    3  Bakehuset Grovbrød M/sirup Skåret 750g      Bakehuset
                                                        order 1000621870

Customer 012 — Department 28                        alt: ja
    1  Ryfylkebrød Sandnes Oppdelt Bakeri          Sandnes Bakeri
    2  Bakehuset Grovbrød M/sirup Skåret 750g      Bakehuset
    2  Kneipp Holdbar Skåret 750g Bakehuset (har Vært Fryst)   Bakehuset
                                                        order 1000621945

    … 7 more departments, same address, same stop
```

Note what those literal names do to the layout: `Sandnes Bakeri` appears twice
on nearly every line, once inside the product name and once in the supplier
column. Whether to strip the trailing bakery name from `Product Name` is an
open question (see `print-layout.md`) — design so that either answer works.

## 8. Free choices

Deliberately not specified — decide these on design grounds:

- Table with ruled columns, or whitespace-separated blocks.
- How the accept-alternatives state is rendered (§5 gives the requirements).
- Typeface, sizes, rules, shading, and whether a block gets a box.
- How the unsequenced tail is separated (heading, rule, tint, indent).
- Page header and footer content beyond the route identity, including page
  numbering for multi-page routes.
- Whether the supplier is a column, a suffix, or a small tag.
- Norwegian or English labels.

## 9. Non-goals

- **Not a screen UI.** This describes paper. The app's own window is a separate
  design problem.
- **No totals, no summaries, no per-product picking view.** The list is
  per-customer; a bakery-wide "how many Ryfylkebrød today" view is not part of
  this.
- **No route optimisation.** The sequence comes from the file and is never
  second-guessed.
- **No prices, no invoicing, no signature lines** — none of that data exists in
  the input.

## 10. Acceptance checks

A design satisfies this brief if, printed in mono on A4:

1. Route 5 (34 lines, 13 blocks) and route 11 (14 blocks) each print without
   two routes ever sharing a page, and without a block being split.
2. A stranger can tell, for any block, whether substitutes are allowed —
   without a legend lookup, and without colour.
3. The nine Customer 012 departments are distinguishable at a glance while
   holding the page at arm's length.
4. The two unsequenced stops on route 8 are obviously unplaced rather than
   simply last.
5. No sequence numbers appear anywhere on the page.
