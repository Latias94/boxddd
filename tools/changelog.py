#!/usr/bin/env python3
"""Changelog utilities for release automation."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import Iterable


HEADING_RE = re.compile(r"^## \[(?P<label>[^\]]+)\](?:\s+-\s+.*)?\s*$")
VERSION_RE = re.compile(r"v?\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?")


def normalize_version(version: str) -> str:
    return version.strip().removeprefix("v")


def heading_matches_version(label: str, version: str) -> bool:
    wanted = normalize_version(version)
    normalized_label = normalize_version(label)
    if normalized_label == wanted:
        return True

    # Accept legacy workspace headings such as "boxddd 0.1.0" while preferring
    # plain "[0.1.0]" release sections for new entries.
    matches = [normalize_version(match.group(0)) for match in VERSION_RE.finditer(label)]
    return wanted in matches


def find_version_section(changelog: Path, version: str) -> str:
    lines = changelog.read_text(encoding="utf-8").splitlines()

    start = None
    for index, line in enumerate(lines):
        match = HEADING_RE.match(line)
        if match and heading_matches_version(match.group("label"), version):
            start = index + 1
            break

    wanted = normalize_version(version)
    if start is None:
        raise ValueError(f"{changelog} has no section for version {wanted}")

    end = len(lines)
    for index in range(start, len(lines)):
        if lines[index].startswith("## "):
            end = index
            break

    section = "\n".join(lines[start:end]).strip()
    if not section:
        raise ValueError(f"{changelog} section {wanted} is empty")
    return section + "\n"


def is_markdown_structure(line: str) -> bool:
    stripped = line.strip()
    return (
        not stripped
        or stripped.startswith(("- ", "* ", "+ ", "> ", "|", "#", "<!--"))
        or stripped.startswith("[")
        or re.match(r"^\d+\.\s", stripped) is not None
    )


def is_plain_prose(line: str) -> bool:
    return bool(line.strip()) and not line.startswith((" ", "\t")) and not is_markdown_structure(line)


def likely_hard_wrap_violations(section: str) -> list[str]:
    violations: list[str] = []
    in_fenced_code = False
    previous_plain_prose = False

    for lineno, line in enumerate(section.splitlines(), start=1):
        stripped = line.strip()
        if stripped.startswith("```"):
            in_fenced_code = not in_fenced_code
            previous_plain_prose = False
            continue
        if in_fenced_code or not stripped:
            previous_plain_prose = False
            continue

        if is_plain_prose(line):
            if previous_plain_prose:
                violations.append(f"line {lineno}: {line}")
            previous_plain_prose = True
            continue

        if line.startswith(("  ", "\t")) and not is_markdown_structure(line):
            violations.append(f"line {lineno}: {line}")
        previous_plain_prose = False

    return violations


def write_output(text: str, output: Path | None) -> None:
    if output is None:
        print(text, end="")
    else:
        output.write_text(text, encoding="utf-8")


def command_extract(args: argparse.Namespace) -> int:
    try:
        section = find_version_section(args.changelog, args.version)
    except ValueError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1

    if args.include_heading:
        heading = f"## [{normalize_version(args.version)}]"
        section = f"{heading}\n\n{section}"
    write_output(section, args.output)
    return 0


def command_check_soft_wrap(args: argparse.Namespace) -> int:
    try:
        section = find_version_section(args.changelog, args.version)
    except ValueError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1

    violations = likely_hard_wrap_violations(section)
    if not violations:
        return 0

    print(
        f"error: {args.changelog} section {normalize_version(args.version)} appears to contain manually wrapped prose.",
        file=sys.stderr,
    )
    print("Use soft wrapping instead of hard wrapping changelog prose:", file=sys.stderr)
    for violation in violations:
        print(f"  {violation}", file=sys.stderr)
    return 1


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    def add_common(command: argparse.ArgumentParser) -> None:
        command.add_argument("--version", required=True, help="Release version, with or without a leading v")
        command.add_argument("--changelog", type=Path, default=Path("CHANGELOG.md"))

    extract = subparsers.add_parser("extract", help="Extract one release section")
    add_common(extract)
    extract.add_argument("--output", type=Path, help="Write release notes to this file")
    extract.add_argument("--include-heading", action="store_true", help="Include a release heading in the output")
    extract.set_defaults(func=command_extract)

    check = subparsers.add_parser("check-soft-wrap", help="Reject manually wrapped prose in one release section")
    add_common(check)
    check.set_defaults(func=command_check_soft_wrap)

    return parser


def main(argv: Iterable[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
