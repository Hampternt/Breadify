# Input Excel format — the freezer list

What the freezer export looks like, and exactly where it differs from the
bread export that [`excel-format.md`](excel-format.md) documents. Read that
file first; this one only carries the deltas and the freezer-specific
figures. Like there, every claim is tagged **[S]** (structural, true by
construction) or **[O]** (observed across one file from one day — check at
load, don't assume).

- **Sample:** `PSR-FREEZER-2026-01-23-to-2026-01-23 (1).xlsx` (23 565 bytes)
- **Derived:** 2026-08-27, with `tools/inspect_xlsx.py` and by unpacking the
  OOXML directly. 231 data rows, 115 orders, 113 products, 15 routes.

## 1. The container is the same file, byte for byte where it can be

Same producer (Excel `AppVersion 12.0000`), same single sheet named `Data`,
same 14 headers in `A1:N1`, same unlabelled column `O` reading `Stavanger`
on every row **[S]**. Of the ten parts in the zip, seven are hash-identical
to the bread sample's; only the shared strings, the sheet data and the
docProps dates differ **[S]**. Any reader that reads the bread file reads
this one.

The filename prefix is the *only* mark of which list a file is: the exporter
writes `PSR-FREEZER-<from>-to-<to>.xlsx` and everything after the prefix
follows the bread convention, browser ` (1)` suffix included **[S]**. The
delivery date is once again nowhere in the sheet — docProps says the file
was created the afternoon before (`2026-01-22T13:03:58Z`) **[S]**.

## 2. What differs

### `Position` is a warehouse pick slot, and it can be missing **[S]**

The one place the freezer file breaks a bread-era rule. Bread's `Position`
was the supplier respelled (`X-Sandnes Bakeri`, decision D4); the freezer
file carries a real location in the wholesaler's warehouse — 38 distinct
values, mostly `W-##-##`, plus loose ones like `U - frysevare` (in three
spellings), `W-NY VARE FRYS`, `A-Ny`, `W-16` and
`svg - Bestilling 3 Dager`.

- The slot belongs to the **product**: `Product ID → Position` holds with
  zero violations, while one slot holds up to 5 products, and
  `Supplier → Position` is far from 1:1 (asko spans 29 slots) **[O]**.
- **26 of 231 rows have no `Position` cell at all** — absent, not blank,
  like `Department` — covering 17 products that never appear with a slot
  anywhere (24 asko rows, 1 Holmens As, 1 ytterøy) **[O]**. The very first
  data row is one of them, which is why the pre-change loader refused the
  file at `cell F2`.

The loader therefore reads `Position` as `Option<String>` and carries it;
nothing prints it (yet — see `freezer-list.md`).

### Seven suppliers, mixed freely **[O]**

asko (147 rows), Fatland (31), Møremat (19), ytterøy (18), Holmens As (6),
Sørlandskjøtt AS (6), gabbas (4). Spelling is inconsistent in case, so
compare case-insensitively. 35 of 115 orders mix suppliers — up to **four**
in one order, where bread never passed two. As on the bread list, nothing
groups or totals by supplier; on the freezer list that is now a decision
(F4) as well as an observation.

### A wide, disjoint catalogue **[O]**

113 products against bread's 35, with **zero shared product IDs** and zero
shared names. Quantities are small — 1 to 20 per line, 521 units in the
whole day against bread's 1 581. SKUs are text here too: 108 all-digit,
5 suffixed (`9725holmens`, `2022138_gab`, …).

### Routes rhyme with bread's but are not the same routes **[O]**

Nicknames `1`–`13` (no `14`), a bare `hau` where bread has `hau 1`/`hau 2`,
and `Svg Employee` — a single line for a private person, evidently a staff
order riding along. Words-only nicknames are ordinary here, so the loader's
familiarity notice accepts them for freezer exports. 24 delivery addresses
appear in both samples but only 11 keep their route number across the two
lists — never assume a nickname means the same geography on both.

Route 13 is one customer (Customer 012, 8 department orders, one
address); route 6 is a single row.

## 3. What the freezer file taught us about the bread rules

Bread-derived **[O]** claims this file revises — the loader's checks already
tolerate all of them:

- **Lines per order runs to 8** (bread: 1–7). Histogram: 1×55, 2×28, 3×18,
  4×8, 5×4, 6×1, 8×1.
- **`Department` and `Comment` co-occur** on 4 rows. `excel-format.md`
  called their bread-file disjointness a coincidence at n=1; proven right.
  Seven cell-presence patterns here against bread's three, because
  `Position` can be absent too.
- **A comment can contain a literal newline** (one does) and **an email
  address** (one order asks for order copies by mail). Comments still
  repeat on every line of their order, and still deduplicate cleanly.
- **The character set grows past `æ ø å`**: one `é`
  (`Street 170`). Three product names carry internal
  double spaces — never whitespace-normalise a name.

Unchanged and re-verified on this file: order attributes are single-valued
per `Order ID`; `(Order ID, Product ID)` is unique; product ID ↔ name ↔ SKU
are mutually 1:1; one address never sits on two routes; `Route ordering = 0`
is the unsequenced sentinel (28 rows, 17 customers, 8 routes);
`Accept alternatives` is a real Excel boolean; rows arrive product-major
(113 contiguous blocks, none split, unsorted within) and must be fully
re-sorted.

One reading *not* carried over: bread's equal non-zero orderings meant one
site with several entrances. The freezer file's single tie (route 10 at
2700) is two unrelated customers on different streets. Ties still sort
adjacently under the D2 tiebreak, so nothing breaks — but the
"same ordering = same site" story is bread-only.

## 4. Validation

The loader runs the same checks on both lists (`src/validate.rs`); the only
kind-aware part is what counts as *familiar*: each list has its own known
suppliers, and words-only route nicknames are familiar on the freezer list
alone. A file whose name reveals no kind validates as bread. On this
sample, validating as a freezer export yields exactly the bread sample's
two notices — 28 unsequenced rows, and column O's missing header.

## 5. Re-deriving this

```bash
python3 tools/inspect_xlsx.py "PSR-FREEZER-2026-01-23-to-2026-01-23 (1).xlsx"
```

The same dumper works on both lists. Run it against any new export before
trusting this document for it.
