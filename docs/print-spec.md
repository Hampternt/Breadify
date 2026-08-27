# Printed pick list — specification

What the printed A4 page must communicate, and the rules it must obey.

**How this file relates to the design handoff.** The design pass is done; its
output lives in
`Printer page formatting application/design_handoff_breadify/`. That handoff is
the source of truth for **how the page looks** — geometry, typography, fills,
copy, all of it measured and verified against the real export. This file is the
source of truth for **what the page must say and do**. Where the two disagree,
the handoff wins on appearance and this file wins on behaviour; §6 is the one
place where this file deliberately overrides it.

Decision history and the reasoning behind each rule:
[`print-layout.md`](print-layout.md) (**D1**–**D15**). Input data:
[`excel-format.md`](excel-format.md).

---

## 1. Who reads it

A Rust desktop app (Windows + Linux) reads the daily bread order export and
prints picking lists on A4. Two people read the same paper:

- **The picker**, at the bakery, packing bread into crates and labelling each
  crate for a customer or department. They work down the page with their hands
  full, ticking boxes as they go.
- **The driver**, delivering those crates in route order.

It is a working document — creased, carried, read at arm's length, ticked with
a pen. Not a screen, not an archive.

## 2. Hard constraints

1. **A4, one route per page set.** A route always starts a fresh page; **no
   page ever carries two routes**, even when the previous route ended near the
   top. Multi-page routes are normal. (**D1**)
2. **Top to bottom is delivery order.** Stops appear by `Route ordering`
   ascending, ties broken address → department → order ID so two runs of one
   file print identically. (**D2**)
3. **The sequence number is never printed.** The order of the blocks carries
   it. `Route ordering` is the one column marked never-printed. (**D6**)
4. **Unsequenced stops print last, under a flag.** See §6. (**D3**)
5. **A stop block never splits across a page break.** Grouped stops may split
   *between* sub-blocks, repeating the address heading with a `same stop` note.
   The route total never splits either. (**D9**)
6. **Every block states whether substitutes are accepted**, in a way that
   survives a black-and-white printer. See §5. (**D8**, **D13**)
7. **A department is a crate label and is rendered as one** — an outlined box
   carrying the department name, never small print, never merged away.
   (**D7**)
8. **Route labels print verbatim.** `hau 1` prints as `hau 1`, not `1`. The
   integer prefix sorts the routes; the label shows the nickname. (**D12**)
9. **Product names print exactly as the file has them** — the trailing bakery
   name stays, nothing is truncated. (**D14**)
10. **Only the order ID is user-toggglable.** Everything else on the page is
    fixed. (**D11**, §8)

## 3. What is on the page

**Page furniture** — masthead (logo, `Route`, the route label, `continued` on
later pages, date from the filename, `Page 1 of 2 · 13 stops · 34 lines`), a
page-note line, a legend strip explaining the boxes and codes, and a footer
carrying the continuation state and the source filename.

**A block** is one order: one customer — or one department of one customer —
and the bread they get. It is also one crate label.

| Element | Source | Weight | Notes |
| --- | --- | --- | --- |
| Customer | `Customer` | **Loud** — block heading | Verbatim; up to 46 chars, ALL-CAPS and mixed case both occur |
| Department | `Department` | **Loud** — outlined `DPT` box | The crate label. An order without one shows no box; its order ID tells it apart |
| Crate count | derived | Loud — glyphs beside the name | How many crates that block's bread needs, at 10 and 5 per crate, adjusted by the small-items list |
| Accept alternatives | `Accept alternatives` | Asymmetric — see §5 | Quiet when true, loud when false |
| Quantity | `Quantity` | **Loud** — the number counted to | 1–48 |
| Supplier | `Supplier` | Code on each line | `SB` / `BH`, spelled out in the legend and the totals. The code, not the name — that is what paid for 11 pt body text |
| Bread type | `Product Name` | **Loud** | Verbatim, up to 57 chars, never truncated |
| Tick boxes | — | Present, empty | **P**icked, **M**issing, **F**ixed, one set per bread line |
| Order ID | `Order ID` | **Quietest** | The only distinguishing mark between several no-department orders at one address, so never smaller than ~8 pt |
| Route total | derived | Closes the route | See §7 |

Nothing else prints: no address on a block (the group heading carries it), no
comment, no product ID, SKU, position or region.

## 4. Hierarchy

1. **Route** — findable without reading body text, from a held stack.
2. **Customer + department** — what gets copied onto a crate; readable at
   bench distance.
3. **Quantity + bread type** — the picking work, read as a pair.

Supplier code, tick boxes and order ID are reference marks and read as such.

## 5. The substitute marker

The one field that changes the picker's behaviour when a bread is sold out.
90 % of orders accept substitutes, 10 % refuse.

- **Deliberately asymmetric.** `true` is quiet — a small mono line in the
  block heading. `false` is loud: an inverted badge *plus* a heavy rule down
  the left of the whole block.
- **Two independent channels** for the loud state, so it survives a mono laser
  printer and a photocopy.
- **Mono is the baseline.** The bakery's printer is sometimes colour and
  sometimes not, and the app cannot know which. Colour may reinforce; it must
  never be the only thing carrying a distinction. (**D13**)
- Both states stay readable — the picker never has to infer "yes" from an
  absence. The legend strip states the convention regardless.

## 6. The unsequenced tail — this file overrides the handoff

`Route ordering = 0` means **no position was assigned**, not "deliver last":
37 rows across 9 routes, and five different stops share `0` on route 5 alone.
Those stops sort to the end of their route (§2) — and **must be visibly marked
as unplaced**. (**D3**)

The design handoff dropped this marker. That is the one point where it is
wrong: without it the driver cannot tell "the system had no position for this"
from "this is the last delivery of the day", and on route 5 that mistake spans
five consecutive stops.

Requirements for the flag:

- A visible separator between the last sequenced stop and the first unplaced
  one, carrying a short line to the effect of *no position assigned — driver
  decides the order*.
- Quiet, not alarming: this is orientation, not a warning. The page-note mono
  style is the right register; it must not compete with the no-substitutes
  badge.
- If the tail splits across a page break, the following page repeats it.
- A route with no unsequenced stops shows nothing at all.

Exact rendering is the design's call, within that register.

**One caveat for whoever builds this:** the handoff's verified page budgets
(§9, "24 sheets") were measured *without* the flag, since the design had
deleted it. Reinstating it adds a line to each of the nine routes that have
unsequenced stops — `2, 4, 5, 8, 9, 11, 12, hau 1, hau 2`. Route 11 page 1 is
the tight one at 28 mm of clearance, and it has an unsequenced stop, so
re-measure that page before trusting the sheet count.

## 7. Route totals

Each route's last page closes with a **route total**: how much of each bread
the whole route needs. It is a cross-check for the bakery, not picking work —
so no tick boxes. (**D15**)

- One column per supplier, Sandnes Bakeri first, each headed with the code,
  the full bakery name and its own `types · units` count.
- Rows in **descending quantity**, ties broken by name so two runs match.
- A **ten-dot** per full ten *inside a single order* —
  `sum over lines of floor(line_qty / 10)`, never `floor(total / 10)`. It says
  how many full trays can be pulled whole. A bread whose orders never reach
  ten shows no dots.
- Never splits across a page break; if it does not fit, it moves and the stops
  are rebalanced so the last page is not nearly empty.

Cost: about 5 of the 24 sheets across a full day. Accepted.

## 8. What the user can change

**On the page: the order ID, and nothing else.** (**D11**) The printed form is
fixed; the app is not a column picker. `Route ordering` remains never-printed.

The app's Configure step still owns the things that are not field choices:
paper size and label language, print density, which of the three
no-substitutes treatments is used, and the crate rules — crate capacities plus
the list of which bread types count as small, which feeds the crate glyphs.

## 9. Size envelope

From the 2026-03-04 export (352 lines, 148 orders, 16 routes). Design against
the maxima.

| Quantity | Typical | Max |
| --- | --- | --- |
| Blocks (orders) per route | 5–11 | **14** (route 11) |
| Lines per block | 1–3 | **7** |
| Lines per route | 12–34 | **34** (route 5) |
| Distinct bread types on one route | — | **14** |
| Blocks at one address | 1 | **9** (Customer 012, one stop) |
| Product name | ~35 chars | **57 chars** |
| Customer name | ~25 chars | **46 chars** |
| Department | ~12 chars | **38 chars** |
| Quantity | 1–4 | **48** |

**Height binds, not width.** The design handoff says the longest product name
needs ~148 mm of its 150 mm column at 11 pt, and reasons from that. It does
not: measured off the embedded Space Grotesk 400 tables, the 57-character
`Holdbart Havrebrød Skåret 750g Bakehuset (har Vært Fryst)` is **112.65 mm**
at 11 pt — 29.03 em over a 1000-unit em. Two independent parses agree, and
kerning only tightens it further.

So the product column has roughly 37 mm of slack, and neither the supplier
code nor the tight margins are forced by width. Keep both — the code reads
faster than a repeated bakery name and the design is built around them — but
do not treat width as the constraint that settles layout arguments. Sheet
count is decided by height, and adding a column is a real option if one is
ever wanted.

Verified budgets from the design pass: **24 sheets** for the whole sample day
at the default density; 19 without route totals; route 5 needs two pages at any
density that respects 11 pt. Every page keeps ≥ 10 mm of clearance above the
footer.

## 10. Worked example — route 8

Content and structure, not geometry. `■` is a crate of ten, `◪` a crate of
five; `☐` are the P / M / F tick boxes.

```
┌ MATVARE EXPRESSEN ┐   ROUTE  8                    2026-03-04
                                            Page 1 of 1 · 5 stops · 13 lines
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 Route 8 in full — 5 stops.        want substitute: true unless marked FALSE
 BOXES ☐P picked ☐M missing ☐F fixed   CRATES ■ 10 ◪ 5     SB Sandnes Bakeri · BH Bakehuset
───────────────────────────────────────────────────────────────────────────

Customer 024  ■ ◪                    want substitute: true  1000621240
 ☐  1  SB  Dansk Rugbrød Hel Sandnes Bakeri                        ☐M ☐F
 ☐  1  SB  Grovbrød M/sirup Oppdelt Sandnes Bakeri                 ☐M ☐F
 ☐  3  SB  Sekskornsbrød Oppdelt Sandnes Bakeri                    ☐M ☐F
 ☐  2  SB  Havrebrød Oppdelt Sandnes Bakeri                        ☐M ☐F
 ☐  4  SB  Ryfylkebrød Sandnes Oppdelt Bakeri                      ☐M ☐F
 ☐  3  SB  Sandnesbrød Oppdelt Sandnes Bakeri                      ☐M ☐F

Customer 041  ■                      want substitute: true  1000622666
 ☐  5  SB  Grovbrød M/sirup Oppdelt Sandnes Bakeri                 ☐M ☐F
 ☐  2  SB  Havrebrød Oppdelt Sandnes Bakeri                        ☐M ☐F
 ☐  2  SB  Sandnesbrød Oppdelt Sandnes Bakeri                      ☐M ☐F

Customer 005  ■  want substitute: true  1000620185
 ☐ 10  BH  Barnehagebrødet - Oppskåret 750g                        ☐M ☐F

─── no position assigned — driver decides the order ───────────────────────

▌Customer 054  ■                    ██ WANT SUBSTITUTE: FALSE ██  1000622461
▌☐  4  SB  Grovbrød M/sirup Oppdelt Sandnes Bakeri                 ☐M ☐F
▌☐  4  SB  Ryfylkebrød Sandnes Oppdelt Bakeri                      ☐M ☐F

Customer 070  ◪             want substitute: true  1000622554
 ☐  2  SB  Sekskornsbrød Oppdelt Sandnes Bakeri                    ☐M ☐F

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Route 8 total
7 bread types · 43 units · most to least
● one full ten inside a single order — 1 on this route

 SB Sandnes Bakeri        6 types · 33 units    BH Bakehuset      1 type · 10 units
 ─────────────────────────────────────────      ──────────────────────────────────
 10  Grovbrød M/sirup Oppdelt Sandnes Bakeri    10  Barnehagebrødet - …  750g   ●
  8  Ryfylkebrød Sandnes Oppdelt Bakeri
  5  Sandnesbrød Oppdelt Sandnes Bakeri
  5  Sekskornsbrød Oppdelt Sandnes Bakeri
  4  Havrebrød Oppdelt Sandnes Bakeri
  1  Dansk Rugbrød Hel Sandnes Bakeri
───────────────────────────────────────────────────────────────────────────
Route 8 — end of route          PSR-BREAD-2026-03-04 · Matvare Expressen
```

Things to read off it: Customer 024's 14 units become one crate of ten plus one of
five; the two unsequenced stops sit under the flag rather than looking like
final deliveries; A3 refuses substitutes and gets both the badge and the left
rule; the only ten-dot on the route is Trollhaugen's single line of 10.

**Grouped stops** work the same way with an address heading above the
sub-blocks — `Customer 012 · Street 12 · one stop ·
13 crates`, then nine `DPT` sub-blocks flush beneath it.

Two corrections to the handoff here, both recomputed from the sample:

- **Customer 012 is 13 crates, not nine.** Nine is the department count; the nine
  orders are 4, 11, 12, 21, 7, 7, 5, 4 and 6 units, which the §3 crate rule
  turns into 1, 2, 2, 3, 1, 1, 1, 1, 1 = **13**. The prototype draws 13
  rectangles while its own heading says nine. Neighbouring Boganes reads
  `8 crates` correctly only because its eight orders happen to need one each.
- **Grouping does not keep a big site to one page.** Route 14 is 10 orders and
  splits 7/3 across two sheets, Customer 012 along with it. What grouping buys is one
  *stop* and eight fewer repeated address headings — real, but not that.
  Route 13's Street 38 (8 orders, all at ordering 600) is the site that
  genuinely fits on one page.

## 11. Still open to design

- Exact rendering of the unsequenced flag, within §6's register.
- Which of the three no-substitutes treatments ships as the default.
- Norwegian or English page copy (the design ships English labels with
  Norwegian data verbatim).

## 12. Non-goals

- **Not a screen UI.** The app window is Part 2 of the design handoff and a
  separate problem.
- **No per-product picking view.** Route totals (§7) are a cross-check, not a
  bakery-wide pick sheet.
- **No route optimisation.** The sequence comes from the file and is never
  second-guessed — including the `0`s, which are surfaced rather than solved.
- **No prices, invoicing or signature lines.** None of that data exists in the
  input.

## 13. Acceptance checks

Printed in mono on A4:

1. No page carries two routes, and no stop block is split across a break.
2. For any block, a stranger can tell whether substitutes are allowed —
   without a legend lookup and without colour.
3. The nine Customer 012 departments are distinguishable at arm's length.
4. Route 8's two unsequenced stops read as unplaced, not as the last
   deliveries of the day.
5. No sequence numbers appear anywhere.
6. `hau 1` prints as `hau 1`.
7. Product names appear exactly as in the file, none truncated.
8. Each route's last page carries its total, and the ten-dots count full tens
   *within an order* — route 11's Kneippbrød (2, 7, 4, 10, 11, 20, 6, 8) shows
   **4** dots, not 6.
