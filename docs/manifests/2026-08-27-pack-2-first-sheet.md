# Pack 2 — First sheet

**Status:** all 9 items done — pack gate green, awaiting review.
**Container:** Breadify v1 (7 packs).
**Branch:** `pack-2-first-sheet`.

## Goal

Draw route 8 as a print-accurate A4 PDF. One sheet, no pagination — that is
pack 3 — but every millimetre of that sheet as the design handoff specifies
it.

**Observable when done:** `breadify dump 8 --pdf route-8.pdf` produces an A4
page that prints at 100 % and holds up beside the prototype's route 8 page:
masthead with the red 2.5 pt rule, legend strip, five stop blocks with tick
boxes and crate glyphs, A3 Transport's inverted badge and 5 pt left rule, the
unsequenced flag, and the route total with its single ten-dot.

## The architecture that matters

A **display list** is the only interface between layout and drawing. A Rust
paginator decides every position in millimetres and emits positioned
primitives; the PDF writer draws them and, later, the app's preview draws the
same list. Neither renderer measures or lays anything out, so they cannot
disagree.

Text measurement is headless, straight off the embedded font files, never via
a UI toolkit's layout — a toolkit rounds and pixel-snaps its geometry by
design, which would put the screen and the paper a fraction of a millimetre
apart on every run.

## Items

| # | Item | Done when |
| --- | --- | --- |
| 1 | Vendor the fonts | Archivo 800/900, Space Grotesk 400/500 and IBM Plex Mono 400–700 are in `assets/fonts/` as **static** TTFs with their OFL licences, and a test asserts each face's family and weight — variable files are rejected, since they silently embed their default instance |
| 2 | Headless text measurement | `text::width(run) -> Millimetres` measures a string in a given face and size straight from the font tables, and a test pins the longest product name at 11 pt against the 150 mm column |
| 3 | The display list | A `Page` of positioned primitives — text runs, rules, filled and outlined rectangles — in millimetres, with no drawing and no font handles |
| 4 | Page geometry | A4 at 210 × 297 mm with 9/8/8/5 mm margins as named constants, plus the mm→pt conversion at exactly one place in the codebase |
| 5 | Bread line and stop block | A stop emits its heading (customer, department box, crate glyphs, marker, order id) and its bread lines with tick boxes, supplier code, zebra tint and 0.4 pt rules |
| 6 | Masthead, page note, legend | The page furniture above the stops, including the 2.5 pt brand rule and the logo panel |
| 7 | Route total | The closing block: supplier columns, descending rows, ten-dots, 3 pt rule |
| 8 | The PDF backend | `printpdf` draws a display list onto an exact A4 MediaBox with the fonts subset and embedded, and `æ ø å` survive a round trip |
| 9 | `--pdf` on the CLI | `breadify dump 8 --pdf route-8.pdf` writes the file, and a test asserts the PDF's page box is A4 to within a rounding error |

<details>
<summary>Why the display list comes before the drawing</summary>

The one failure this pack has to design out is the page and its on-screen
preview being laid out twice by two different engines. If any preview code
ever decides *where* something goes, they drift. Putting the display list in
first, with the renderers unable to measure, makes that structural rather than
a matter of discipline.

</details>

## Ledger

- [x] 1 — Vendor the fonts · 8 static faces, ~1.2 MB, OFL
- [x] 2 — Headless text measurement · longest name 112.65 mm at 11 pt
- [x] 3 — The display list · text, rules, boxes; no font handles
- [x] 4 — Page geometry · A4, 9/8/8/5 mm, one mm↔pt conversion
- [x] 5 — Bread line and stop block · zebra, tick boxes, badge, crate glyphs
- [x] 6 — Masthead, page note, legend · brand rule, legend band, unsequenced flag
- [x] 7 — Route total · columns, ten-dots, 3 pt rule
- [x] 8 — The PDF backend · exact A4, fonts embedded, æøå round-trips
- [x] 9 — `--pdf` on the CLI · `dump 8 --pdf route-8.pdf`

**Deviations:**

- The fonts came from the projects' own upstream repositories rather than
  Google Fonts for two of the three families: Google ships Archivo and Space
  Grotesk only as variable files, which item 1 exists to keep out. Same
  typefaces, same OFL 1.1, official sources (Omnibus-Type/Archivo,
  floriankarsten/space-grotesk); IBM Plex Mono is Google's own static build.
- The heading needed a rule the design has no case for: where a customer name,
  its department box and its crates will not fit beside the marker and order
  id, the box and crates drop to a second line. Without it, OCAB AS on route 10
  — a 32-character name with a 38-character department — drew 8 mm off the
  right edge of the sheet. A test now asserts nothing on any of the 16 routes
  is drawn outside the page.
- The logo is a white wordmark on its dark panel rather than the supplied SVG;
  the PDF backend does not import vector artwork yet. The panel, its colour and
  its position are right, so this is a swap of one primitive when it lands.
- Item 2 turned up a wrong figure in the design handoff: the longest product
  name is 112.65 mm at 11 pt, not the ~148 mm the handoff calls the binding
  constraint. `print-spec.md` §9 now records the measurement and what follows
  from it.
**Pack gate:** not run.
