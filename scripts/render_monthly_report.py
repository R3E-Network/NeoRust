#!/usr/bin/env python3
"""Render a monthly Markdown report from one Criterion baseline."""

from __future__ import annotations

import argparse
import os
import re
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path

from criterion_results import CriterionDataError, format_duration, load_results, markdown_cell


def _month(value: str) -> str:
	if not re.fullmatch(r"\d{4}-(0[1-9]|1[0-2])", value):
		raise argparse.ArgumentTypeError("month must use YYYY-MM format")
	return value


def _parser() -> argparse.ArgumentParser:
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--baseline", required=True)
	parser.add_argument("--criterion-dir", type=Path, default=Path("target/criterion"))
	parser.add_argument("--template", type=Path, required=True)
	parser.add_argument("--month", type=_month, required=True)
	parser.add_argument("--output", type=Path, required=True)
	return parser


def _results_table(results: dict) -> str:
	rows = [
		"| Benchmark | Mean | p50 | p95 | p99 |",
		"|---|---:|---:|---:|---:|",
	]
	for name, result in sorted(results.items()):
		cells = [
			name,
			format_duration(result.mean_ns),
			format_duration(result.p50_ns),
			format_duration(result.p95_ns),
			format_duration(result.p99_ns),
		]
		rows.append("| " + " | ".join(markdown_cell(cell) for cell in cells) + " |")
	return "\n".join(rows)


def _atomic_write(path: Path, content: str) -> None:
	path.parent.mkdir(parents=True, exist_ok=True)
	temporary_name: str | None = None
	try:
		with tempfile.NamedTemporaryFile(
			"w", encoding="utf-8", dir=path.parent, prefix=f".{path.name}.", delete=False
		) as temporary_file:
			temporary_file.write(content)
			temporary_name = temporary_file.name
		os.replace(temporary_name, path)
	except Exception:
		if temporary_name:
			try:
				os.unlink(temporary_name)
			except FileNotFoundError:
				pass
		raise


def main(argv: list[str] | None = None) -> int:
	args = _parser().parse_args(argv)
	try:
		results = load_results(args.criterion_dir, args.baseline)
		if not results:
			raise CriterionDataError(
				f"no Criterion results found for baseline {args.baseline!r}"
			)
		template = args.template.read_text(encoding="utf-8")
		replacements = {
			"{{MONTH}}": args.month,
			"{{GENERATED_UTC}}": datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S UTC"),
			"{{BENCH_COUNT}}": str(len(results)),
			"{{RESULTS_TABLE}}": _results_table(results),
		}
		for placeholder, value in replacements.items():
			template = template.replace(placeholder, value)
		unresolved = sorted(set(re.findall(r"{{[A-Z0-9_]+}}", template)))
		if unresolved:
			raise CriterionDataError(
				"unresolved template placeholders: " + ", ".join(unresolved)
			)
		_atomic_write(args.output, template)
	except (CriterionDataError, OSError, ValueError) as error:
		print(f"error: unable to render monthly report: {error}", file=sys.stderr)
		return 2

	print(f"Wrote {args.output} with {len(results)} benchmark result(s)")
	return 0


if __name__ == "__main__":
	raise SystemExit(main())
