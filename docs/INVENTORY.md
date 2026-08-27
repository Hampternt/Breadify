# Breadify — inventory

What this repo is, at feature altitude. Implementation lives in the design
docs and the code; this file only says what exists and for whom.

## The product

A Rust desktop app for Windows and Linux that reads Matvare Expressen's daily
bread-order export and prints A4 picking lists. The picker packs bread into
crates and labels each crate; the driver delivers them in route order.

## What exists today

**Specifications** — complete and reconciled.

- [`excel-format.md`](excel-format.md) — the input file, verified against a
  real export, with the validation a loader must perform.
- [`print-layout.md`](print-layout.md) — the decision log, D1–D18.
- [`print-spec.md`](print-spec.md) — what the printed page must say and do.
- `Printer page formatting application/design_handoff_breadify/` — the design
  pass: high-fidelity geometry for the printed page and the four-step app
  window, plus HTML prototypes. Source of truth for appearance; `print-spec.md`
  overrides it on the unsequenced flag.

**The data spine** — `src/`, a Rust library and binary.

- Reads an export's one sheet into typed rows, and checks it against the
  invariants the printed pages rely on, reporting rather than refusing.
- Folds lines into orders, groups them into routes, and puts both into
  printing order — routes naturally, stops by delivery sequence with the
  unsequenced ones last.
- Derives what the page needs but the file does not carry: the delivery date
  from the filename, an order's crates, and each route's total with its
  ten-dots.
- `breadify dump <route> [export.xlsx]` prints one route in the shape of the
  worked examples.

**Tools**

- [`../tools/inspect_xlsx.py`](../tools/inspect_xlsx.py) — stdlib-only dumper
  that re-derives the format doc from any export.

## In flight

🚧 **Pack 2 — First sheet** →
[`manifests/2026-08-27-pack-2-first-sheet.md`](manifests/2026-08-27-pack-2-first-sheet.md)
Route 8 as a print-accurate A4 PDF.

## Not built yet

Pagination for a whole day, the four-step app window, printing and shipping —
packs 3 to 7. Only the active pack is planned in detail.
