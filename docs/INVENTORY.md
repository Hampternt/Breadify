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

**The printed sheet** — `src/layout/`, `src/pdf.rs`, `assets/fonts/`.

- Eight embedded typefaces, text measured off the font tables rather than any
  toolkit's layout.
- A display list of positioned primitives is the only interface between
  deciding where things go and drawing them, so the paper and the app's
  preview cannot disagree.
- `breadify dump <route> --pdf <file>` draws one route; `breadify print --pdf
  <file>` draws the whole day. Every route starts a fresh page, no block or
  total is ever cut, and every page keeps 10 mm clear above its footer. The
  sample day is 26 sheets.

**The app window** — `src/app/`.

- A four-step wizard in the design's dark palette: Open, Check, Configure,
  Print. `breadify` with no arguments opens it.
- Step 1 takes a dropped file, a chosen one or a recent one, and reads it off
  the paint thread. Step 2 shows what was read and every finding computed from
  that file — structural errors block, observations warn.
- Step 3 offers what is actually the user's to decide: whether the order ID
  prints, how a refusal to substitute is marked, and the crate arithmetic —
  including how much room each bread takes. A sample block redraws from the
  same display list the PDF is drawn from.
- `breadify --screenshot <file.ppm>` renders a frame and writes it;
  `--step <n>` opens on a given step.

**Tools**

- [`../tools/inspect_xlsx.py`](../tools/inspect_xlsx.py) — stdlib-only dumper
  that re-derives the format doc from any export.

## In flight

🚧 **Pack 6 — Print** →
`manifests/` — not yet written.
Step 4's route table and both output paths.

## Not built yet

The four-step app window, printing and shipping — packs 4 to 7. Only the active pack is planned in detail.
