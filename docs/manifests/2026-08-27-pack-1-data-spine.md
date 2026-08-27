# Pack 1 — Data spine

**Status:** in progress — items 1–2 of 9 done.
**Container:** Breadify v1 (7 packs — see [`../print-layout.md`](../print-layout.md) §Next).
**Branch:** `pack-1-data-spine`.

## Goal

Turn the `.xlsx` export into a validated, sorted, fully-derived print model.
Everything the printed page needs to *say* exists as data by the end of this
pack; nothing about how it *looks* is touched.

**Observable when done:** `cargo run -- dump 8` prints route 8's five stops,
their crate counts and the route total in the shape of the worked examples in
[`../print-spec.md`](../print-spec.md) §10, and `cargo test` proves those
figures against the real export.

## Items

| # | Item | Done when |
| --- | --- | --- |
| 1 | Repo skeleton and the two gates | `cargo build` succeeds on an empty `breadify` binary; `scripts/check.sh` (fmt + clippy `-D warnings` + build) and `scripts/verify.sh` (check + full tests) both exit 0 |
| 2 | xlsx loader keyed off cell references | All 352 rows load from the `Data` sheet with the unlabelled 15th column present, `Department`/`Comment` absent rather than blank, `Accept alternatives` as a bool, `Supplier SKU` as text, `Order ID` as `i64`; a test asserts the three row shapes 251 / 91 / 10 |
| 3 | Validation findings | The seven checks in [`../excel-format.md`](../excel-format.md) §6 run on load and produce a list of typed findings with severities, rather than panicking or silently passing |
| 4 | Order model | 352 lines fold into 148 orders, each single-valued on customer, department, address, route, ordering, accept-alternatives and comment; `(Order ID, Product ID)` unique; comments deduplicated; the 37 mixed-bakery orders stay whole |
| 5 | Delivery date from the filename | `PSR-BREAD-2026-03-04-to-2026-03-04 (1).xlsx` yields `2026-03-04` despite the ` (1)` suffix; an unparseable name asks for the date instead of failing |
| 6 | The two sorts | Routes come out `1..14, hau 1, hau 2` and never lexically; stops sort by `Route ordering` ascending with `0` last, tiebreaking address → department → order ID; route 8 reproduces `excel-format.md` §5 exactly, Street 55 before Street 71 |
| 7 | Crate arithmetic on slots | The D17 rule (`slots = ceil(Σ qty × modifier)`, then fewest containers at 10 and 5) gives route 8's five stops ■◪ / ■ / ■ / ■ / ◪ and Customer 012's nine departments 13 crates with every modifier at `1.0` |
| 8 | Route totals and ten-dots | Routes 5, 8, 11 and 14 reproduce their totals exactly — per-supplier type/unit splits, descending quantity with name tiebreak, correct `type`/`unit` pluralisation — and ten-dots sum `floor(line_qty / 10)` per line, so route 11's Kneippbrød shows **4**, not 6 |
| 9 | `dump` subcommand and golden tests | `cargo run -- dump <route>` prints any route in the worked-example shape; `cargo test` asserts per-route line counts 24, 13, 30, 22, 34, 29, 17, 13, 25, 20, 29, 14, 26, 32, 12, 12 |

<details>
<summary>Why this shape</summary>

The page is the deliverable, so the data it needs comes first and the window
comes last. Scoping the pack to "what route 8 needs" keeps it honest: route 8
is small (5 stops, 13 lines) but exercises an unsequenced tail, a
no-substitutes order, a mixed-bakery order and both crate sizes.

Every done-condition is checkable against the sample export rather than
against a document, because six hand-written figures in the docs and the
design handoff did not survive recomputation. Items 7 and 8 in particular
assert numbers this repo has already re-derived.

</details>

<details>
<summary>Risky items, reviewed individually</summary>

- **Item 6 (sorts)** — a lexical route sort is the single most likely bug in
  the whole app, and it fails silently and plausibly.
- **Item 7 (crates)** — D17's modifier mechanism is new and untested against
  real crates; the `1.0` baseline is the only verified case.

</details>

## Ledger

- [x] 1 — Repo skeleton and the two gates · `d9a5d00`
- [x] 2 — xlsx loader keyed off cell references · 7 tests green
- [ ] 3 — Validation findings
- [ ] 4 — Order model
- [ ] 5 — Delivery date from the filename
- [ ] 6 — The two sorts
- [ ] 7 — Crate arithmetic on slots
- [ ] 8 — Route totals and ten-dots
- [ ] 9 — `dump` subcommand and golden tests

**Deviations:** none yet.
**Pack gate:** not run.
