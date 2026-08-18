#!/usr/bin/env python3
"""Validate two probe reports and render a fixed-format Markdown comparison."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any


MAX_REPORT_BYTES = 64 * 1024
MAX_METRIC_BYTES = 1_000_000_000
COMMIT_RE = re.compile(r"^[0-9a-f]{40}$")

# key, label, higher-is-better
METRICS = (
    ("ui_static", "UI reservation", False),
    ("bss", "`.bss`", False),
    ("static_data_bss", "Static `.data` + `.bss`", False),
    ("dram_reserved_before_stack", "DRAM reserved before stack", False),
    ("iram_used", "IRAM used", False),
    ("stack_available", "Available stack region", True),
    ("flash_payload", "Flash payload", False),
)


def load_json(path: Path) -> dict[str, Any]:
    if path.stat().st_size > MAX_REPORT_BYTES:
        raise ValueError(f"report is unexpectedly large: {path}")
    value = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise ValueError(f"report must contain an object: {path}")
    return value


def validate(report: dict[str, Any]) -> dict[str, int] | None:
    if report.get("available") is False:
        return None
    if report.get("schema") != 1:
        raise ValueError("unsupported report schema")
    if report.get("probe") != "power-monitor":
        raise ValueError("unexpected probe name")
    if report.get("target") != "riscv32imc-unknown-none-elf":
        raise ValueError("unexpected target")
    if report.get("profile") != "release":
        raise ValueError("unexpected profile")
    if not isinstance(report.get("commit"), str) or not COMMIT_RE.fullmatch(report["commit"]):
        raise ValueError("invalid commit SHA")

    values = report.get("metrics")
    if not isinstance(values, dict):
        raise ValueError("metrics must contain an object")

    validated: dict[str, int] = {}
    for key, _, _ in METRICS:
        value = values.get(key)
        if isinstance(value, bool) or not isinstance(value, int):
            raise ValueError(f"metric {key!r} must be an integer")
        if not 0 <= value <= MAX_METRIC_BYTES:
            raise ValueError(f"metric {key!r} is outside the accepted range")
        validated[key] = value
    return validated


def bytes_label(value: int) -> str:
    return f"{value:,} B"


def delta_label(base: int, head: int, higher_is_better: bool) -> str:
    delta = head - base
    if delta == 0:
        return "—"
    improved = delta > 0 if higher_is_better else delta < 0
    marker = "🟢" if improved else "🔴"
    return f"{delta:+,} B {marker}"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base", type=Path, required=True)
    parser.add_argument("--head", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    base_report = load_json(args.base)
    head_report = load_json(args.head)
    base = validate(base_report)
    head = validate(head_report)
    if head is None:
        raise ValueError("head measurement cannot be unavailable")

    lines = [
        "<!-- guillotine-memory-probe -->",
        "## ESP32-C3 memory probe",
        "",
    ]

    if base is None:
        lines.extend(
            [
                "No base measurement was available; this run establishes the initial result.",
                "",
                "| Metric | Current |",
                "|---|---:|",
            ]
        )
        for key, label, _ in METRICS:
            lines.append(f"| {label} | {bytes_label(head[key])} |")
    else:
        lines.extend(
            [
                "| Metric | Base | PR | Delta |",
                "|---|---:|---:|---:|",
            ]
        )
        for key, label, higher_is_better in METRICS:
            lines.append(
                f"| {label} | {bytes_label(base[key])} | {bytes_label(head[key])} | "
                f"{delta_label(base[key], head[key], higher_is_better)} |"
            )

    lines.extend(
        [
            "",
            "The UI reservation is the named `POWER_MONITOR_UI` static at full configured "
            "capacity. This report is informational and does not enforce a budget.",
        ]
    )
    args.output.write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    main()
