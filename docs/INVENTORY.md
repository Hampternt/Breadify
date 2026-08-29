# Breadify — inventory

What this repo is, at feature altitude. Implementation lives in the design
docs and the code; this file only says what exists and for whom.

## The product

A Rust desktop app for Windows and Linux that reads Matvare Expressen's daily
bread-order export and prints A4 picking lists. The bakeries deliver to the
warehouse; each driver picks their own route out of that delivery, packs and
labels the crates, and drives it — one person, two jobs, the same sheet.

## What exists today

**Specifications** — complete and reconciled.

- [`excel-format.md`](excel-format.md) — the bread input file, verified
  against a real export, with the validation a loader must perform.
- [`freezer-format.md`](freezer-format.md) — the freezer input file, as
  deltas against the bread one.
- [`print-layout.md`](print-layout.md) — the decision log, D1–D22.
- [`freezer-list.md`](freezer-list.md) — the freezer version's decision log,
  F1–F5, and its open questions.
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
  bread sample day is 27 sheets; the freezer one is 21.

**The app window** — `src/app/`.

- A four-step wizard in the design's dark palette: Open, Check, Configure,
  Print. `breadify` with no arguments opens it.
- Step 1 takes a dropped file, a chosen one or a recent one, and reads it off
  the paint thread. Step 2 shows what was read and every finding computed from
  that file — structural errors block, observations warn.
- Step 3 offers what is actually the user's to decide: whether the order ID
  prints, how a refusal to substitute is marked, and the crate arithmetic —
  including how much room each bread takes, said in fractions of a slot on a
  list that stays quiet about the breads nobody has changed. Crate capacities
  and bread sizes are written to the OS config directory and come back next
  time; nothing else about a print is remembered. A sample block redraws from
  the same display list the PDF is drawn from.
- Step 4 lists every route with what it costs in paper, draws every sheet as a
  thumbnail, and either exports a PDF or hands it to the system to print.
- Two steps carry a joke at low opacity behind everything they draw: a bread
  roll at a computer behind Check, and Megamind asking `NO BREAD?` behind Open
  while nothing has been opened. They are why `image` is a dependency.
- `breadify --screenshot <file.ppm>` renders a frame and writes it;
  `--step <n>` opens on a given step; `--version` says which build it is.
- The Matvare Expressen symbol is rasterised from the same SVG the masthead
  draws — as the window's icon at startup, and as `assets/breadify.ico`, which
  the Windows build compiles into the executable so Explorer and the taskbar
  have a file icon. A test re-derives the `.ico` so the two cannot drift.

**Tools**

- [`../tools/inspect_xlsx.py`](../tools/inspect_xlsx.py) — stdlib-only dumper
  that re-derives the format doc from any export.
- [`../tools/anonymise_xlsx.py`](../tools/anonymise_xlsx.py) — replaces the
  people in an export with placeholders, keeping every structural property the
  tests rest on. The samples committed here have been through it; the repo is
  public and those columns described real customers.

## In flight

**The freezer list.** A second export kind — `PSR-FREEZER-*.xlsx`, the list
a packed freezer box is checked against rather than picked from. Both kinds
are recognised by filename; `Position` (a warehouse pick slot there, absent
for some products) reads as optional; validation knows each list's own
suppliers and route-name shapes; and the freezer sheet prints as the bread
sheet minus the picking machinery — no crate glyphs, each line a checked
box, a dotted note field and a missing box, a flat most-to-least total
closing each route, a legend keyed to the route's own suppliers, and a page
note that says `check list`. Decisions and what stays open:
[`freezer-list.md`](freezer-list.md).

Before that, Breadify **v1.1.0** was built and tagged: seven packs, then two passes
of changes from using it — the heading reshaped, crate sizes said in fractions
and kept between runs, the window icon, and a review pass over the lot.

What is deliberately not built, and why, is in
[`manifests/2026-08-27-pack-7-ship.md`](manifests/2026-08-27-pack-7-ship.md).

## Left to a person

Three things no test can settle:

- **The Windows build has never been run.** CI compiles it and attaches the
  `.exe`; the window, the file dialog and handing a PDF to the system's
  handler are unexercised there.

- **A day's sheets through the warehouse's own printer.** The bottom margin is
  5 mm and plenty of office printers cannot print that low; the fills on the
  no-substitutes badge and the crate glyphs need `print-color-adjust` to
  survive. Print one day and look at the paper.
- **The size of each bread**, in the Configure step. Every bread is a whole
  slot until someone who packs the crates clicks 1/2 or 2 against the ones
  that are not. Once. They are kept in `crates.conf` from then on. Only the active pack is planned in detail.
