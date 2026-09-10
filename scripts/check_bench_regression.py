#!/usr/bin/env python3
"""Compare two Criterion baselines and fail on mean-time regressions."""

from __future__ import annotations

import argparse
import math
import sys
from pathlib import Path

from criterion_results import CriterionDataError, format_duration, load_results, markdown_cell


def _threshold(value: str) -> float:
	try:
		threshold = float(value)
	except ValueError as error:
		raise argparse.ArgumentTypeError("threshold must be numeric") from error
	if not math.isfinite(threshold) or threshold < 0:
		raise argparse.ArgumentTypeError("threshold must be finite and non-negative")
	return threshold


def _parser() -> argparse.ArgumentParser:
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--baseline", required=True, help="name of the baseline result directory")
	parser.add_argument("--current", required=True, help="name of the current result directory")
	parser.add_argument("--threshold", required=True, type=_threshold, help="maximum slowdown percent")
	parser.add_argument("--criterion-dir", type=Path, default=Path("target/criterion"))
	parser.add_argument("--markdown", type=Path, help="append a Markdown summary to this file")
	return parser


def _render_table(
	baseline: dict,
	current: dict,
	threshold: float,
) -> tuple[str, list[str]]:
	rows = [
		"| Benchmark | Base mean | Current mean | Delta | p50 | p95 | p99 | Status |",
		"|---|---:|---:|---:|---:|---:|---:|---|",
	]
	regressions: list[str] = []
	for name in sorted(set(baseline) | set(current)):
		base_result = baseline.get(name)
		current_result = current.get(name)
		if current_result is None:
			base_mean = format_duration(base_result.mean_ns)
			current_mean = delta = p50 = p95 = p99 = "n/a"
			status = "REMOVED"
		elif base_result is None:
			base_mean = delta = "n/a"
			current_mean = format_duration(current_result.mean_ns)
			p50 = format_duration(current_result.p50_ns)
			p95 = format_duration(current_result.p95_ns)
			p99 = format_duration(current_result.p99_ns)
			status = "NEW"
		else:
			delta_percent = (current_result.mean_ns - base_result.mean_ns) / base_result.mean_ns * 100
			base_mean = format_duration(base_result.mean_ns)
			current_mean = format_duration(current_result.mean_ns)
			delta = f"{delta_percent:+.2f}%"
			p50 = format_duration(current_result.p50_ns)
			p95 = format_duration(current_result.p95_ns)
			p99 = format_duration(current_result.p99_ns)
			if delta_percent > threshold:
				status = "REGRESSION"
				regressions.append(name)
			else:
				status = "OK"

		cells = [name, base_mean, current_mean, delta, p50, p95, p99, status]
		rows.append("| " + " | ".join(markdown_cell(cell) for cell in cells) + " |")
	return "\n".join(rows), regressions


def main(argv: list[str] | None = None) -> int:
	args = _parser().parse_args(argv)
	try:
		baseline = load_results(args.criterion_dir, args.baseline)
		current = load_results(args.criterion_dir, args.current)
	except (CriterionDataError, OSError, ValueError) as error:
		print(f"error: unable to read Criterion results: {error}", file=sys.stderr)
		return 2

	if not current:
		print(f"error: no Criterion results found for current baseline {args.current!r}", file=sys.stderr)
		return 2

	table, regressions = _render_table(baseline, current, args.threshold)
	summary = (
		f"## Benchmark regression gate\n\n"
		f"Maximum allowed mean slowdown: {args.threshold:g}%\n\n"
		f"{table}\n"
	)
	print(summary)
	if args.markdown:
		try:
			with args.markdown.open("a", encoding="utf-8") as markdown_file:
				markdown_file.write("\n" + summary)
		except OSError as error:
			print(f"error: unable to write Markdown summary: {error}", file=sys.stderr)
			return 2

	if regressions:
		print(
			f"error: {len(regressions)} benchmark(s) exceeded the {args.threshold:g}% threshold",
			file=sys.stderr,
		)
		return 1
	return 0


if __name__ == "__main__":
	raise SystemExit(main())
