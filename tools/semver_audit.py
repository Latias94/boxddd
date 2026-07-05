#!/usr/bin/env python3
"""Audit expected 0.2 semver breaks with cargo-semver-checks."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass
from typing import Iterable


FAILURE_RE = re.compile(r"^--- failure (?P<lint>[^:]+):", re.MULTILINE)


@dataclass(frozen=True)
class ExpectedFailure:
    lint: str
    needle: str
    reason: str


EXPECTED_0_2_BREAKS: dict[str, tuple[ExpectedFailure, ...]] = {
    "boxddd": (
        ExpectedFailure(
            lint="enum_struct_variant_field_added",
            needle="field handle of variant DebugDrawCommand::Shape",
            reason="debug draw shape commands now carry an owned asset handle",
        ),
        ExpectedFailure(
            lint="enum_struct_variant_field_missing",
            needle="field shape of variant DebugDrawCommand::Shape",
            reason="debug draw shape geometry moved into persistent assets",
        ),
    ),
    "bevy_boxddd": (
        ExpectedFailure(
            lint="enum_marked_non_exhaustive",
            needle="enum HullDescriptor",
            reason="hull authoring remains open for future Box3D shape families",
        ),
    ),
    "boxddd-sys": (),
}


def failure_blocks(output: str) -> list[tuple[str, str]]:
    matches = list(FAILURE_RE.finditer(output))
    blocks: list[tuple[str, str]] = []
    for index, match in enumerate(matches):
        end = matches[index + 1].start() if index + 1 < len(matches) else len(output)
        blocks.append((match.group("lint"), output[match.start() : end]))
    return blocks


def run_semver(crate: str, baseline_rev: str, release_type: str) -> subprocess.CompletedProcess[str]:
    command = [
        "cargo",
        "semver-checks",
        "check-release",
        "-p",
        crate,
        "--baseline-rev",
        baseline_rev,
        "--release-type",
        release_type,
    ]
    return subprocess.run(
        command,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )


def audit_crate(crate: str, baseline_rev: str, release_type: str) -> int:
    expected = EXPECTED_0_2_BREAKS[crate]
    completed = run_semver(crate, baseline_rev, release_type)
    output = completed.stdout
    blocks = failure_blocks(output)

    if completed.returncode != 0 and not blocks:
        print(f"error: cargo-semver-checks failed for {crate} without semver failure blocks", file=sys.stderr)
        print(output, file=sys.stderr)
        return 1

    unknown: list[str] = []
    matched: set[ExpectedFailure] = set()
    for lint, block in blocks:
        block_matches = [
            item for item in expected if item.lint == lint and item.needle in block
        ]
        if block_matches:
            matched.update(block_matches)
        else:
            first_line = next(
                (line.strip() for line in block.splitlines() if line.strip().startswith(("field ", "enum "))),
                block.splitlines()[0].strip(),
            )
            unknown.append(f"{lint}: {first_line}")

    missing = [item for item in expected if item not in matched]
    if unknown or missing:
        for item in unknown:
            print(f"error: unexpected semver break in {crate}: {item}", file=sys.stderr)
        for item in missing:
            print(
                f"error: expected semver break not observed in {crate}: {item.lint} / {item.needle}",
                file=sys.stderr,
            )
        return 1

    if expected and completed.returncode == 0:
        print(f"error: {crate} semver audit expected known breaks but cargo-semver-checks passed", file=sys.stderr)
        return 1
    if not expected and completed.returncode != 0:
        print(f"error: {crate} should have no known semver breaks", file=sys.stderr)
        print(output, file=sys.stderr)
        return 1

    if expected:
        print(f"{crate}: known 0.2 semver breaks matched:")
        for item in expected:
            print(f"  - {item.lint}: {item.needle} ({item.reason})")
    else:
        print(f"{crate}: no semver breaks")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--baseline-rev",
        default="v0.1.0",
        help="Baseline git ref passed to cargo-semver-checks.",
    )
    parser.add_argument(
        "--release-type",
        default="minor",
        choices=["patch", "minor", "major"],
        help="Forced release type for the audit. Use minor for 0.x break discovery.",
    )
    parser.add_argument(
        "--crate",
        action="append",
        choices=sorted(EXPECTED_0_2_BREAKS),
        help="Crate to audit. Repeat to audit multiple crates; defaults to all workspace crates.",
    )
    return parser


def main(argv: Iterable[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    crates = args.crate or sorted(EXPECTED_0_2_BREAKS)
    failed = False
    for crate in crates:
        failed |= audit_crate(crate, args.baseline_rev, args.release_type) != 0
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
