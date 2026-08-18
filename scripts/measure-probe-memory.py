#!/usr/bin/env python3
"""Extract stable memory metrics from the ESP layout probe ELF."""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path


PROBE = "power-monitor"
TARGET = "riscv32imc-unknown-none-elf"
UI_SYMBOL = "POWER_MONITOR_UI"


def command(*args: str) -> str:
    return subprocess.run(args, check=True, text=True, stdout=subprocess.PIPE).stdout


def read_sections(llvm_size: str, elf: Path) -> dict[str, int]:
    sections: dict[str, int] = {}
    for line in command(llvm_size, "-A", str(elf)).splitlines():
        fields = line.split()
        if len(fields) >= 3 and fields[0].startswith("."):
            sections[fields[0]] = int(fields[1], 10)
    return sections


def read_symbol_size(llvm_nm: str, elf: Path, symbol: str) -> int:
    for line in command(llvm_nm, "--print-size", str(elf)).splitlines():
        fields = line.split()
        if len(fields) >= 4 and fields[-1] == symbol:
            return int(fields[1], 16)
    raise RuntimeError(f"ELF does not contain required symbol {symbol!r}")


def section_sum(sections: dict[str, int], *names: str) -> int:
    return sum(sections.get(name, 0) for name in names)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--elf", type=Path, required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--rustc", required=True)
    parser.add_argument("--llvm-size", required=True)
    parser.add_argument("--llvm-nm", required=True)
    parser.add_argument("--json", type=Path, required=True)
    parser.add_argument("--symbols", type=Path, required=True)
    args = parser.parse_args()

    if not args.elf.is_file():
        raise RuntimeError(f"ELF not found: {args.elf}")

    sections = read_sections(args.llvm_size, args.elf)
    ui_static = read_symbol_size(args.llvm_nm, args.elf, UI_SYMBOL)

    static_data_bss = section_sum(
        sections, ".data", ".data.wifi", ".bss", ".bss.wifi", ".noinit"
    )
    metrics = {
        "ui_static": ui_static,
        "bss": section_sum(sections, ".bss", ".bss.wifi"),
        "static_data_bss": static_data_bss,
        "dram_reserved_before_stack": static_data_bss
        + section_sum(sections, ".rwdata_dummy"),
        "iram_used": section_sum(sections, ".trap", ".rwtext", ".rwtext.wifi"),
        "stack_available": section_sum(sections, ".stack"),
        "flash_payload": section_sum(
            sections, ".flash.appdesc", ".rodata", ".rodata.wifi", ".text"
        ),
    }

    report = {
        "schema": 1,
        "available": True,
        "probe": PROBE,
        "target": TARGET,
        "profile": "release",
        "commit": args.commit,
        "rustc": args.rustc,
        "metrics": metrics,
        "sections": dict(sorted(sections.items())),
    }

    args.json.parent.mkdir(parents=True, exist_ok=True)
    args.json.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")

    symbols = command(
        args.llvm_nm,
        "--demangle",
        "--print-size",
        "--size-sort",
        str(args.elf),
    )
    args.symbols.write_text(symbols)


if __name__ == "__main__":
    main()
