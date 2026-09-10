"""Read and format Criterion benchmark result directories."""

from __future__ import annotations

import json
import math
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable


class CriterionDataError(ValueError):
	"""Raised when Criterion output exists but is incomplete or malformed."""


@dataclass(frozen=True)
class BenchmarkResult:
	name: str
	mean_ns: float
	p50_ns: float | None
	p95_ns: float | None
	p99_ns: float | None


def _finite_number(value: Any, field: str, source: Path) -> float:
	if isinstance(value, bool) or not isinstance(value, (int, float)):
		raise CriterionDataError(f"{source}: {field} must be numeric")
	number = float(value)
	if not math.isfinite(number):
		raise CriterionDataError(f"{source}: {field} must be finite")
	return number


def nearest_rank(values: Iterable[float], quantile: float) -> float | None:
	"""Return a nearest-rank quantile for an iterable of sample values."""

	if not 0 < quantile <= 1:
		raise ValueError("quantile must be in the interval (0, 1]")
	sorted_values = sorted(values)
	if not sorted_values:
		return None
	index = max(0, math.ceil(quantile * len(sorted_values)) - 1)
	return sorted_values[index]


def _sample_percentiles(sample_path: Path) -> tuple[float | None, float | None, float | None]:
	if not sample_path.is_file():
		return None, None, None

	with sample_path.open(encoding="utf-8") as sample_file:
		sample = json.load(sample_file)
	iters = sample.get("iters")
	times = sample.get("times")
	if not isinstance(iters, list) or not isinstance(times, list) or len(iters) != len(times):
		raise CriterionDataError(f"{sample_path}: iters and times must be equal-length arrays")

	per_iteration_ns: list[float] = []
	for index, (iterations, elapsed) in enumerate(zip(iters, times)):
		iteration_count = _finite_number(iterations, f"iters[{index}]", sample_path)
		elapsed_ns = _finite_number(elapsed, f"times[{index}]", sample_path)
		if iteration_count <= 0 or elapsed_ns < 0:
			raise CriterionDataError(
				f"{sample_path}: sample iteration counts must be positive and times non-negative"
			)
		per_iteration_ns.append(elapsed_ns / iteration_count)

	return (
		nearest_rank(per_iteration_ns, 0.50),
		nearest_rank(per_iteration_ns, 0.95),
		nearest_rank(per_iteration_ns, 0.99),
	)


def load_results(criterion_dir: Path, baseline: str) -> dict[str, BenchmarkResult]:
	"""Load one named Criterion baseline, keyed by benchmark path."""

	if not baseline or "/" in baseline or "\\" in baseline:
		raise CriterionDataError("baseline must be a non-empty directory name")
	if not criterion_dir.exists():
		return {}

	results: dict[str, BenchmarkResult] = {}
	for estimates_path in criterion_dir.rglob("estimates.json"):
		if estimates_path.parent.name != baseline:
			continue
		benchmark_dir = estimates_path.parent.parent
		try:
			benchmark_name = benchmark_dir.relative_to(criterion_dir).as_posix()
		except ValueError as error:
			raise CriterionDataError(f"{estimates_path}: result is outside criterion directory") from error

		with estimates_path.open(encoding="utf-8") as estimates_file:
			estimates = json.load(estimates_file)
		try:
			mean_value = estimates["mean"]["point_estimate"]
		except (KeyError, TypeError) as error:
			raise CriterionDataError(
				f"{estimates_path}: missing mean.point_estimate"
			) from error
		mean_ns = _finite_number(mean_value, "mean.point_estimate", estimates_path)
		if mean_ns <= 0:
			raise CriterionDataError(f"{estimates_path}: mean.point_estimate must be positive")

		if benchmark_name in results:
			raise CriterionDataError(f"duplicate benchmark result: {benchmark_name}")
		p50_ns, p95_ns, p99_ns = _sample_percentiles(estimates_path.with_name("sample.json"))
		results[benchmark_name] = BenchmarkResult(
			name=benchmark_name,
			mean_ns=mean_ns,
			p50_ns=p50_ns,
			p95_ns=p95_ns,
			p99_ns=p99_ns,
		)

	return results


def format_duration(nanoseconds: float | None) -> str:
	if nanoseconds is None:
		return "n/a"
	if nanoseconds < 1_000:
		return f"{nanoseconds:.2f} ns"
	if nanoseconds < 1_000_000:
		return f"{nanoseconds / 1_000:.2f} us"
	if nanoseconds < 1_000_000_000:
		return f"{nanoseconds / 1_000_000:.2f} ms"
	return f"{nanoseconds / 1_000_000_000:.2f} s"


def markdown_cell(value: str) -> str:
	return value.replace("\\", "\\\\").replace("|", "\\|").replace("\n", " ")
