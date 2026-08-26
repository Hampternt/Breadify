#!/usr/bin/env python3
"""Dump the structure of a PSR-BREAD order export.

Python 3 stdlib only -- reads the OOXML out of the .xlsx zip directly, so no
openpyxl/pandas needed. Prints what docs/excel-format.md documents: the header
check, per-column profiles, the functional dependencies between columns, the
route/stop table and the product-major block order. Run it against a new export
before trusting that doc.

    python3 tools/inspect_xlsx.py "PSR-BREAD-2026-03-04-to-2026-03-04 (1).xlsx"
"""

import collections
import re
import sys
import xml.etree.ElementTree as ET
import zipfile

NS = "{http://schemas.openxmlformats.org/spreadsheetml/2006/main}"

HEADERS = [
    "Order ID", "Quantity", "Product ID", "Product Name", "Supplier SKU",
    "Position", "Supplier", "Customer", "Department", "Delivery street",
    "Comment", "Route nickname", "Route ordering", "Accept alternatives",
]
# Column letter -> short name used below.
FIELD = {
    "A": "order", "B": "qty", "C": "pid", "D": "pname", "E": "sku",
    "F": "pos", "G": "sup", "H": "cust", "I": "dept", "J": "addr",
    "K": "note", "L": "route", "M": "ord", "N": "alt", "O": "unlabelled",
}


def load(path):
    """-> (headers dict by column letter, list of rows, list of cell types)."""
    with zipfile.ZipFile(path) as z:
        names = z.namelist()
        shared = []
        if "xl/sharedStrings.xml" in names:
            root = ET.fromstring(z.read("xl/sharedStrings.xml"))
            shared = [
                "".join(t.text or "" for t in si.iter(NS + "t"))
                for si in root.findall(NS + "si")
            ]
        book = ET.fromstring(z.read("xl/workbook.xml"))
        sheets = [s.get("name") for s in book.iter(NS + "sheet")]
        sheet = ET.fromstring(z.read("xl/worksheets/sheet1.xml"))

    dim = sheet.find(NS + "dimension")
    print(f"sheets      : {sheets}")
    print(f"dimension   : {dim.get('ref') if dim is not None else '(none)'}")
    print(f"sharedStrings: {len(shared)}")

    rows = []
    for row in sheet.find(NS + "sheetData"):
        cells = {}
        for c in row.findall(NS + "c"):
            col = re.match(r"[A-Z]+", c.get("r")).group(0)
            v = c.find(NS + "v")
            t = c.get("t")
            if t == "s" and v is not None:
                val = shared[int(v.text)]
            else:
                val = v.text if v is not None else ""
            cells[col] = (val, t or "n")
        rows.append((int(row.get("r")), cells))

    header = {col: val for col, (val, _) in rows[0][1].items()}
    data = []
    for rnum, cells in rows[1:]:
        rec = {name: "" for name in FIELD.values()}
        rec["_row"] = rnum
        rec["_present"] = "".join(sorted(cells, key=lambda c: (len(c), c)))
        rec["_types"] = {FIELD[c]: t for c, (_, t) in cells.items() if c in FIELD}
        for col, (val, _) in cells.items():
            if col in FIELD:
                rec[FIELD[col]] = val
        data.append(rec)
    return header, data


def check_headers(header):
    print("\n== headers ==")
    got = [header.get(chr(ord("A") + i), "") for i in range(len(HEADERS))]
    for i, (want, have) in enumerate(zip(HEADERS, got)):
        flag = "ok " if want == have else "!! "
        print(f"  {flag}{chr(ord('A') + i)}: {have!r}" + ("" if want == have else f"  (expected {want!r})"))
    extra = sorted(c for c in header if c > chr(ord("A") + len(HEADERS) - 1))
    print(f"  headers beyond N: {extra or 'none (column O is unlabelled, as expected)'}")


def profile(data):
    print("\n== column profiles ==")
    for col in sorted(FIELD, key=lambda c: (len(c), c)):
        name = FIELD[col]
        vals = [r[name] for r in data]
        nonempty = [v for v in vals if v != ""]
        types = collections.Counter(
            r["_types"].get(name, "MISSING") for r in data
        )
        uniq = collections.Counter(nonempty)
        print(f"  {col} {name:<11} types={dict(types)} empty={len(vals) - len(nonempty)} distinct={len(uniq)}")
        if 0 < len(uniq) <= 20:
            print(f"      values: {uniq.most_common()}")

    print("\n== cell-presence patterns (empty cells are absent, not blank) ==")
    for pat, n in collections.Counter(r["_present"] for r in data).most_common():
        print(f"  {pat} -> {n}")


def deps(data):
    print("\n== functional dependencies ==")
    pairs = [
        ("pid", "pname"), ("pname", "pid"), ("pid", "sku"), ("sku", "pid"),
        ("pid", "sup"), ("sup", "pos"),
        ("order", "cust"), ("order", "dept"), ("order", "addr"),
        ("order", "route"), ("order", "ord"), ("order", "alt"),
        ("order", "note"), ("order", "sup"),
        ("cust", "route"), ("cust", "addr"), ("cust", "alt"),
        ("addr", "route"),
    ]
    for a, b in pairs:
        seen = collections.defaultdict(set)
        for r in data:
            seen[r[a]].add(r[b])
        bad = {k: v for k, v in seen.items() if len(v) > 1}
        note = "1:1" if not bad else f"{len(bad)} violations e.g. {list(bad.items())[:2]}"
        print(f"  {a} -> {b}: {note}")

    dup = {k: v for k, v in collections.Counter(
        (r["order"], r["pid"]) for r in data).items() if v > 1}
    print(f"  (order, product) duplicates: {len(dup)} {list(dup)[:3]}")
    lines = collections.Counter(collections.Counter(r["order"] for r in data).values())
    print(f"  lines-per-order histogram: {sorted(lines.items())}")


def route_key(nickname):
    """Natural sort key: numeric prefix first, then the rest as text."""
    m = re.match(r"\s*(\d+)\s*(.*)", nickname)
    if m:
        return (0, int(m.group(1)), m.group(2))
    m = re.match(r"([^\d]*)(\d*)", nickname)
    return (1, m.group(1), int(m.group(2) or 0))


def routes(data):
    print("\n== routes and stops ==")
    by = collections.defaultdict(list)
    for r in data:
        by[r["route"]].append(r)
    for rt in sorted(by, key=route_key):
        rs = by[rt]
        stops = collections.defaultdict(set)
        for r in rs:
            stops[int(r["ord"] or 0)].add(r["addr"])
        line = ", ".join(
            f"{o}{'*' + str(len(a)) if len(a) > 1 else ''}"
            for o, a in sorted(stops.items())
        )
        print(f"  route {rt!r:<8} rows={len(rs):<3} customers={len({r['cust'] for r in rs}):<3} orderings: {line}")
    print("  (* = several addresses share that ordering: one stop, several delivery points)")

    zero = [r for r in data if r["ord"] == "0"]
    print(f"\n  ordering==0 (unsequenced): {len(zero)} rows, "
          f"{len({r['cust'] for r in zero})} customers, "
          f"{len({r['route'] for r in zero})} routes")


def blocks(data):
    print("\n== incoming row order (product-major: must be re-sorted) ==")
    seq = []
    for r in data:
        key = (r["pos"], r["pid"], r["sku"], r["pname"])
        if not seq or seq[-1][0] != key:
            seq.append([key, 0])
        seq[-1][1] += 1
    for (pos, pid, sku, pname), n in seq:
        print(f"  {pos:<17} pid={pid:<5} sku={sku:<11} n={n:<3} {pname}")
    print(f"  {len(seq)} blocks over {len({r['pid'] for r in data})} distinct products")


def main():
    if len(sys.argv) != 2:
        print(__doc__)
        return 1
    path = sys.argv[1]
    header, data = load(path)
    print(f"data rows   : {len(data)}")
    check_headers(header)
    profile(data)
    deps(data)
    routes(data)
    blocks(data)
    return 0


if __name__ == "__main__":
    sys.exit(main())
