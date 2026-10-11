"""Write `test/sql/vectors.test` from `test/vectors.json`, the cases every ng-postcode
implementation must pass. The JSON is copied from `spec/vectors.json` in
https://github.com/Adeniyikayodee/ng-postcode; copy it again when that file changes.

    python3 scripts/vectors_test.py            # write the test file
    python3 scripts/vectors_test.py --check    # fail if it is out of date

SQL has no error values, so a rejected input is NULL. Each case is one query that is true
when the extension agrees with the spec.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TARGET = ROOT / "test" / "sql" / "vectors.test"


def text(value: str | None) -> str:
    return "NULL" if value is None else "'" + value.replace("'", "''") + "'"


def same(expression: str, expected: str | None) -> str:
    return f"({expression}) IS NOT DISTINCT FROM {text(expected)}"


def checks() -> list[str]:
    spec = json.loads((ROOT / "test" / "vectors.json").read_text(encoding="utf-8"))
    found: list[str] = []
    for case in spec["parse"]["valid"]:
        parsed = f"ng_postcode_parse({text(case['input'])})"
        found += [
            same(parsed, case["compact"]),
            same(f"ng_postcode_format({parsed})", case["canonical"]),
        ]
    for case in spec["parse"]["invalid"]:
        found.append(same(f"ng_postcode_parse({text(case['input'])})", None))
    for case in spec["prefix"]:
        cut = f"ng_postcode_truncate({text(case['input'])}, {text(case['through'])})"
        found.append(same(f"ng_postcode_format({cut})", case["prefix"]))
    for case in spec["parse_prefix"]["valid"]:
        parsed = f"ng_postcode_prefix_parse({text(case['input'])})"
        found += [
            same(parsed, case["compact"]),
            same(f"ng_postcode_format({parsed})", case["canonical"]),
        ]
    for case in spec["parse_prefix"]["invalid"]:
        found.append(same(f"ng_postcode_prefix_parse({text(case['input'])})", None))
    for case in spec["prefix_contains"]:
        inside = "true" if case["contains"] else "false"
        call = f"ng_postcode_within({text(case['code'])}, {text(case['prefix'])})"
        found.append(f"{call} IS NOT DISTINCT FROM {inside}")
    for case in spec["prefix_parent"]:
        parent = f"ng_postcode_format(ng_postcode_parent({text(case['prefix'])}))"
        found.append(same(parent, case["parent"]))
    return found


def build() -> str:
    head = (
        "# name: test/sql/vectors.test\n"
        "# description: the shared ng-postcode cases; written by scripts/vectors_test.py\n"
        "# group: [ng_postcode]\n\nrequire ng_postcode\n"
    )
    return head + "".join(f"\nquery I\nSELECT {check};\n----\ntrue\n" for check in checks())


def main() -> None:
    if "--check" not in sys.argv:
        TARGET.write_text(build(), encoding="utf-8")
    elif TARGET.read_text(encoding="utf-8") != build():
        sys.exit("test/sql/vectors.test is out of date: run python3 scripts/vectors_test.py")


if __name__ == "__main__":
    main()
