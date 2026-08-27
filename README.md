# Breadify

Turns the daily bread-order export into picking lists you can print — one route
per sheet, in the order the van gets emptied.

The bakeries deliver the morning's bread to the warehouse in one heap. Each
driver picks their own route out of it, packs and labels the crates, and drives
it. This prints the sheet they do both jobs from.

---

## Getting it

Download it from
[**Releases**](https://github.com/Hampternt/Breadify/releases/latest):

- **Windows** — `breadify.exe`
- **Linux** — `breadify`

There is nothing to install. Put the file wherever you like and open it. It
does not need the internet, and it never sends the order file anywhere.

On Windows, the first time you open it, SmartScreen may say the publisher is
unknown. Click **More info → Run anyway**. That warning is about the file not
being code-signed, not about anything it does.

---

## Using it, every morning

**1. Open the export.** Drag the `.xlsx` straight onto the window, or click
**Choose file**. Recently used files are listed so you can click one.

> Leave the filename alone. **The delivery date is read from the filename** —
> nothing inside the sheet carries one. `PSR-BREAD-2026-03-04-to-2026-03-04
> (1).xlsx` is fine; the `(1)` your browser adds is ignored.

**2. Read what it found.** The second step says how many rows, orders, routes
and breads it read, and lists anything worth knowing about the file:

| | |
| --- | --- |
| **note** | Just telling you. Nothing is wrong. |
| **warning** | Worth a look. The pages will still be right. |
| **blocking** | The file is not shaped the way it should be, and the pages would be wrong. Read it before you print. |

**3. Change anything that needs changing.** Most days, nothing. See
[Settings you only set once](#settings-you-only-set-once) below.

**4. Print.** Tick the routes you want — all of them, by default — and press
**Print**.

---

## Printing

**Print** writes the sheets to a PDF and opens it in whatever program opens
PDFs on your machine. **You pick the printer in that program**, the same way
you would print anything else. Breadify does not talk to printers itself and
does not have a printer list of its own — so if a printer works elsewhere on
that machine, it works here.

> **Print at actual size — 100 %, no scaling.**
>
> Every PDF viewer defaults to something like *Fit to printable area*, which
> shrinks the page by about 4 % and makes the text smaller than it was designed
> to be. Look for **Actual size**, **100 %**, or **Scale: None** in the print
> dialog before you send it.

**Export PDF** instead saves the file wherever you choose, without opening it —
for emailing it, or printing it from another machine.

If you have no printer on the machine you are testing on, **Print** still
works: the PDF opens, and the print dialog will offer *Microsoft Print to PDF*
or *Save as PDF*. That is enough to check the pages look right.

---

## What is on a sheet

Each route gets its own sheets. No sheet ever carries two routes, so you can
hand a driver their pages and nothing of anyone else's.

Down the page, top to bottom, is **the order you pack and the order you
deliver** — first stop at the top. The delivery position number itself is not
printed; the order of the blocks is the instruction.

Each **block** is one stop and one crate label:

- The **customer**, large. Under it, the **department** in a boxed `DPT` label
  where there is one — that is what goes on the crate, and one customer can
  have several.
- The **crate glyphs** on the right, one per crate the stop needs. A solid one
  is a full crate, a half-filled one is a small crate.
- **want substitute: true / false** — whether that customer will take a
  different bread when theirs is missing from the delivery. `FALSE` is set
  loud, because it changes what you do.
- Then a line per bread: a **tick box** for your pen, the **quantity**, the
  supplier code (**SB** Sandnes Bakeri, **BH** Bakehuset), and the bread's name
  exactly as the order has it. Two more boxes on the right — **M** for missing,
  **F** for fixed.

Stops the export gave no delivery position to are printed **last**, under a
flag saying the order is yours to choose. They are not missing; they are just
unplaced.

Each route ends with a **route total** — every bread that route needs, in one
list, split into a column per bakery. Hold it against what was actually
delivered that morning and it tells you whether the route can be picked before
you start packing it.

<p align="center">
  <img src="assets/nobread.jpg" alt="NO BREAD?" width="260">
  <br>
  <em>The route total, the morning it does not add up.</em>
</p>

---

## Settings you only set once

The third step, **Configure**.

**How much room each bread takes.** Crates hold ten of a normal loaf. Some
breads are not normal — a bag of rolls might take half a slot, something bulky
might take two. Click a bread in the list and pick a fraction: `1/4`, `1/3`,
`1/2`, `2/3`, `1`, `1 1/2`, `2`, `3`. Anything you change moves to the top of
the list under **NOT A WHOLE SLOT** so you can see at a glance what has been
set. Setting one back to `1` forgets it.

**How many a crate holds.** Ten in a large crate and five in a small one, out
of the box. Change them if that is not what your crates are.

Both of these are **remembered between runs** — set them once and they are
there tomorrow. They are kept in a small text file you can open and read:

- Windows — `%APPDATA%\breadify\crates.conf`
- Linux — `~/.config/breadify/crates.conf`

The Configure step shows the exact path once it has written it.

Everything else about the printed page is fixed on purpose, so that two people
printing the same day get the same paper. The one exception is the **order ID**,
which you can turn off if you find it noisy.

---

## Odds and ends

**Getting back to the start.** Click **01 Open** at the top at any time. You do
not have to close the program between files.

**Which version you have.** From a terminal, `breadify --version`. Worth
including if you report a problem.

**Nothing happens when I open a file.** Check it is the export as the system
produced it — one sheet named `Data`, with its original column headers. If a
column has been renamed or removed, the second step will say so rather than
guess.

---

## For developers

The reasoning, the input format, and every decision behind the printed page:

- [`docs/excel-format.md`](docs/excel-format.md) — the export's structure,
  verified against a real file, and the checks the loader performs.
- [`docs/print-spec.md`](docs/print-spec.md) — what the page must say and do.
- [`docs/print-layout.md`](docs/print-layout.md) — the decision log, D1 to D22.
- [`docs/INVENTORY.md`](docs/INVENTORY.md) — what exists, and what is left to a
  person.

It also does the same work from a terminal:

```
breadify print --pdf today.pdf          every route in the export in this folder
breadify dump 8                         one route, to read
breadify dump 8 --pdf route-8.pdf       one route, as sheets
breadify --help                         everything it accepts
```

Building it:

```
cargo build --release
./scripts/verify.sh     fmt, clippy, build, and the whole test suite
```

On Linux the window needs `libxkbcommon`, `libwayland` and GTK development
packages. Tagged pushes build a Linux binary and a Windows `.exe` in CI.

Three typefaces are embedded, all SIL Open Font License 1.1 — run
`breadify licences`.
