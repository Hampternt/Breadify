# Handoff: Breadify — printed pick list + formatting app

## Overview

Breadify is a Rust desktop app (Windows + Linux) that reads a daily bread-order
export (`.xlsx`) from Matvare Expressen and prints picking lists on A4. Two
people read the printed page: the **picker** at the bakery, packing bread into
crates and labelling each crate for a customer or department, and the **driver**,
delivering those crates in route order.

This handoff covers two designs:

1. **The printed page** — the A4 pick list. This is the primary deliverable.
2. **The app window** — a four-step wizard (Open → Check → Configure → Print)
   that loads the export, reports validation findings, lets the user choose
   fields and crate rules, and prints.

Source of truth for the data and the decisions already made:
`docs/excel-format.md`, `docs/print-layout.md` and `docs/print-spec.md` in the
Breadify repo. Where this document and those files disagree, **this document
wins** — several decisions changed during the design pass (see
[Decisions that changed](#decisions-that-changed)).

## About the design files

The files in `reference/` are **design references created in HTML** —
prototypes showing intended look and behaviour, not production code to copy.
They are self-contained: open them in a browser, no server needed.

The task is to **recreate these designs in the Rust app**. The printed page
should be produced by whatever the app already uses for printing (an HTML/CSS
print path, a PDF writer such as `printpdf`/`typst`, or direct GDI/Cairo
drawing) — the geometry below is given in millimetres and points so it
translates to any of them. The app window should be built with the project's
existing UI toolkit; if none is chosen yet, pick one appropriate for a
Windows + Linux Rust desktop app.

`source/` holds the editable originals. They are Design Components and need
the design-system files that live outside this bundle; use `reference/` to
look at the designs and `source/` only if you want to diff against future
revisions.

## Fidelity

**High-fidelity.** Final typography, sizes, rules, fills and copy. Every
measurement below is exact. The printed page in particular should be matched
precisely — it was sized against the worst-case routes in the real export and
has verified page buffers.

---

## Part 1 — The printed page

### Geometry

| Property | Value |
| --- | --- |
| Paper | A4 portrait, 210 × 297 mm |
| Margins | 9 mm top, 8 mm left/right, 5 mm bottom |
| Content width | 194 mm. The product-name column is 150 mm of it — enough for the longest name in the file (57 characters) on one line at 11 pt. |
| Body text minimum | 11 pt — never go below this. The sheet sets the product name at exactly 11 pt. |

**All type sizes below are given in points, and the prototype now uses absolute
units throughout** (`mm` for lengths, and `1 mm = 2.8346 pt` for type). The page
box is a literal `210mm × 297mm`. Nothing scales with the viewport — what you
measure on screen is what prints.

An earlier revision sized everything in `cqw` (percent of the page's content
box). Do not do that: it ties type size to page width, so changing the margins
scales the type with them and you can never buy column width. It also silently
rendered the body at ~9.5 pt.

### Page structure, top to bottom

1. **Masthead** — logo block, `Route`, route number, `continued` when applicable;
   date and page counter right-aligned. Bottom border **2.5 pt `#FF4F46`**
   (brand red), 5.9 mm below the masthead content.
2. **Page note line** — one sentence of page context, left; the
   want-substitute legend, right. Mono 8 pt `#6b6b6b`, never wraps.
3. **Legend strip** — tinted band explaining the tick boxes, the crate glyphs
   and the supplier codes.
4. **Stop blocks** — the picking work.
5. **Route total** — on the route's last page only.
6. **Footer** — pushed to the bottom of the sheet.

### Masthead

| Element | Spec |
| --- | --- |
| Logo block | `assets/matvare-expressen.svg`, 26.3 × 6.3 mm, on a `#141116` panel with 1.5 mm / 2.1 mm padding, 1.5 pt radius. The wordmark in the SVG is white, so it **must** sit on a dark panel — do not place it on white paper. |
| `Route` label | IBM Plex Mono 400, 9.2 pt, uppercase, letter-spacing 0.10em, `#555` |
| Route number | Archivo 900, 25 pt, line-height 0.85, letter-spacing −0.03em, `#000` |
| `continued` | Mono 400, 8.6 pt, uppercase, letter-spacing 0.06em, `#555` — only on second and later pages of a route |
| Date | Mono 500, 10.4 pt, ISO format (`2026-03-04`) |
| Page counter | Mono 400, 8.3 pt, uppercase, letter-spacing 0.08em, `#555`. Format: `Page 1 of 2 · 13 stops · 34 lines` |
| Rule | 2.5 pt solid `#FF4F46` |

### Legend strip

Background `#F4F4F4`, 0.5 pt `#C9C9C9` top and bottom borders, padding 0.9 mm
× 1.3 mm, mono 8 pt `#3d3d3d`, items spaced 4 mm apart, never wraps. Contents,
left to right:

- `Boxes` — bold, uppercase, letter-spacing 0.08em, `#000`
- A tick-box swatch containing `P`, then **P**icked
- A swatch containing `M`, then **M**issing
- A swatch containing `F`, then **F**ixed
- `Crates` — bold uppercase — then a solid glyph and `10`, a half-filled glyph and `5`
- Right-aligned: **SB** Sandnes Bakeri · **BH** Bakehuset

The initial of each word is Archivo/Space Grotesk **bold at 9.8 pt** while the
rest of the word stays at 8 pt, matching the letter shown inside the box. The
swatches are 4 × 4 mm, 1 pt `#3d3d3d` outline, and their letters sit at **30 %
opacity**.

### A stop block

One block = one order = one crate.

| Element | Spec |
| --- | --- |
| Block rule | `border-top: 1.25 pt solid #000`; padding 1.7 mm top, 1.2 mm bottom |
| Customer name | Archivo 800, **14 pt**, line-height 1.12, letter-spacing −0.02em, `#000`. Verbatim from the file — mixed case and ALL-CAPS both occur. Never wraps in practice (longest is 46 characters at 62 mm of 182 mm). |
| Crate glyphs | Immediately right of the name, 1.3 mm gap — see [Crate indicator](#crate-indicator) |
| Marker + order ID | Right-aligned on the same line, 2.3 mm apart |
| Bread lines | Below the heading, 0.7 mm gap |

**Heading row layout:** flex row, `justify-content: flex-start`, baseline
alignment, 4.6 mm gap; the marker/order-ID group is pushed right with
`margin-left: auto`. The name must sit hard left — the crate glyphs follow it,
not the page edge.

### A bread line

Flex row, baseline aligned, 3.6 mm gap, padding 0.36 mm × 1.3 mm with a
−1.3 mm horizontal margin so the zebra tint bleeds slightly past the text.
`border-bottom: 0.4 pt solid #E2E2E2`. **Every second line in a block** carries
`background: #F1F1F1` — the stripe restarts at each block so it never straddles
a heading.

Children, left to right:

| # | Element | Spec |
| --- | --- | --- |
| 1 | Picked box | 4.6 × 4.6 mm, 1 pt `#3d3d3d`, 1 pt radius, centred `P` in mono 700 8.9 pt at **16 % opacity** |
| 2 | Quantity | Mono 600, **13.1 pt**, right-aligned in a 10.1 mm column |
| 3 | Supplier code | Mono 700, **10.7 pt**, `#000`, letter-spacing 0.02em, left-aligned in an 8 mm column. `SB` or `BH` |
| 4 | Product name | Space Grotesk 400, **11 pt**, line-height 1.22, `#000`, fills the remaining width. **Verbatim from the file** — do not strip the trailing bakery name, do not truncate. Longest in the sample is 57 characters and fits on one line. |
| 5 | Missing + Fixed boxes | Two boxes as #1, 1.5 mm apart, containing `M` and `F` |
| — | Supplier name | **Not printed on the row.** The bold `SB` / `BH` code carries the supplier; the legend strip and the route totals spell both bakeries out in full. Dropping the repeated name is what paid for the 11 pt body — it freed ~20 mm of the product column. |

The three boxes are the packer's marks: **P** ticked when that bread is
collected, **M** when it is short, **F** when a shortfall has been resolved.
Nothing is written on the row itself — the legend carries the meaning.

### Departments and grouped stops

Where several orders share one delivery address, they print under **one address
heading** with each order as a sub-block. This is what keeps big sites to one
page (Customer 012 is 9 orders at one address; Customer 017 is 8).

**Group heading:** customer name (Archivo 800, 14 pt) + address (mono 8.3 pt
`#6b6b6b`) on one line, with a right-aligned note in mono 8 pt uppercase
letter-spacing 0.06em `#8a8a8a` — e.g. `one stop · nine crates`,
`one stop · 8 crates · 3 address spellings`. Padding-bottom 0.5 mm.

**Sub-block:** `border-top: 0.75 pt solid #B8B8B8`, padding 0.55 mm top,
0.4 mm bottom. **No left indent** — sub-blocks align flush with every other
customer name.

**Department label:** an outlined box — 1.5 pt `#000` border, padding
0.34 × 1.3 mm — containing a bold mono `DPT` tag (8.6 pt, letter-spacing
0.08em, 1.3 mm right padding, 1 pt right divider) followed by the department
name in Archivo 800 **11.9 pt**, letter-spacing −0.015em. The box is the crate
label; it is what gets copied onto the crate.

**An order with no department shows no box at all** — the empty left side is
the signal. Those orders are told apart by their order ID.

### The want-substitute marker

`Accept alternatives` belongs to the order, not the customer, and it changes
what the picker does when a bread is sold out. Deliberately asymmetric: 90 % of
orders accept substitutes, so that state is quiet and the 10 % is loud.

| State | Rendering |
| --- | --- |
| `true` | `want substitute: true` — mono 500, 9.2 pt, `#3d3d3d`, in the heading's right-hand group |
| `false` | `WANT SUBSTITUTE: FALSE` — Archivo 800, 10.4 pt, **white on solid `#000`**, padding 0.7 × 1.6 mm, in the same right-hand group; **and** the whole block gets `border-left: 5 pt solid #000` with 2.3 mm of left padding |

Two independent channels (an inverted badge and a heavy rule) so it survives a
mono laser printer and a photocopy. Colour may reinforce but must never carry
it alone. Set `print-color-adjust: exact` on the badge so the fill is not
dropped.

### Order ID

Mono 400, **8.3 pt**, `#9c9c9c`, right-hand group, after the marker. Present
for lookups, deliberately the quietest thing on the page. It is the only thing
distinguishing several orders at one address with no department, so it must not
go below ~8 pt.

### Crate indicator

Right of every customer and department name: how many crates that block's bread
needs. Crates hold 10 or 5.

**Rule** — fewest containers:

```
tens = floor(units / 10)
rem  = units % 10
rem == 0        -> tens × crate-of-10
rem in 1..5     -> tens × crate-of-10  +  1 × crate-of-5
rem in 6..9     -> (tens + 1) × crate-of-10
```

`units` is the sum of `Quantity` across every line of that order.

**Glyphs** — both are the same 7.1 × 4.1 mm outlined rectangle (1 pt `#000`,
0.5 pt radius), 1.1 mm apart, so the row of glyphs stays aligned:

- **Crate of 10** — filled solid `#4a4a4a`
- **Crate of 5** — outline only, with the **bottom 50 % filled** `#4a4a4a`

Ink level reads as capacity. Set `print-color-adjust: exact` on both.

**Small items.** Some breads are physically smaller, so a crate holds more of
them. The app carries the configuration for this (see
[Crate rules](#step-3--configure)); the crate arithmetic in the app should
apply it. The prototype's glyphs are computed from a flat unit count, so
changing the small-items list in the mock does not redraw them.

### Route total

Closes the route's last page. Answers "how much of each bread does this whole
route need" — a cross-check for the bakery, not picking work, so no tick boxes.

| Element | Spec |
| --- | --- |
| Container | `margin-top: 5 mm`, `border-top: 3 pt solid #000`, padding-top 2.1 mm, never splits across pages |
| Title | `Route 5 total` — Archivo 800, 12.5 pt, letter-spacing −0.02em, never wraps |
| Meta line 1 | `14 bread types · 117 units · most to least` — mono 8.3 pt `#6b6b6b` |
| Meta line 2 | A 2.5 mm filled dot, then `one full ten inside a single order — 4 on this route` — mono 7.7 pt `#3d3d3d` |
| Grid | Two columns, 8.4 mm gap. **One supplier per column**, Sandnes Bakeri first. |
| Column head | Bold mono code (10.7 pt) + supplier name (Archivo 800, 10.4 pt) + right-aligned `10 types · 85 units` (mono 500, 8.9 pt `#555`); `border-bottom: 1.25 pt solid #000` |
| Row | Quantity (mono 600, 11 pt, right-aligned, 8.4 mm column) + name (Space Grotesk 400, 9.2 pt, `#222`) + right-aligned dot column (18 mm); `border-bottom: 0.4 pt solid #E2E2E2` |
| Order | **Descending quantity**, ties broken by name so two runs of one file print identically |

**The ten-dots.** For each bread type, one 2.7 mm filled black dot per full ten
**inside a single order** — `sum over lines of floor(line_qty / 10)`, not
`floor(total / 10)`. A bread with no order reaching ten shows nothing. This
tells the picker how many full trays can be pulled whole. Worked example from
the sample file, route 11 Kneippbrød: line quantities 2, 7, 4, 10, 11, 20, 6, 8
→ 68 units total, dots = 0+0+0+1+1+2+0+0 = **4**.

Pluralise `type`/`unit` correctly — route 8's Bakehuset column is `1 type · 10 units`.

### Footer

`margin-top: auto` so it always sits at the bottom of the sheet.
`border-top: 0.5 pt solid #C9C9C9`, padding-top 1.9 mm, mono 400 7.4 pt
`#8a8a8a`. Left: continuation state (`Route 5 continues on page 2`,
`Route 14 — end of route`). Right: `PSR-BREAD-2026-03-04 · Matvare Expressen`.

### Pagination rules

1. **One route per sheet set.** A route always starts a fresh page, even if the
   previous route ended a third of the way down. No page ever carries two routes.
2. **A stop block never splits across a page break.** If it does not fit, the
   whole block moves to the next page. Grouped stops may split *between*
   sub-blocks, repeating the address heading with a `same stop` note.
3. **The route total never splits.** If it does not fit under the last stop, it
   moves to a new page — rebalance the stops so that page is not nearly empty.
4. Multi-page routes are normal. Legibility first, paper second.

### Verified page budgets

Measured at the true A4 box against the real export. Every page must keep
**≥ 10 mm** between the last content and the footer; below that, a printer with
different text metrics silently clips a row.

| Sheet | Content | Buffer |
| --- | --- | --- |
| Route 5 page 1 | 8 stops, 24 lines | 53 mm |
| Route 5 page 2 | 5 stops, 10 lines, route total | 46 mm |
| Route 11 page 1 | 4 stops incl. an 8-order group, 22 lines | 28 mm |
| Route 11 page 2 | 3 stops, 7 lines, route total | 94 mm |
| Route 14 page 1 | 7 crates, 22 lines | 62 mm |
| Route 14 page 2 | 3 crates, 10 lines, route total | 94 mm |
| Route 8 | 5 stops, 13 lines, route total | 67 mm |

Across all 16 routes in the sample: **24 sheets**. Without the route totals it
is 19; without stop grouping and with a three-line heading per stop it was 27.

The hard limit, so you know where the floor is: 34 lines plus 13 stop headings
is roughly 326 mm of content at 11 pt. No layout fits that on a 297 mm sheet —
route 5 needs two pages at any density that respects the 11 pt minimum.

**Width is the binding constraint, not height.** At 11 pt the longest product
name needs about 148 mm and the column is 150 mm. That is why the row carries a
supplier *code* rather than a name, and why margins are 8 mm rather than the
14 mm a document would normally use. If you add another column to the bread
line, the 11 pt body is the thing that has to give.

---

## Part 2 — The app window

A four-step wizard. Dark theme throughout (see [Design tokens](#design-tokens)).

### Window

| Property | Value |
| --- | --- |
| Size | 1280 × 864 px, 1 px `#262232` border, 10 px radius |
| Title bar | 40 px, `rgba(14,12,20,.82)`, 1 px bottom border `rgba(242,238,248,.07)`. Logo (83 × 20) left, then the loaded filename in mono 11 px `#5F5876`. Right: a `Search` button with a `K` keycap, then minimise / maximise / close. |
| Step rail | Four equal tabs, `#0E0C14`, 1 px bottom border. Each: mono 11 px number (letter-spacing 0.10em), Archivo 700 15 px label, mono 11 px note right-aligned. Active tab gets `#17141F` background, a 2 px `#B48EF7` bottom border, `#F2EEF8` label and `#CBB0FF` number. Tabs are clickable — the wizard is not one-way. |
| Action bar | 56 px, `#0E0C14`, 1 px top border. `Back` (ghost, disabled on step 1) and a contextual hint on the left; the primary action right. On the Print step it also carries a printer select and `Export PDF`. |

Primary button label and hint per step:

| Step | Primary | Hint |
| --- | --- | --- |
| 01 Open | `Read the file` | One sheet named Data. 35 MB or less. |
| 02 Check | `Continue anyway` | Two warnings — nothing that stops a print. |
| 03 Configure | `Preview sheets` | Six fields always print; the rest are yours. |
| 04 Print | `Print 24 sheets` | One route per sheet set. Orders at one address share a heading. |

### Step 1 — Open

Two columns: a drop zone (`1fr`) and a `Recent` rail (320 px).

**Drop zone** — full-bleed panel, 1 px dashed `#322C42`, 12 px radius,
`#0E0C14` with a 32 px blueprint grid at 4.5 % white. Centred: a
`file-spreadsheet` icon (32 px), `Drop today's export here` in Archivo 800
26 px, then a mono 13 px `#8D87A0` line naming the expected shape
(`PSR-BREAD-<from>-to-<to>.xlsx — one sheet named Data, 14 headers plus one
unlabelled column. The delivery date is read from the filename.`), then a
primary `Choose file` button and a `Ctrl O` keycap pair.

**Recent rail** — cards on `#17141F`, 1 px `rgba(242,238,248,.07)`, 8 px
radius: ISO date in mono 500 13 px, then `352 rows · 148 orders · 16 routes` in
mono 11 px `#5F5876`.

### Step 2 — Check

**What was read** — five stat cards in a `repeat(5, 1fr)` grid: value in mono
500 26 px `#F2EEF8`, label in mono 11 px uppercase letter-spacing 0.08em
`#5F5876`. From the sample: `352 rows`, `148 orders`, `16 routes`,
`35 products`, `03-04 date from filename`.

**Checks** — a header row (`6 passed · 2 warnings · 2 notes`) then one card per
finding: a 104 px badge column (`warning` amber, `note` azure, both with a
status dot), then a title in Space Grotesk 500 14 px `#F2EEF8` and a detail
line in mono 12.5 px `#8D87A0`. Left border 2 px in the status colour.

The four findings, verbatim:

- **warning** — *Route ordering 1400 repeats on route 9.* Street 64 and 19 — one stop, two entrances. Both print at that
  position, tiebroken by address.
- **warning** — *Route 11 @ 1400 — three spellings of one building.*
  Street 17 · Street 62 · Street 112.
  Printed as five separate stops under two customer names.
- **note** — *37 rows have Route ordering 0.* 20 customers across 9 routes.
  Printed after the sequenced stops.
- **note** — *Column O carries no header.* Read positionally. Value is
  Stavanger on all 352 rows; nothing is keyed off it.

Below them, a summary strip: `Sheet shape, required cells, column types, order
consistency, product consistency, one route per address — all passed.`

**Severity policy: warn, do not refuse.** Hard structural errors (missing
`Data` sheet, changed header row) should still block, but everything derived
from a single day's observation is a warning the user can print past.

### Step 3 — Configure

Three columns: `304px 1fr 356px`.

**Left rail — fields.**
*Always printed* (six rows, each with a lock icon, mono 12.5 px, on a card):
Quantity, Product Name, Customer, Department, Accept alternatives, Route nickname.
*Optional columns* as togglable chips: Route totals, Supplier, Order ID (on by
default), then Street, Comment, Date, Product ID, SKU, Position, Region.
*Never printed*: `Route ordering`, struck through, with the note
`the list order says it`.

**Middle column.**
- *Page* — two selects: paper (A4 / Letter) and label language.
- *Density* — three cards: `Roomy` (11.5 pt · 29 sheets), `Grouped`
  (11 pt, stops grouped · 24 sheets, **default**), `Paper-saving`
  (10 pt · 20 sheets). Selected card gets `rgba(180,142,247,.14)` background
  and a `rgba(180,142,247,.45)` border.
- *No-substitutes marker* — three cards, each previewing the treatment on a
  white swatch: `Inverted badge` (default), `Heavy left rule`, `Word only`.
- *Crates* — large-crate capacity, small-crate capacity, and how many small
  breads share a slot; then **all 35 bread types as togglable chips**, marking
  which count as small. Fourteen are pre-marked: the rolls, horn, wienerbrød,
  skoleboller, frøhorn, ostebrød, energi stykke and the 4pk/6pk/10pk packs.
  A readout shows `14 of 35 marked small`. This list feeds the crate
  arithmetic on the printed page.
- *Fixed rules* — locked, non-editable statements of the pagination rules.

**Right rail — sample block.** A white card showing one real stop block at
print size, live-updating as fields are toggled. Below it, a note explaining
the department-as-crate-label decision.

Chips are DS `Tag` components in `interactive` mode; each must sit in a
non-shrinking wrapper or it will be squeezed below its own text width.

### Step 4 — Print

Two columns: a 420 px route table and a preview grid.

**Route table** — header (`16 of 16 · 24 sheets`), then a
`24px 1fr 56px 56px` grid per row: a checkbox (violet fill + `✓` when
selected), the route name in mono 500 14 px, lines and sheets right-aligned in
mono 12.5 px. Deselected rows drop to `#07060B` background and `#5F5876` text.
Route order is natural: `1 … 14`, then `hau 1`, `hau 2` — **never** lexical,
which would give `1, 10, 11, … 2, 3`.

Per-route figures (lines / sheets at the default density): 1: 24/2 · 2: 13/1 ·
3: 30/2 · 4: 22/2 · 5: 34/2 · 6: 29/2 · 7: 17/1 · 8: 13/1 · 9: 25/2 ·
10: 20/1 · 11: 29/2 · 12: 14/1 · 13: 26/1 · 14: 32/2 · hau 1: 12/1 ·
hau 2: 12/1.

**Preview grid** — A4 thumbnails, four per row, `aspect-ratio: 210/297`, white
on `#07060B`, 0 2px 10px rgba(0,0,0,.5) shadow, with a violet outline on the
current sheet. Each carries a caption `route 5` / `1/2`. Captions are derived
from the same route data as the table so `n/m` can never disagree with a row.

---

## Interactions & behaviour

- **Step navigation** — primary button advances, `Back` retreats, and the rail
  tabs jump directly. No step is gated.
- **Field, crate and route toggles** — immediate, no apply step. Toggling a
  field updates the sample block; toggling routes updates the sheet total in
  the header, the primary button label and the preview grid.
- **Keyboard** — `Ctrl/⌘ K` opens a command palette, `?` shows the shortcut
  overlay, `/` focuses the current filter, `Esc` closes the topmost overlay,
  `↑ ↓` move a selection, `↵` runs it. Single-letter shortcuts never fire while
  a text field has focus. Every shortcut must be visible somewhere on screen.
- **Motion** — 130 ms for controls, 190 ms for surfaces, 280 ms for overlays,
  all `cubic-bezier(.2,.8,.3,1)`. Press state translates 1 px down, never
  scales. `prefers-reduced-motion` zeroes every duration.
- **Focus** — one ring everywhere: 2 px page colour then 2 px `#B48EF7`. Never
  removed, never restyled per control.
- **Respond first, finish after** — parsing 352 rows is fast, but keep the
  window responsive: read and validate off the UI thread, and never make the
  user wait for work that does not change what they see next.

## State

| State | Type | Notes |
| --- | --- | --- |
| `step` | 0–3 | Which wizard step is showing |
| `file` | path + parsed rows | Source of every derived figure |
| `findings` | list | Validation results, each with severity |
| `fields` | map<field, bool> | Optional columns; the six mandatory fields are not in it |
| `density` | enum | roomy / grouped / paper-saving |
| `marker` | enum | badge / rule / word |
| `small` | map<product, bool> | Which bread types count as small |
| `crateCaps` | {large, small, perSlot} | Crate arithmetic inputs |
| `selectedRoutes` | set | Which routes to print |
| `previewSheet` | int | Current thumbnail |

Derived, never stored: sheet counts, crate glyphs, route totals, ten-dots.
All of them fall out of the parsed rows plus `density`, `small` and `crateCaps`.

## Data rules the app must honour

From `docs/excel-format.md`, and all of it matters for correctness:

- **Sheet** — exactly one, named `Data`. 14 headers in `A1:N1` plus a
  **15th unlabelled column** with data in every row. A reader that derives its
  width from the header row silently drops it.
- **Empty cells are absent, not blank.** `Department` and `Comment` have no
  `<c>` element at all. Key off the cell reference, never off cell count.
- **`Route nickname` is a string** even when it looks like a number. Sort
  naturally: integer-prefixed nicknames first by that integer, then the rest
  alphabetically. Lexical sorting is the single most likely bug in the app.
- **`Accept alternatives` is a real Excel boolean** (`t="b"`), not an integer.
- **`Supplier SKU` is text** — `107_san`, `10022_bhb`, bare `115`,
  `21061bhb`. Parsing it as a number loses most of the catalogue.
- **`Order ID` is a 10-digit number** stored as a float; format as an integer
  and hold it in `i64`.
- **`Customer` is not a key.** Group by address. One site appears under
  several spellings; one name appears on two routes.
- **`Position` is just the supplier again** — carry it, group nothing by it.
  37 of 148 orders mix both bakeries and stay one stop on one list.
- **`Route ordering` is an opaque sortable integer**, gap-numbered, comparable
  only within a route. `0` means unsequenced.
- **The delivery date is not in the sheet.** Parse it from the filename, which
  may carry a ` (1)` browser-download suffix.
- **Comments repeat on every line of an order** — deduplicate before printing.

**Sort for the printed list:** route (natural) → `Route ordering` ascending
with `0` last → address → department → order ID. The final tiebreak is not
optional: equal non-zero orderings occur and mean one stop with several
delivery points.

## Decisions that changed

These override `docs/print-layout.md` and `docs/print-spec.md`:

| Was | Now |
| --- | --- |
| **D3** — unsequenced stops print last **with a visible flag** | The flag is removed. Those stops simply print last. Consider striking D3 in the decision log so the docs and the sheet agree — as it stands the driver cannot tell "no position assigned" from "last delivery". |
| **§9** — no totals, no summaries | Each route ends with a **Route total** block, split by supplier, ordered most to least, with ten-dots. |
| **D5** — department shown on the block | Department is the **loud** element, in an outlined `DPT` box; the customer heads the group. |
| **§5** — wording free (`alt: yes/no`) | `want substitute: true` / `WANT SUBSTITUTE: FALSE`. English labels, Norwegian data verbatim. |
| Open Q3 — printer unknown | Mono-first, colour may reinforce but never carry meaning. |
| Open Q4 — strip the bakery name from product names? | **Print verbatim.** A bold `SB` / `BH` code carries the supplier instead. |
| Open Q1 — which fields are configurable | Any of the 15 columns, on top of six mandatory ones. |
| Not previously specified | Crate indicators, per-line tick boxes, a small-items list. |

## Design tokens

**Print** — the sheet is black on white; these are the only values used.

| Token | Value | Use |
| --- | --- | --- |
| `#000` | black | Text, block rules, badge fill, department box, crate outline |
| `#141116` | near-black | Logo panel |
| `#222` | | Route-total product names |
| `#3d3d3d` | | Quiet marker text, tick-box outline |
| `#4a4a4a` | dark grey | Crate glyph fill |
| `#555` | | Masthead labels, column subtotals |
| `#6b6b6b` | | Page note line, group address |
| `#8a8a8a` | | Supplier name, group note, footer |
| `#9c9c9c` | | Order ID |
| `#B8B8B8` | | Sub-block rule (0.75 pt) |
| `#C9C9C9` | | Footer rule (0.5 pt) |
| `#E2E2E2` | | Bread-line rule (0.4 pt) |
| `#F1F1F1` | | Zebra row tint |
| `#F4F4F4` | | Legend strip |
| `#FF4F46` | brand red | Masthead rule only. From the Matvare Expressen logo. |

Rule weights: 3 pt (route total) · 2.5 pt (masthead) · 1.5 pt (department box) ·
1.25 pt (block) · 1 pt (tick box, crate) · 0.75 pt (sub-block) ·
0.5 pt (footer) · 0.4 pt (bread line). The no-substitutes left rule is 5 pt.

Print type scale, in points: 25 (route number) · 14 (customer) ·
13.1 (quantity) · 12.5 (route-total title) · 11.9 (department) ·
11 (product name, route-total quantity) · 10.7 (supplier code) ·
10.4 (date, badge, supplier column head) · 9.8 (legend initials) ·
9.2 (marker, route label, route-total name) · 8.9 (box letters) ·
8.6 (DPT tag, continued) · 8.3 (page counter, order ID, group address) ·
8 (supplier name, legend) · 7.7 (dot note) · 7.4 (footer).

**Screen** — the app uses the Hampter design system's dark palette.

| Token | Value |
| --- | --- |
| Page / void | `#0B0910` / `#07060B` |
| Raised / card / chip | `#0E0C14` / `#17141F` / `#262232` |
| Text strong / body / muted / faint | `#F2EEF8` / `#CDC6DD` / `#8D87A0` / `#5F5876` |
| Accent (violet) | `#B48EF7`, hover `#CBB0FF`, press `#8F6FD8` |
| Accent tint / border | `rgba(180,142,247,.14)` / `rgba(180,142,247,.45)` |
| Borders subtle / default / strong | `rgba(242,238,248,.07)` / `#262232` / `#322C42` |
| Status success / warning / danger / info | `#4FD6A8` / `#FFB570` / `#F7768E` / `#7AA2F7` |

Spacing: 2, 4, 6, 8, 12, 16, 20, 24, 32, 40, 48, 64 px. Controls snap to
28 / 34 / 42 px. Radii: 3 px keycaps and badges, 5 px buttons and inputs,
8 px cards, 12 px dialogs, 10 px the window.

## Typography

| Face | Weights | Use |
| --- | --- | --- |
| **Archivo** | 800, 900 | Headings, customer names, department labels, the route number, badges. Tracking −0.02 to −0.035em. |
| **Space Grotesk** | 400, 500 | Product names, prose, UI labels. |
| **IBM Plex Mono** | 400, 500, 600, 700 | Anything a machine produced: quantities, dates, order IDs, supplier codes, keycaps, and 8 pt uppercase micro-labels tracked to 0.10em. Never used for paragraphs. |

Archivo and Space Grotesk are self-hosted woff2 in the design system
(`_ds/.../assets/fonts/`). IBM Plex Mono comes from Google Fonts and is a
flagged substitution — swap or self-host it freely.

## Assets

| Asset | Source | Notes |
| --- | --- | --- |
| `assets/matvare-expressen.svg` | Supplied by the user | 166 × 40. Red bag mark `#FF4F46` plus a **white** wordmark, so it must sit on a dark panel. Used in the print masthead and the app title bar. |
| Icons | Lucide 0.462.0 via CDN, rendered as a CSS mask over `currentColor` | A flagged substitution — the repo ships no icon set. Replace with whatever the Rust UI toolkit provides; only the icon wrapper changes. |
| `data/PSR-BREAD-2026-03-04.xlsx` | The real sample export | 352 rows, 148 orders, 16 routes, 35 products |
| `data/orders.json` | Parsed form of the above | Every figure in the designs is computed from this — useful for asserting your own parser against it |

No hand-drawn illustration, no photography, no gradients. The printed page has
no imagery beyond the logo.

## Files in this bundle

```
design_handoff_breadify/
├── README.md                              this document
├── reference/
│   ├── printed-pick-list.html             the A4 sheet — 7 pages, 4 routes
│   └── breadify-app-window.html           the four-step wizard
├── source/
│   ├── Pick List Print Dense.dc.html      editable original of the sheet
│   ├── Breadify App.dc.html               editable original of the app
│   └── Pick List Print (roomy).dc.html    earlier, roomier sheet — 27 sheets
├── assets/
│   └── matvare-expressen.svg
└── data/
    ├── PSR-BREAD-2026-03-04.xlsx
    └── orders.json
```

The `reference/` files are self-contained — open them in any browser. Print
`printed-pick-list.html` to A4 to see the real thing; each `.page` is exactly
one sheet.

The seven demo pages were chosen to cover every hard case in the file: **route
5** (34 lines, 13 stops, the most lines — two pages), **route 11** (14 orders,
including 8 at one building under three spellings of its address), **route 14**
(the nine Customer 012 crates at one address), and **route 8** (a small
route with a no-substitutes stop). If your implementation renders those four
routes correctly, it renders the file.
