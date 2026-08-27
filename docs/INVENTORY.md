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

**Tools**

- [`../tools/inspect_xlsx.py`](../tools/inspect_xlsx.py) — stdlib-only dumper
  that re-derives the format doc from any export.

## In flight

🚧 **Pack 1 — Data spine** →
[`manifests/2026-08-27-pack-1-data-spine.md`](manifests/2026-08-27-pack-1-data-spine.md)
The export becomes a validated, sorted, fully-derived print model.

## Not built yet

The application itself. Packs 2–7 cover the printed sheet, pagination, the
four-step window, printing and shipping; only the active pack is planned in
detail.
