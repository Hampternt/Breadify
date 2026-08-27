# Breadify

Turns Matvare Expressen's daily bread-order export into printed A4 picking
lists — one route per sheet set, in the order the driver drives them.

Two people read the paper it prints: the **picker** at the bakery, packing
bread into crates and labelling each crate, and the **driver**, delivering
those crates in route order.

## Running it

```
breadify
```

opens the window. Drop today's export onto it, look at what it found, change
what you want to change, and print.

It also does the same work from a terminal:

```
breadify print --pdf today.pdf          every route in the export in this folder
breadify dump 8                         one route, to read
breadify dump 8 --pdf route-8.pdf       one route, as sheets
breadify --help                         everything it accepts
```

With no file named, it looks for a single `PSR-BREAD-*.xlsx` in the working
directory. **The delivery date comes from the filename** — no column in the
sheet carries one.

## On print day

1. Open the export.
2. Read the **Check** step. Warnings are worth a look; they do not stop a
   print. A blocking finding means the pages would be wrong.
3. **Print at actual size — 100 %, no scaling.** Every viewer defaults to
   "fit to printable area", which shrinks A4 by about 4 % and takes the body
   text under the size it was set at.

## What the printed page says

Each block is one stop — one customer, or one department of one customer — and
one crate label. It carries the quantity, the bread, the supplier code
(`SB` Sandnes Bakeri, `BH` Bakehuset), tick boxes for the picker, crate glyphs
for how many crates it needs, and whether that customer accepts a substitute
when a bread is sold out. Stops the export gave no position print last, under
a flag saying so. Each route ends with a total of every bread it needs.

## The documents

- [`docs/excel-format.md`](docs/excel-format.md) — the input file, verified
  against a real export, and the validation a loader must do.
- [`docs/print-spec.md`](docs/print-spec.md) — what the printed page must say
  and do.
- [`docs/print-layout.md`](docs/print-layout.md) — every decision behind it,
  D1 to D18, and why.
- [`docs/INVENTORY.md`](docs/INVENTORY.md) — what exists.

## Building it

```
cargo build --release
./scripts/verify.sh     fmt, clippy, build, and the whole test suite
```

On Linux the window needs `libxkbcommon`, `libwayland` and GTK development
packages. Tagged pushes build a Linux binary and a Windows `.exe` in CI.

Three typefaces are embedded, all SIL Open Font License 1.1 — run
`breadify licences`.
