# Pack 8 — The freezer list

**Status:** in progress. Both design calls answered.
**Branch:** `pack-8-freezer-list` (not yet cut).

## Goal

Breadify opens the freezer export as well as the bread one.

**Observable when done:** drop `PSR-FREEZER-2026-01-23-to-2026-01-23 (1).xlsx`
on the window and get picking sheets that are right for frozen goods, with a
Check step that is quiet about everything that is merely *not bread*.

## Not the goal

**One sheet per route carrying bread and freezer together.** The two are
separate lists in the same shape, picked from the same warehouse against the
same route numbers, and the predecessor tool keeps them as two pages that link
to each other. Nothing in either file lines the two up: the samples are
different days, share no order ID, and their route nicknames disagree (`hau`
against `hau 1` and `hau 2`). Merging them per route needs a route-name
reconciliation nobody has asked for. One export at a time.

## What the freezer file is

Structurally the same export — one `Data` sheet, the same 14 headers plus the
unlabelled region column. Different in four ways that reach the code:

| | Bread (352 rows) | Freezer (231 rows) |
| --- | --- | --- |
| Distinct products | 35 | 113 |
| Suppliers | 2, both bakeries | 7, most on one route |
| Position (column F) | always a bakery heap | warehouse shelf codes, empty on 26 rows |
| Route nicknames | `1`–`14`, `hau 1`, `hau 2` | `1`–`13`, `hau`, `Svg Employee` |

Product IDs do not collide across the two files today — 35 against 113, zero
shared — so a size rule in `crates.conf` cannot cross from a bread to a frozen
product. That is a property of this pair of exports and not a guarantee; if
crates ever apply to a freezer sheet, the store key needs the list kind in it.

## Items

| # | Item | Done when |
| --- | --- | --- |
| 1 | The filename carries the list kind | `date.rs` reads `PSR-<kind>-<from>-to-<to>` and hands the kind on; the folder scan in `main.rs` finds either file |
| 2 | Position may be empty | The freezer export loads at all — column F is read as optional text, and 26 blank cells are not 26 findings |
| 3 | A total wider than two suppliers | Route 8 has 7 suppliers; the total block lays out every route in both samples without collision, under test |
| 4 | The Check step knows which list it is | A clean freezer export raises no notice that is not real — familiar suppliers and route nicknames are per kind |
| 5 | Crates on a freezer sheet | A freezer sheet carries no crate glyph, no per-stop count, and the Configure step does not offer sizes for a list that has none |
| 6 | The sheet says which list it is | Header, drop-zone copy and the exported PDF's name all name the list |
| 7 | Docs, inventory, and the freezer sample under test | `excel-format.md` covers both kinds; the sample day paginates in the suite |

## The two calls, answered

**No crate count on a freezer sheet.** The arithmetic is bread-shaped — fifty
units to a large crate, each product a fraction of a slot — and `Lasagne 2,5
Kg` and `Hamburgerbrød 48stk Eske` are not slot-shaped. A wrong number on a
sheet the driver trusts is worse than no number, and 113 products is not a list
anyone will size by hand. A second set of rules for frozen goods would be a
pack of its own; it does not ride along inside this one.

**The warehouse shelf position stays off the printed line.** It is the one
column that could order a walk through the warehouse, and the predecessor tool
read it and deliberately ignored it. The freezer line keeps the same shape as
the bread line.

## Ledger

- [ ] 1 — The filename carries the list kind
- [ ] 2 — Position may be empty
- [ ] 3 — A total wider than two suppliers
- [ ] 4 — The Check step knows which list it is
- [ ] 5 — Crates on a freezer sheet
- [ ] 6 — The sheet says which list it is
- [ ] 7 — Docs, inventory, and the freezer sample under test

**Evidence so far:** `cargo run -- dump 11 "PSR-FREEZER-…xlsx"` answers
`breadify: cell F2 should hold text but reads ""` — the loader refuses the file
before anything else can be judged. That is item 2, and it is the whole of what
stands between here and a first look.
