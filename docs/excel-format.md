# Input Excel format

What the exported order file looks like, derived by unpacking the OOXML of the
one sample in the repo. This describes **input only** — nothing about the app's
own behaviour beyond what the data forces.

- **Samples:** `PSR-BREAD-2026-03-04-to-2026-03-04 (1).xlsx` (27 994 bytes) and
  `PSR-FREEZER-2026-01-23-to-2026-01-23 (1).xlsx` (23 565 bytes)
- **Derived:** 2026-08-26 from the bread file, re-checked 2026-08-27 against
  the freezer one, by reading `xl/worksheets/sheet1.xml` directly
  (`tools/inspect_xlsx.py` re-derives everything here)

## 0. Two lists, one shape

The exporter writes a bread list and a freezer list, and they are the same file
format: one sheet named `Data`, the same fourteen headers, the same unlabelled
fifteenth column. Only the filename says which is which (**D23**).

Where the freezer file stretched what the bread file taught, and what it cost:

| | Bread, 352 rows | Freezer, 231 rows | |
| --- | --- | --- | --- |
| Distinct products | 35 | 113 | |
| Suppliers | 2, both bakeries | 7, all seven on route 8 | the route total was drawn for two columns (**D25**) |
| **Position** (F) | always a bakery heap | warehouse shelf codes, **no cell at all on 26 rows** | read as required text, which refused the whole file on row 2 |
| Route nicknames | `1`–`14`, `hau 1`, `hau 2` | `1`–`13`, `hau`, `Svg Employee` | the nickname check wanted a trailing number |

Product IDs do not collide across the pair — 35 against 113, none shared — so a
size rule saved against one list cannot reach the other. That is a property of
these two files and not a guarantee; if crates ever apply to a freezer sheet
(**D24** says they do not), the store key needs the list in it.

Every claim below is tagged:

- **[S] structural** — read out of the OOXML itself; true of this file by
  construction, and true of any file the same exporter produces.
- **[O] observed** — a pattern that holds across all 352 data rows of *one
  file from one day*. Plausible as a rule, not proven. The loader should
  **check** these at read time and report violations, not assume them.

---

## 1. Container

| Fact | Value |
| --- | --- |
| Format **[S]** | Real `.xlsx` (OOXML zip), not CSV-renamed |
| Producer **[S]** | `Microsoft Excel`, `AppVersion 12.0000` (Excel 2007 vintage), `rupBuild 4505` |
| Sheets **[S]** | Exactly one, named `Data` (`sheetId=1`) |
| Dimension **[S]** | `A1:O353` — 15 columns, 1 header row + **352 data rows** |
| Strings **[S]** | Shared-string table, 382 entries, no inline strings |
| Absent features **[S]** | No `cols` widths, no `autoFilter`, no `mergeCells`, no `dataValidations`, no tables, no formulas, no frozen panes, no styling that carries meaning |
| Encoding **[S]** | UTF-8; Norwegian `æ ø å` throughout, and one dept typo `` Department 33 `` with a backtick |

Practical consequence: any xlsx reader works. In Rust, `calamine` on the sheet
named `Data` is enough; nothing here needs style or format handling.

There is a stale LibreOffice lock file (`.~lock.…xlsx#`) next to the sample —
ignore it, it is not part of the format.

### The delivery date is not in the sheet **[S]**

No column anywhere holds a date. The only date is in the **filename**:
`PSR-BREAD-<from>-to-<to>.xlsx` → `2026-03-04` to `2026-03-04` (a single day).
`docProps` gives a created/modified stamp of `2026-03-03T14:21:57Z` — the day
before, i.e. the export time, not the delivery day.

If printed lists need a date on them, it must come from the filename (or be
entered by the user). Note the sample also carries a ` (1)` browser-download
suffix, so filename parsing has to tolerate trailing junk.

---

## 2. Columns

Header row is `A1:N1` — **14 headers**. Column `O` has data in every row but
**no header** **[S]**. A reader that derives its width from the header row
silently drops it; a reader that asserts `headers.len() == row.len()` breaks.

| Col | Header | Cell type | Empty | Distinct | Notes |
| --- | --- | --- | --- | --- | --- |
| A | `Order ID` | number | 0 | 148 | Groups lines into one order |
| B | `Quantity` | number | 0 | 22 | Integers 1–48 |
| C | `Product ID` | number | 0 | 35 | Internal product key |
| D | `Product Name` | string | 0 | 35 | Display name, 22–57 chars |
| E | `Supplier SKU` | **string** | 0 | 35 | Not numeric — see below |
| F | `Position` | string | 0 in bread, 26 in freezer | 2 in bread, 38 in freezer | `X-Sandnes Bakeri`; `W-05-02`, `U-Frysevare` |
| G | `Supplier` | string | 0 | 2 | `sandnes bakeri`, `bakehuset` |
| H | `Customer` | string | 0 | 120 | Free text, not a key |
| I | `Department` | string | **261** | 34 | Sub-location within customer |
| J | `Delivery street` | string | 0 | 119 | Street address, sometimes with floor/entrance |
| K | `Comment` | string | **342** | 5 | Order-level free text |
| L | `Route nickname` | **string** | 0 | 16 | `1`..`14`, `hau 1`, `hau 2` |
| M | `Route ordering` | number | 0 | 45 | Stop sequence, `0` = unsequenced |
| N | `Accept alternatives` | **bool** (`t="b"`) | 0 | 2 | 1 = substitutes OK, 0 = no |
| O | *(no header)* | string | 0 | 1 | `Stavanger` in all 352 rows |

### Empty cells are *absent*, not blank **[S]**

`I` and `K` are missing `<c>` elements entirely — there is no empty-string cell
to read. Three row shapes occur:

| Cells present | Rows |
| --- | --- |
| `ABCDEFGH_J_LMNO` (no dept, no comment) | 251 |
| `ABCDEFGHIJ_LMNO` (dept, no comment) | 91 |
| `ABCDEFGH_JKLMNO` (comment, no dept) | 10 |

So a positional reader must key off the cell reference (`I7`), never off cell
count. That no row has *both* `I` and `K` is coincidence at n=1 — do not encode
it. `calamine` handles this for you by returning `Data::Empty`.

No field in the sample has leading or trailing whitespace, except inside
comment text **[O]** — still worth trimming defensively.

### Per-column detail

**A — Order ID.** 10-digit integer, range `1000617801`–`1000622767`, 148
distinct values over 352 rows. Stored as a *number*, so a naive reader gets
`1000620628.0`; format as integer, and hold it in `i64` (it fits `i32` with
room to spare today, but the IDs are opaque and only grow). One order = one
delivery point's order for the day: 1–7 lines each (43 orders have 1 line, 44 have 2, 36 have
3, 18 have 4, and 7 have 5–7) **[O]**.

**B — Quantity.** Always a positive integer, 1–48. No decimals, no unit
column — the unit is implicit in the product name (`4pk`, `750g`, `10pk`) **[O]**.

**C/D/E — the product.** Mutually 1:1 in both directions across all 352 rows:
`Product ID ↔ Product Name ↔ Supplier SKU` **[O]**. `Product ID` is the safe
key. `Supplier SKU` is **text, not a number** — values include `107_san`,
`10022_bhb`, bare `115`, and one missing-underscore `21061bhb`. Parsing it as
an integer loses most of the catalogue.

**F/G — Position and Supplier.** Two values each and 1:1 with each other; the
`Position` string is `"X-"` + the supplier name in title case **[O]**.
`Supplier` is lower-case in this file, so compare case-insensitively.
On the freezer list it is not the supplier at all but a warehouse shelf, and
sometimes nothing. Nothing reads it either way (**D24**).

**Confirmed 2026-08-26: `Position` is just the supplier again, not a pick
location** — the two columns carry the same fact in two spellings, so use
`Supplier` and treat `Position` as redundant. Nothing splits or groups by it,
and the 37 orders that span both bakeries stay whole on one list. All 35
products map to exactly one supplier **[O]**.

**H — Customer.** Free-text and **not a stable key** **[O]**:

- `Customer 031` appears on *two* routes at two addresses (`3` / `hau 1`).
- The same site appears under two spellings: `Customer 061` and
  `Customer 017`; likewise `Customer 085` /
  `Customer 083` / `Customer 087`.
- Casing is inconsistent (`Customer 012` vs `Customer 031`), and some names
  carry internal codes (`K3214`, `k 3065`).

Group by **address**, not by name.

**I — Department.** Present on 91 of 352 rows (26 %). It is the sub-location
the goods go to inside a large customer — `Department 11`, `Department 22`, `Department 18`,
`Department 16`. Customer 012 is the clearest case: one address,
9 departments, 9 separate order IDs. Two near-duplicates exist in the data
(`Department 33 and `` Department 33 ``) — a source typo, not a format feature.

**J — Delivery street.** Street + number; occasionally a postcode
(`Street 12`) or an entrance hint
(`Street 112`, `Street 62`). 119 distinct
values. **`Delivery street → Route nickname` holds with zero violations** —
the address is the most reliable stop identity in the file **[O]**. But three
spellings of one building can exist (see route 11 in §4), so string equality
under-merges rather than over-merges: safe for correctness, imperfect for
tidiness.

**K — Comment.** Order-level free text, repeated onto **every line of that
order** — 10 rows carry 5 distinct comments belonging to 5 orders, and in each
case every line of that order carries the text **[O]**. Deduplicate before
printing. Content is real customer messaging in Norwegian (delivery-day
requests, quality complaints, and one `ved mangel ønsker ikke bytte varer` —
"if something is missing, do not substitute" — on an order whose
`Accept alternatives` is `0`, which corroborates that column's meaning).

**L — Route nickname.** **A string, even when it looks like a number** **[S]** —
14 numeric-looking routes `1`..`14` plus `hau 1` and `hau 2`. Sorting these
lexically gives `1, 10, 11, 12, 13, 14, 2, 3, …`, which is the single most
likely bug in the finished app. Sort naturally, i.e. in two buckets:
nicknames starting with an integer first, ordered by that integer, then all
others alphabetically with any trailing number compared numerically. On this
file that yields `1, 2, … 14, hau 1, hau 2`; `route_key()` in
`tools/inspect_xlsx.py` is the reference implementation.
(`hau` is probably Haugesund, but that is a guess — see §7.) Row counts per
route run 12–34.

**M — Route ordering.** Integer stop sequence, higher = later in the day.
45 distinct values, `0` to `4100`, gap-numbered rather than consecutive
(`100, 200, 250, 300, 400, 450, 500, 600, 700, 750, 800, 850, 1000, 1025, …`) —
not multiples of 50 or 100 (`1055`, `1390`, `1025` occur), so treat it as an
opaque sortable integer, never as an index. Values are only comparable *within*
a route **[O]**. `0` is a sentinel, see §3.

**N — Accept alternatives.** A genuine Excel boolean (`t="b"`, value `0`/`1`),
so `calamine` yields `Data::Bool` — matching on Int fails. `1` = the customer
accepts a replacement product if their bread is out of stock; `0` = deliver
short instead. 318 rows `1`, 34 rows `0`. It varies **between orders of the
same customer** (`Customer 037` and `Customer 061` each have
both) **[O]**, so it belongs to the order, never to the customer.

**O — unlabelled.** `Stavanger` in all 352 rows. Reads like a depot or region
for the whole export, which is why it has no header and never varies here.
With one file and one value there is no way to tell whether it can vary or
whether a second file adds a header. Read it positionally, carry it through,
and don't key anything off it yet **[O]**.

---

## 3. What a row means

```
row  =  one order line  =  (order, product, quantity)
order =  one delivery point's order for the day
        → customer, department, address, route, route ordering,
          accept-alternatives, comment
```

Verified over all 352 rows **[O]**:

- `Order ID → Customer`, `→ Department`, `→ Delivery street`,
  `→ Route nickname`, `→ Route ordering`, `→ Accept alternatives`,
  `→ Comment` — all single-valued. Everything except the product and quantity
  is an *order* attribute duplicated onto each line.
- `(Order ID, Product ID)` is unique — no duplicate lines to merge.
- **`Order ID → Supplier` does NOT hold.** 37 of 148 orders contain lines from
  both bakeries. Since `Position`/`Supplier` is only a label (§2), this costs
  nothing: an order stays one stop on one list, with mixed-supplier lines.
- `(Delivery street, Department)` does **not** identify an order either: 7
  address+department pairs carry 2–3 order IDs (e.g. `Street 84`
  with no department has three, under three different customer spellings).
  `Order ID` is the only reliable per-order key.

### `Route ordering = 0` means "not sequenced" **[O]**

37 rows / 20 customers / 9 of the 16 routes have `0`, and *multiple distinct
customers share it within one route* — route 5 has five, route 12 has four,
route 4 has three. Under "higher = later" five stops cannot all be first, so
`0` is a sentinel for "no position assigned", not position zero. Every other
value in the file is unique per stop within its route, apart from the same-site
ties in §4.

**Decided 2026-08-26:** `0` means unsequenced, and those stops print **after**
the sequenced ones, visibly flagged so the driver knows the order is theirs to
pick, not the system's.

---

## 4. Sorting for pick lists

The file arrives **product-major** and must be fully re-sorted **[O]**: rows
are grouped into 35 blocks of one `(Position, Product ID)` each, the blocks are
not even contiguous by supplier (`X-Bakehuset` → `X-Sandnes Bakeri` →
`X-Bakehuset` → …), and rows inside a block are in no route order at all.
There is no usable incoming sort — read everything, then sort.

The sort the printed list needs:

1. **Route nickname**, natural order (numeric prefix first, then string).
2. **Route ordering** ascending, with `0` sorted last rather than first
   (§3) and its stops marked as unsequenced on the printed list.
3. Tiebreak, so output is byte-identical run to run:
   **address → department → order ID**.

Step 3 is not optional, because equal non-zero orderings do occur — they mean
one stop with several delivery points:

- Route 9 @ `1400`: `Customer 064` `Department 18`
  (Street 64 **17**) and `Department 19` (gate **19**).
- Route 11 @ `1400`: 12 rows, **8 orders**, 3 spellings of one building
  (`Street 17` ×4, `Street 62` ×3, `Street 17,1. etg,
  høyre` ×1) under 2 customer spellings. Exact-string grouping therefore makes
  this three stops, not one — see `print-spec.md` §8 for why that matters.

Ties at `0` are just the unsequenced bucket and need the same tiebreak.

Routes in the sample, with stop counts:

| Route | Rows | Distinct customers | Orderings |
| --- | --- | --- | --- |
| 1 | 24 | 7 | 500 … 2700 |
| 2 | 13 | 6 | 0, 1100 … 1900 |
| 3 | 30 | 9 | 1100 … 2700 |
| 4 | 22 | 9 | 0 ×3, 800 … 2900 |
| 5 | 34 | 13 | 0 ×5, 300 … 2300 |
| 6 | 29 | 11 | 300 … 1800 |
| 7 | 17 | 7 | 400 … 2900 |
| 8 | 13 | 5 | 0 ×2, 100, 1100, 2100 |
| 9 | 25 | 11 | 0, 400 … 3000 |
| 10 | 20 | 7 | 200 … 4100 |
| 11 | 29 | 8 | 0, 400 … 3800 |
| 12 | 14 | 7 | 0 ×4, 500, 2300, 3400 |
| 13 | 26 | 4 | 200, 300, 400, 600 |
| 14 | 32 | 2 | 1390, 1800 |
| hau 1 | 12 | 6 | 0, 300 … 1600 |
| hau 2 | 12 | 9 | 0 ×2, 200 … 1050 |

---

## 5. Worked example — route 8

All 13 rows carrying `Route nickname = 8`, in the order they appear in the file
(the `row` number is the Excel row):

| row | Order ID | Qty | Product | Position | Customer | Address | Route ord | Alt |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 6 | 1000620185 | 10 | Barnehagebrødet - Oppskåret 750g | X-Bakehuset | Customer 005 | Street 05 | 2100 | 1 |
| 28 | 1000621240 | 1 | Dansk Rugbrød Hel Sandnes Bakeri | X-Sandnes Bakeri | Customer 024 | Street 24 | 100 | 1 |
| 50 | 1000622666 | 5 | Grovbrød M/sirup Oppdelt Sandnes Bakeri | X-Sandnes Bakeri | Customer 041 | Street 42 | 1100 | 1 |
| 63 | 1000621240 | 1 | Grovbrød M/sirup Oppdelt Sandnes Bakeri | X-Sandnes Bakeri | Customer 024 | Street 24 | 100 | 1 |
| 73 | 1000622461 | 4 | Grovbrød M/sirup Oppdelt Sandnes Bakeri | X-Sandnes Bakeri | Customer 054 | Street 55 | **0** | **0** |
| 81 | 1000621240 | 3 | Sekskornsbrød Oppdelt Sandnes Bakeri | X-Sandnes Bakeri | Customer 024 | Street 24 | 100 | 1 |
| 105 | 1000622554 | 2 | Sekskornsbrød Oppdelt Sandnes Bakeri | X-Sandnes Bakeri | Customer 070 | Street 71 | **0** | 1 |
| 140 | 1000622666 | 2 | Havrebrød Oppdelt Sandnes Bakeri | X-Sandnes Bakeri | Customer 041 | Street 42 | 1100 | 1 |
| 146 | 1000621240 | 2 | Havrebrød Oppdelt Sandnes Bakeri | X-Sandnes Bakeri | Customer 024 | Street 24 | 100 | 1 |
| 231 | 1000621240 | 4 | Ryfylkebrød Sandnes Oppdelt Bakeri | X-Sandnes Bakeri | Customer 024 | Street 24 | 100 | 1 |
| 261 | 1000622461 | 4 | Ryfylkebrød Sandnes Oppdelt Bakeri | X-Sandnes Bakeri | Customer 054 | Street 55 | **0** | **0** |
| 270 | 1000621240 | 3 | Sandnesbrød Oppdelt Sandnes Bakeri | X-Sandnes Bakeri | Customer 024 | Street 24 | 100 | 1 |
| 287 | 1000622666 | 2 | Sandnesbrød Oppdelt Sandnes Bakeri | X-Sandnes Bakeri | Customer 041 | Street 42 | 1100 | 1 |

Grouped by order and sorted by the §4 rule, route 8 becomes 5 stops:

```
ROUTE 8
  1. ord  100  Customer 024 — Street 24          [order 1000621240, subst. OK]
        1 × Dansk Rugbrød Hel Sandnes Bakeri
        1 × Grovbrød M/sirup Oppdelt Sandnes Bakeri
        3 × Sekskornsbrød Oppdelt Sandnes Bakeri
        2 × Havrebrød Oppdelt Sandnes Bakeri
        4 × Ryfylkebrød Sandnes Oppdelt Bakeri
        3 × Sandnesbrød Oppdelt Sandnes Bakeri
  2. ord 1100  Customer 041 — Street 42         [order 1000622666, subst. OK]
        5 × Grovbrød M/sirup Oppdelt Sandnes Bakeri
        2 × Havrebrød Oppdelt Sandnes Bakeri
        2 × Sandnesbrød Oppdelt Sandnes Bakeri
  3. ord 2100  Customer 005 — Street 05
                                                            [order 1000620185, subst. OK]
       10 × Barnehagebrødet - Oppskåret 750g   (X-Bakehuset)
  -- unsequenced (Route ordering = 0) --
  ?. Customer 054 — Street 55            [order 1000622461, NO SUBSTITUTES]
        4 × Grovbrød M/sirup Oppdelt Sandnes Bakeri
        4 × Ryfylkebrød Sandnes Oppdelt Bakeri
  ?. Customer 070 — Street 71 [order 1000622554, subst. OK]
        2 × Sekskornsbrød Oppdelt Sandnes Bakeri
```

Note the two unsequenced stops tiebreak by address (`Street 55` before
`Street 71`), and that route 8 mixes both pick positions.

---

## 6. Validation the loader should perform

Everything in §2–§4 marked **[O]** rests on one day's export. Rather than
assume, check on load and surface violations to the user — a silently
mis-sorted list is worse than a refused file:

1. **Shape** — sheet named `Data` exists; header row matches the 14 expected
   strings exactly, in order; a 15th unlabelled column is present.
   Any header change is a hard error.
2. **Required cells** — `A,B,C,D,E,G,H,J,L,M,N` non-empty on every data row.
   **Not `F`:** the freezer export leaves `Position` off 26 of its 231 rows,
   and reading it as required text refused the whole file at row 2.
3. **Types** — `A,B,C,M` numeric and integral; `N` boolean; `E,L` read as text
   regardless of how they look.
4. **Order consistency** — all lines sharing an `Order ID` agree on customer,
   department, address, route, route ordering, accept-alternatives and comment.
   Disagreement means the export changed shape; report the order ID.
5. **Route consistency** — one route per address. A repeated non-zero `Route
   ordering` within a route is *not* checked: it is how a multi-entrance site
   is written down, the stops sort adjacently anyway, and reporting it taught
   the reader that warnings are noise.
6. **Product consistency** — `Product ID` maps to one name, one SKU, one
   supplier across the file.
7. **Unknowns** — surface, don't reject: an unlabelled-column value other than
   `Stavanger`, and a supplier the list in question has not bought from
   before. The familiar suppliers are **per list** — two bakeries for bread,
   seven for freezer — because the bread pair raised nine notices on a clean
   freezer file, which teaches the reader that the step is noise.

   A route nickname is checked against **what the sorter can place**, not
   against a shape: `route::natural_key` has always handled a bare name, so
   `hau` and `Svg Employee` are not findings. Asking the sorter is the only
   version of this check whose reason survives reading.

---

## 7. Open questions

These change what gets printed and cannot be settled from the file. Decisions
already made about the printout live in [`print-layout.md`](print-layout.md).

1. **Column O / `hau`** — is `Stavanger` the depot, and are `hau 1`/`hau 2`
   Haugesund routes running out of it?
2. **Department** — its own line on the list, or folded into the customer name?
3. **Delivery date** — take it from the filename, or have the user enter it?

## 8. Re-deriving this

`tools/inspect_xlsx.py` (Python 3 stdlib only, no `openpyxl` needed) unpacks a
file and prints the header check of §6, the per-column profiles of §2, the
functional dependencies of §3, the route table of §4 and the product-block
order of §4:

```bash
python3 tools/inspect_xlsx.py "PSR-BREAD-2026-03-04-to-2026-03-04 (1).xlsx"
```

Run it against any new export before trusting this document for it.
