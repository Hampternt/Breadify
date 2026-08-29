#!/usr/bin/env python3
"""Replace the people in an export with placeholders, in place.

The two sample exports in this repo are real days: real customers, real
delivery addresses, and free text written by real people — one of the comments
carried someone's work email address. The repo is public, and the tests need a
file of exactly this shape, so the answer is neither "delete it" nor "ship it":
it is to keep every structural property the tests rest on and replace only the
columns that describe people.

Rewritten (columns of `docs/excel-format.md` §2):

    H  Customer          -> Customer 001, Customer 002, ...
    I  Department        -> Department 01, ...
    J  Delivery street   -> Street 01, ...
    K  Comment           -> Note 1, ...

Left exactly as they were: order id, quantity, product id, product name,
supplier SKU, position, supplier, route nickname, route ordering, accept
alternatives, and the unlabelled region column. So row counts, route
structure, the product catalogue, the department-presence pattern and the
sequence numbers all survive, and every mapping between them holds — one
customer still has one address, one address still sits on one route.

How, and why this way: every string in those four columns is a *shared
string*, and none of those entries is used by any other column (asserted
below). So the only file that changes is `xl/sharedStrings.xml`, and only the
entries in question — the worksheet, its cell types, the styles and the
workbook are copied through byte for byte. Rebuilding the workbook instead
would risk turning a real Excel boolean into the string "TRUE", which is the
one thing the loader cannot read.

A maintainer's tool, stdlib only, like `inspect_xlsx.py` beside it. It parses
with `xml.etree`, which is not hardened against hostile XML — point it at this
repo's own exports, not at a file someone sent you.

Usage:

    python3 tools/anonymise_xlsx.py FILE.xlsx [FILE.xlsx ...]

Writes each file in place after checking its own work.
"""

import re
import shutil
import sys
import zipfile
from collections import defaultdict
from pathlib import Path
from xml.etree import ElementTree as ET

NS = "{http://schemas.openxmlformats.org/spreadsheetml/2006/main}"
SHARED_STRINGS = "xl/sharedStrings.xml"
SHEET = "xl/worksheets/sheet1.xml"

# The columns that describe a person or a place, and what each becomes. Order
# matters only when one string somehow lands in two of them; the first wins.
REPLACED = [
    ("H", "Customer {n:03d}"),
    ("I", "Department {n:02d}"),
    ("J", "Street {n:02d}"),
    ("K", "Note {n}"),
]


def column_of(reference: str) -> str:
    """`H14` -> `H`."""
    return re.match(r"[A-Z]+", reference).group()


def shared_string_use(sheet_xml: bytes) -> dict[int, set[str]]:
    """Which columns use each shared string, ignoring the header row."""
    sheet = ET.fromstring(sheet_xml)
    used: dict[int, set[str]] = defaultdict(set)

    for row in sheet.find(NS + "sheetData"):
        if row.get("r") == "1":
            continue  # the header row names the columns; it is not data
        for cell in row.findall(NS + "c"):
            if cell.get("t") != "s":
                continue
            value = cell.find(NS + "v")
            if value is not None:
                used[int(value.text)].add(column_of(cell.get("r")))

    return used


class Placeholders:
    """One placeholder per real string, shared across every file in a run.

    A customer who appears on both the bread day and the freezer day has to
    become the *same* `Customer 042` in both, or the mapping cannot be applied
    to the docs and tests that name them. Numbering therefore runs across the
    whole run rather than per file, and is stable because the files are taken
    in the order given and each file's strings in index order.
    """

    def __init__(self) -> None:
        self.counters = {column: 0 for column, _ in REPLACED}
        self.by_text: dict[tuple[str, str], str] = {}

    def name(self, column: str, text: str) -> str:
        key = (column, text)
        if key not in self.by_text:
            self.counters[column] += 1
            template = dict(REPLACED)[column]
            self.by_text[key] = template.format(n=self.counters[column])
        return self.by_text[key]


def plan(
    used: dict[int, set[str]], strings: list[str], placeholders: Placeholders
) -> dict[int, str]:
    """Which shared-string index in this file becomes which placeholder."""
    replacements: dict[int, str] = {}

    for index in sorted(used):
        columns = used[index]
        for column, _ in REPLACED:
            if column not in columns:
                continue
            replacements[index] = placeholders.name(column, strings[index])
            break

    return replacements


def check_separable(used: dict[int, set[str]], replacements: dict[int, str]) -> None:
    """No string being replaced may also be used by a column we keep.

    If one ever were, rewriting it here would silently change a product name
    or a route nickname somewhere else in the sheet.
    """
    replaced_columns = {column for column, _ in REPLACED}
    for index in replacements:
        stray = used[index] - replaced_columns
        if stray:
            raise SystemExit(
                f"shared string {index} is also used by column(s) "
                f"{', '.join(sorted(stray))} — refusing to rewrite it"
            )


def rewrite_shared_strings(xml: bytes, replacements: dict[int, str]) -> bytes:
    """Replace the text of the named entries, leaving the rest of the file be.

    An `<si>` may hold several `<t>` runs. The first takes the placeholder and
    the rest are emptied, so the entry reads as one string afterwards.
    """
    ET.register_namespace("", NS[1:-1])
    table = ET.fromstring(xml)

    for index, entry in enumerate(table.findall(NS + "si")):
        if index not in replacements:
            continue
        runs = entry.iter(NS + "t")
        for position, run in enumerate(runs):
            run.text = replacements[index] if position == 0 else ""
            run.set("{http://www.w3.org/XML/1998/namespace}space", "preserve")

    return ET.tostring(table, encoding="UTF-8", xml_declaration=True)


def read_strings(xml: bytes) -> list[str]:
    """The shared-string table, flattened to one string per entry."""
    table = ET.fromstring(xml)
    return [
        "".join(run.text or "" for run in entry.iter(NS + "t"))
        for entry in table.findall(NS + "si")
    ]


def anonymise(path: Path, placeholders: Placeholders) -> None:
    source = zipfile.ZipFile(path)
    used = shared_string_use(source.read(SHEET))
    strings = read_strings(source.read(SHARED_STRINGS))
    replacements = plan(used, strings, placeholders)
    check_separable(used, replacements)

    scratch = path.with_suffix(path.suffix + ".writing")
    with zipfile.ZipFile(scratch, "w", zipfile.ZIP_DEFLATED) as out:
        for item in source.infolist():
            data = source.read(item.filename)
            if item.filename == SHARED_STRINGS:
                data = rewrite_shared_strings(data, replacements)
            out.writestr(item, data)
    source.close()

    verify(scratch, len(replacements))
    shutil.move(scratch, path)
    print(f"{path.name}: {len(replacements)} strings replaced")
    return {strings[index]: name for index, name in replacements.items()}


def verify(path: Path, expected: int) -> None:
    """The rewritten file still reads, and holds none of the old text."""
    check = zipfile.ZipFile(path)
    table = ET.fromstring(check.read(SHARED_STRINGS))
    strings = [
        "".join(run.text or "" for run in entry.iter(NS + "t"))
        for entry in table.findall(NS + "si")
    ]

    placeholders = sum(
        1
        for text in strings
        if re.fullmatch(r"(Customer|Department|Street|Note) ?\d+", text)
    )
    if placeholders != expected:
        raise SystemExit(f"{path.name}: expected {expected} placeholders, found {placeholders}")

    for text in strings:
        if "@" in text:
            raise SystemExit(f"{path.name}: an email address survived: {text!r}")


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(__doc__)
        return 1
    placeholders = Placeholders()
    mapping: dict[str, str] = {}
    for name in argv[1:]:
        mapping.update(anonymise(Path(name), placeholders))

    # The same names appear in the docs, the tests and the design handoff.
    # Longest first, so a name that contains another is replaced whole.
    trail = Path("anonymise-mapping.tsv")
    with trail.open("w", encoding="utf-8") as out:
        for real in sorted(mapping, key=len, reverse=True):
            out.write(f"{real}\t{mapping[real]}\n")
    print(f"{trail}: {len(mapping)} names — apply, then delete this file")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
