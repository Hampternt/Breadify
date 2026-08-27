# Pack 4 — Window shell: Open and Check

**Status:** all 8 items done, reviewed, pack gate green. Merged.
**Container:** Breadify v1 (7 packs).
**Branch:** `pack-4-window-shell`.

## Goal

Stand the app window up and make its first two steps work on real files.

**Observable when done:** run `breadify`, drop today's export on the window,
and the Check step shows what was read and every finding computed from *that*
file — not the four the design hardcodes.

## Items

| # | Item | Done when |
| --- | --- | --- |
| 1 | The window opens | `breadify` with no arguments opens a 1280 × 864 window with the design's dark palette and the embedded typefaces, and the terminal subcommands still work |
| 2 | Design tokens | The palette, spacing, radii and type scale of the handoff's screen section exist as named values, not literals scattered through the widgets |
| 3 | Title bar and step rail | The 40 px title bar carries the wordmark and the loaded filename; four clickable step tabs show number, label and note, with the active one marked |
| 4 | Action bar | Back and a contextual hint on the left, the step's primary action on the right, both disabled where the step says so |
| 5 | Step 1 — Open | A drop zone that accepts a dropped file, a `Choose file` dialog, and a recent rail listing what has been opened before |
| 6 | Loading off the paint thread | Reading a 352-row export never blocks a frame; the window shows it is working and the result arrives without a freeze |
| 7 | Step 2 — Check | Five stat cards and one card per finding, all computed from the opened file, with the severity policy: structural errors block, observations warn |
| 8 | A screenshot flag | `breadify --screenshot <file.ppm>` renders a frame and writes it, so a window can be looked at without a person at the keyboard |

## Ledger

- [x] 1 — The window opens · 1280 × 864, dark, embedded faces
- [x] 2 — Design tokens · `app::theme`, no literals in widgets
- [x] 3 — Title bar and step rail · filename, four clickable tabs with live notes
- [x] 4 — Action bar · Back, contextual hint, primary action
- [x] 5 — Step 1 — Open · drop, dialog, recent rail
- [x] 6 — Loading off the paint thread · read on a thread, collected per frame
- [x] 7 — Step 2 — Check · five stat cards, findings from the file itself
- [x] 8 — A screenshot flag · `--screenshot <file.ppm>`

**Deviations:**

- The window keeps its native decorations for now. The design draws its own
  40 px bar with minimise/maximise/close and a 10 px radius, which means a
  borderless window; on Wayland that also means hand-rolled resize hit regions,
  and it is not worth blocking the first two steps on. The bar itself is built
  and carries the wordmark, the filename and a close button.
- The step-2 findings are computed from the opened file rather than being the
  four the handoff lists verbatim. Two of the handoff's four are inaccurate
  about the sample anyway (route 11 prints as eight stops, not five). The
  informational pair it shows — the unsequenced rows and the unlabelled column
  — are now produced by `validate` as notes, so they are true of whatever file
  is open.
- Icons are drawn rather than set: none of the three embedded faces carries a
  document glyph, and the design's Lucide set is a flagged substitution.
**Code review:** one pass at high effort. Two findings, both fixed:

1. The Print tab's note paginated the whole day on every frame — 148 blocks
   laid out sixty times a second — and `Face::parsed()` re-parsed a font file
   on every measurement underneath it. The sheet count is now worked out once
   when the file is read, and the eight faces are parsed once per process. A
   full day's render went from seconds to **0.02 s**.
2. `--screenshot` had no timeout: if the capture never arrived it spun at full
   frame rate forever. It gives up after 240 frames now.

**Pack gate:** `./scripts/verify.sh` — fmt, clippy `-D warnings`, build, 85
tests across 13 files, all green. Both steps were opened and looked at:
`--screenshot` of the empty Open step and of Check with the sample loaded.
