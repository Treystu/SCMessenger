#!/usr/bin/env python3
"""Derive the kernel's V051 car queue from the document that owns the car list.

`#579`: `scripts/orchestrate_strict.py` defaulted `--queue` to
`scm_v1_farm_queue.jsonl`, the older farm-wave queue, which holds none of the
V051 cars, and `orchestration/manifest.yaml` named no queue at all. Section 6 of
`HANDOFF/freebuff/queue/V051_UNIFIED_MERGE_TRAIN_2026-10-14.md` records that gap
and says the document stays the execution authority until a jsonl exists.

This script is the missing half. It reads the three car sections of that document
(carry-over cars, residual cars, final cars) and writes
`orchestration/v051_cars.jsonl` in the schema `read_next_task` already consumes:
`id`, `wave`, `tier`, `status`, `depends`, `blocks`, `review`, `loc_est`,
`description`, plus `source` for provenance.

The document stays authoritative: run this again after any edit to its car
tables. `--check` regenerates in memory and exits 1 if the file on disk differs,
which is the CI-safe form (it never writes).

Two deliberate choices, so a reader does not have to infer them:

- Status is mapped from the document's own words. A row whose status or
  disposition begins with DONE is `done`; a row that is operator-gated, or whose
  disposition is `operator`, is `blocked` (the kernel must not dispatch an
  operator settings action); `superseded-close` is `retired`; everything else is
  `open`, including UNVERIFIED and DECIDED rows, because both still need work.
- Dependencies are only the ones the document states in words, quoted below in
  `STATED_DEPENDS`. Nothing is inferred from section order.

`MT-08..MT-12` is one row in the document and five cars in the queue, because
`#580` tracks their status one by one.
"""

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "HANDOFF" / "freebuff" / "queue" / "V051_UNIFIED_MERGE_TRAIN_2026-10-14.md"
QUEUE = ROOT / "orchestration" / "v051_cars.jsonl"

CAR_ID = re.compile(r"^(MT-\d{2}(?:\.\.MT-\d{2})?|R-\d{2}[ab]?|F-\d+|P0-8 BK-01)$")
SPAN = re.compile(r"^MT-(\d{2})\.\.MT-(\d{2})$")
FINAL_BULLET = re.compile(r"^- \*\*(F-\d+)\s+([^*]*):\*\*\s*(.*)$")

# Quoted from the document, not inferred:
#   R-00a  "Must land first: every car runs CI."
#   R-00b  "P0, land before #521" and R-15 is the #521 vehicle (D5, section 1).
#   F-3    "only after every car above is landed or explicitly closed" and
#          "F-1/F-2 are merged"; F-2 follows F-1's version bump.
STATED_DEPENDS = {
    "R-15": ["R-00a", "R-00b"],
    "F-2": ["F-1"],
    "F-3": ["F-1", "F-2"],
}


def map_status(text):
    lowered = text.strip().lower()
    if lowered.startswith("done"):
        return "done"
    if lowered in ("operator", "operator-gated") or "operator-gated" in lowered:
        return "blocked"
    if lowered.startswith("superseded-close"):
        return "retired"
    return "open"


def review_of(gate_text):
    upper = gate_text.upper()
    return "r8" if ("R8" in upper or "RULE-8" in upper) else "none"


def expand(car_id):
    span = SPAN.match(car_id)
    if not span:
        return [car_id]
    first, last = int(span.group(1)), int(span.group(2))
    return [f"MT-{number:02d}" for number in range(first, last + 1)]


def parse_cars(text):
    """Every car row or bullet, in document order, with the text it carries."""
    cars = []
    section = None
    for line in text.splitlines():
        heading = re.match(r"^##\s+(\d+)\.", line)
        if heading:
            section = int(heading.group(1))
            continue
        if section == 5:
            bullet = FINAL_BULLET.match(line)
            if bullet:
                cars.append({
                    "id": bullet.group(1),
                    "description": f"{bullet.group(2).strip()}: {bullet.group(3).strip()}",
                    "gate": "",
                    "status": "open",
                })
            continue
        if section not in (3, 4) or not line.startswith("|"):
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if not cells or not CAR_ID.match(cells[0]):
            continue
        if section == 3 and len(cells) >= 5:
            car, scope, prs, gate, status = cells[0], cells[1], cells[2], cells[3], cells[4]
            description = f"{scope} | PRs: {prs} | recorded: {status}"
        elif section == 4 and len(cells) >= 5:
            car, pr, title, gate, status = cells[0], cells[1], cells[2], cells[3], cells[4]
            description = f"{title} | PR: {pr} | disposition: {status}"
        else:
            continue
        for one in expand(car):
            cars.append({
                "id": one,
                "description": description,
                "gate": gate,
                "status": map_status(status),
            })
    return cars


def build(text):
    cars = parse_cars(text)
    if not cars:
        raise SystemExit("[ERROR] no cars parsed from the merge-train document")
    entries = []
    for car in cars:
        entries.append({
            "id": car["id"],
            "wave": "V051",
            "tier": "CODER",
            "status": car["status"],
            "depends": STATED_DEPENDS.get(car["id"], []),
            "blocks": [],
            "review": review_of(car["gate"]),
            "loc_est": 0,
            "description": car["description"],
            "source": f"{SOURCE.relative_to(ROOT).as_posix()}#car-{car['id']}",
        })
    for entry in entries:
        for dependency in entry["depends"]:
            for other in entries:
                if other["id"] == dependency:
                    other["blocks"].append(entry["id"])
    unknown = {
        dependency
        for entry in entries
        for dependency in entry["depends"]
        if dependency not in {other["id"] for other in entries}
    }
    if unknown:
        raise SystemExit(f"[ERROR] depends on cars that do not exist: {sorted(unknown)}")
    return entries


def render(entries):
    return "".join(json.dumps(entry, sort_keys=True) + "\n" for entry in entries)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true",
                       help="exit 1 if the queue on disk differs from the document")
    args = parser.parse_args(argv)

    try:
        text = SOURCE.read_text(encoding="utf-8")
    except OSError as exc:
        print(f"[ERROR] cannot read car source {SOURCE}: {exc}", file=sys.stderr)
        return 2

    entries = build(text)
    rendered = render(entries)
    counts = {}
    for entry in entries:
        counts[entry["status"]] = counts.get(entry["status"], 0) + 1
    summary = ", ".join(f"{status}={count}" for status, count in sorted(counts.items()))

    if args.check:
        try:
            current = QUEUE.read_text(encoding="utf-8")
        except OSError as exc:
            print(f"[FAIL] queue {QUEUE} cannot be read: {exc}")
            return 1
        if current == rendered:
            print(f"[OK] {QUEUE.relative_to(ROOT)} matches the document: {len(entries)} cars ({summary})")
            return 0
        on_disk = {json.loads(line)["id"] for line in current.splitlines() if line.strip()}
        generated = {entry["id"] for entry in entries}
        print("[FAIL] queue on disk does not match the merge-train document")
        print(f"  missing from disk: {sorted(generated - on_disk)}")
        print(f"  present only on disk: {sorted(on_disk - generated)}")
        print("  run: python3 scripts/v051_queue_sync.py")
        return 1

    QUEUE.parent.mkdir(parents=True, exist_ok=True)
    QUEUE.write_text(rendered, encoding="utf-8")
    print(f"[DONE] wrote {QUEUE.relative_to(ROOT)}: {len(entries)} cars ({summary})")
    for entry in entries:
        deps = ",".join(entry["depends"]) or "-"
        print(f"  {entry['id']} [{entry['status']}] review={entry['review']} depends={deps}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
