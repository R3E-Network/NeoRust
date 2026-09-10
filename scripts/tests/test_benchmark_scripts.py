from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPTS_DIR = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS_DIR))

import check_bench_regression  # noqa: E402
import render_monthly_report  # noqa: E402
from criterion_results import format_duration, load_results, nearest_rank  # noqa: E402


def write_result(
	criterion_dir: Path,
	name: str,
	baseline: str,
	mean_ns: float,
	per_iteration_ns: list[float],
) -> None:
	result_dir = criterion_dir / name / baseline
	result_dir.mkdir(parents=True)
	(result_dir / "estimates.json").write_text(
		json.dumps({"mean": {"point_estimate": mean_ns}}), encoding="utf-8"
	)
	(result_dir / "sample.json").write_text(
		json.dumps(
			{
				"iters": [index + 1 for index in range(len(per_iteration_ns))],
				"times": [
					value * (index + 1) for index, value in enumerate(per_iteration_ns)
				],
			}
		),
		encoding="utf-8",
	)


class CriterionResultsTests(unittest.TestCase):
	def test_nearest_rank_and_duration_formatting(self) -> None:
		self.assertEqual(nearest_rank([40.0, 10.0, 30.0, 20.0], 0.50), 20.0)
		self.assertEqual(nearest_rank([40.0, 10.0, 30.0, 20.0], 0.99), 40.0)
		self.assertEqual(format_duration(1_250), "1.25 us")

	def test_load_results_uses_per_iteration_samples(self) -> None:
		with tempfile.TemporaryDirectory() as temporary_dir:
			criterion_dir = Path(temporary_dir)
			write_result(criterion_dir, "crypto/sign", "current", 25.0, [10.0, 20.0, 30.0])
			result = load_results(criterion_dir, "current")["crypto/sign"]
			self.assertEqual(result.mean_ns, 25.0)
			self.assertEqual(result.p50_ns, 20.0)
			self.assertEqual(result.p95_ns, 30.0)


class RegressionGateTests(unittest.TestCase):
	def test_regression_fails_and_writes_summary(self) -> None:
		with tempfile.TemporaryDirectory() as temporary_dir:
			root = Path(temporary_dir)
			criterion_dir = root / "criterion"
			write_result(criterion_dir, "crypto/sign", "base", 100.0, [100.0])
			write_result(criterion_dir, "crypto/sign", "current", 106.0, [106.0])
			summary = root / "summary.md"

			exit_code = check_bench_regression.main(
				[
					"--baseline",
					"base",
					"--current",
					"current",
					"--threshold",
					"5",
					"--criterion-dir",
					str(criterion_dir),
					"--markdown",
					str(summary),
				]
			)

			self.assertEqual(exit_code, 1)
			self.assertIn("+6.00%", summary.read_text(encoding="utf-8"))
			self.assertIn("REGRESSION", summary.read_text(encoding="utf-8"))

	def test_new_benchmark_passes_without_baseline(self) -> None:
		with tempfile.TemporaryDirectory() as temporary_dir:
			criterion_dir = Path(temporary_dir)
			write_result(criterion_dir, "new/benchmark", "current", 42.0, [42.0])
			exit_code = check_bench_regression.main(
				[
					"--baseline",
					"base",
					"--current",
					"current",
					"--threshold",
					"5",
					"--criterion-dir",
					str(criterion_dir),
				]
			)
			self.assertEqual(exit_code, 0)


class MonthlyReportTests(unittest.TestCase):
	def test_report_replaces_template_fields(self) -> None:
		with tempfile.TemporaryDirectory() as temporary_dir:
			root = Path(temporary_dir)
			criterion_dir = root / "criterion"
			write_result(criterion_dir, "wallet/encrypt", "monthly-2026-09", 2_000.0, [1_900.0])
			template = root / "template.md"
			template.write_text(
				"# {{MONTH}}\n{{GENERATED_UTC}}\n{{BENCH_COUNT}}\n{{RESULTS_TABLE}}\n",
				encoding="utf-8",
			)
			output = root / "report.md"
			exit_code = render_monthly_report.main(
				[
					"--baseline",
					"monthly-2026-09",
					"--criterion-dir",
					str(criterion_dir),
					"--template",
					str(template),
					"--month",
					"2026-09",
					"--output",
					str(output),
				]
			)

			self.assertEqual(exit_code, 0)
			report = output.read_text(encoding="utf-8")
			self.assertIn("# 2026-09", report)
			self.assertIn("wallet/encrypt", report)
			self.assertNotIn("{{", report)


if __name__ == "__main__":
	unittest.main()
