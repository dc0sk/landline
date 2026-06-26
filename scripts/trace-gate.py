#!/usr/bin/env python3
"""landline traceability gate.

Enforces rules R3 and R4 from docs/README.md section 4:

  R3  Every Must/Should FR/NFR is covered by >= 1 TC. (hard fail)
      Could/Won't gaps are reported informationally only.
  R4  Every TC names >= 1 known requirement ID, and every requirement ID
      a TC references must be declared in the SRS. (hard fail)

Parses:
  docs/requirements/system-requirements.md  -> declared {req_id: priority}
  docs/test/test-strategy.md                -> TC -> [req_ids], traced set

Stdlib only. Exit 0 when clean, exit 1 (violations to stderr) otherwise.
"""

import re
import sys
from pathlib import Path

# REPO_ROOT = parent of scripts/ (this file lives in scripts/).
REPO_ROOT = Path(__file__).resolve().parent.parent
SRS_PATH = REPO_ROOT / "docs" / "requirements" / "system-requirements.md"
TS_PATH = REPO_ROOT / "docs" / "test" / "test-strategy.md"

REQ_FULL_RE = re.compile(r"^(?:FR|NFR)-[A-Z]+-\d+$")
REQ_SCAN_RE = re.compile(r"(?:FR|NFR)-[A-Z]+-\d+")
TC_FULL_RE = re.compile(r"^TC-[A-Z]+-\d+$")
BACKTICK_RE = re.compile(r"`([^`]+)`")

MS_PRIOS = {"M", "S"}
INFO_PRIOS = {"C", "W"}


def first_backtick_token(line):
    """Return the first backtick-quoted token on the line, or None."""
    m = BACKTICK_RE.search(line)
    return m.group(1).strip() if m else None


def parse_requirements(path):
    """Return {req_id: priority} declared in the SRS tables."""
    declared = {}
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.lstrip()
        if not line.startswith("| `"):
            continue
        token = first_backtick_token(line)
        if not token or not REQ_FULL_RE.match(token):
            continue
        # Priority is the 3rd table cell: split on '|' -> ['', ' `ID` ', ' stmt ', ' M ', ...]
        cells = line.split("|")
        prio = cells[3].strip() if len(cells) > 3 else ""
        declared[token] = prio
    return declared


def parse_tests(path):
    """Return (tc_to_reqs, traced, dangling_tests).

    tc_to_reqs:      {tc_id: [req_ids]}
    traced:          set of all referenced req_ids
    dangling_tests:  list of tc_ids whose 2nd cell named zero requirements
    """
    tc_to_reqs = {}
    traced = set()
    dangling_tests = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.lstrip()
        if not line.startswith("| `TC-"):
            continue
        token = first_backtick_token(line)
        if not token or not TC_FULL_RE.match(token):
            continue
        cells = line.split("|")
        # cells: ['', ' `TC-...` ', ' <requirements cell> ', ...]
        req_cell = cells[2] if len(cells) > 2 else ""
        reqs = REQ_SCAN_RE.findall(req_cell)
        tc_to_reqs[token] = reqs
        if reqs:
            traced.update(reqs)
        else:
            dangling_tests.append(token)
    return tc_to_reqs, traced, dangling_tests


def main():
    errors = []

    if not SRS_PATH.exists():
        print(f"ERROR: missing {SRS_PATH}", file=sys.stderr)
        return 1
    if not TS_PATH.exists():
        print(f"ERROR: missing {TS_PATH}", file=sys.stderr)
        return 1

    declared = parse_requirements(SRS_PATH)
    tc_to_reqs, traced, dangling_tests = parse_tests(TS_PATH)

    declared_ids = set(declared)
    ms_ids = {rid for rid, prio in declared.items() if prio in MS_PRIOS}

    # R4: traced req ids that are not declared anywhere.
    dangling_traces = sorted(traced - declared_ids)
    # R4: tests that reference no requirement at all.
    untraced_tests = sorted(dangling_tests)
    # R3: M/S requirements with no covering TC.
    uncovered_ms = sorted(ms_ids - traced)
    # Informational: Could/Won't requirements with no covering TC.
    uncovered_info = sorted(
        rid for rid, prio in declared.items()
        if prio in INFO_PRIOS and rid not in traced
    )

    # ---- Report ----
    print("landline traceability gate")
    print("-" * 60)
    print(f"Declared requirements : {len(declared_ids)}")
    print(f"  of which M/S        : {len(ms_ids)}")
    print(f"Test cases parsed     : {len(tc_to_reqs)}")
    print(f"Traced requirements   : {len(traced & declared_ids)}")
    print()

    if uncovered_info:
        print("Informational - Could/Won't requirements without a test "
              "(not enforced):")
        for rid in uncovered_info:
            print(f"  - {rid} ({declared[rid]})")
        print()

    if dangling_traces:
        errors.append("R4 dangling trace(s) - TC references unknown "
                      "requirement id(s):")
        for rid in dangling_traces:
            owners = sorted(tc for tc, reqs in tc_to_reqs.items() if rid in reqs)
            errors.append(f"    {rid}  (referenced by {', '.join(owners)})")

    if untraced_tests:
        errors.append("R4 untraced test(s) - TC names zero requirement ids:")
        for tc in untraced_tests:
            errors.append(f"    {tc}")

    if uncovered_ms:
        errors.append("R3 uncovered M/S requirement(s) - no TC verifies:")
        for rid in uncovered_ms:
            errors.append(f"    {rid} ({declared[rid]})")

    if errors:
        print("VIOLATIONS:", file=sys.stderr)
        for line in errors:
            print(line, file=sys.stderr)
        print(file=sys.stderr)
        print("FAIL: traceability gate found violations.", file=sys.stderr)
        return 1

    print("OK: R3 (M/S coverage) and R4 (no dangling traces) satisfied.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
