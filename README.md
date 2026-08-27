# Breadify

Takes a daily order export — the bread list or the freezer one — and turns it
into picking lists you can print. One route per sheet, in the order the van
gets emptied.

The bakeries drop the morning's bread at the warehouse in one heap. Each driver
picks their own route out of it, packs and labels the crates, and drives it.
This is the sheet they do both jobs from.

## Get it

Grab it from
[**Releases**](https://github.com/Hampternt/Breadify/releases/latest) —
`breadify.exe` for Windows, `breadify` for Linux.

Nothing to install. Put it wherever you like and open it. Windows may warn that
the publisher is unknown the first time: **More info → Run anyway**.

## Use it

1. **Drag today's export onto the window.** Keep the filename as it came —
   which list it is and which day it covers are both read from it, not from
   inside the sheet. One list at a time; do the freezer one after the bread.
2. **Glance at what it found.** Notes and warnings are just information. If
   something says *blocking*, read it before you print.
3. **Print.** All routes are ticked; untick any you don't want.

That opens the sheets as a PDF in whatever opens PDFs on your machine, and you
pick the printer there like anything else. **Print at 100 %, actual size** —
the default *Fit to page* shrinks everything by about 4 %. Or use **Export
PDF** to just save the file.

## Worth knowing

**Crate sizes.** On the Configure step you can say a bread takes half a slot,
or two. Click it and pick a fraction. Set once — it's remembered from then on.
Freezer sheets don't count crates: a box of lasagne isn't half a bread, so the
sheet says nothing rather than something wrong.

**Getting back.** Click **01 Open** at the top whenever you want a different
file.

<p align="center">
  <img src="assets/nobread.jpg" alt="NO BREAD?" width="280">
  <br>
  <em>Each route ends with a total of everything it needs. Hold it against what
  actually turned up.</em>
</p>

---

<details>
<summary>For developers</summary>

- [`docs/excel-format.md`](docs/excel-format.md) — the export's structure and
  the checks the loader performs.
- [`docs/print-spec.md`](docs/print-spec.md) — what the page must say and do.
- [`docs/print-layout.md`](docs/print-layout.md) — the decision log, D1 to D22.
- [`docs/INVENTORY.md`](docs/INVENTORY.md) — what exists, and what is left to a
  person.

Same work from a terminal:

```
breadify print --pdf today.pdf          every route in the export in this folder
breadify dump 8                         one route, to read
breadify dump 8 --pdf route-8.pdf       one route, as sheets
breadify --version                      which build this is
breadify --help                         everything it accepts
```

Building:

```
cargo build --release
./scripts/verify.sh     fmt, clippy, build, and the whole test suite
```

The window needs `libxkbcommon`, `libwayland` and GTK development packages on
Linux. Tagged pushes build both binaries in CI.

</details>

---

MIT licensed — © 2026 Jesper Løvland. See [`LICENSE`](LICENSE). The three
embedded typefaces stay under the SIL Open Font License 1.1 and the Matvare
Expressen wordmark is that company's own; `breadify licences` says so too.
