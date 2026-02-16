import json
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKFLOW_PATH = REPO_ROOT / ".github" / "workflows" / "ci.yml"
SCRIPT_PATH = REPO_ROOT / "scripts" / "dev" / "ci-cache-parallel-tuning-report.sh"
RUNNER_PATH = REPO_ROOT / ".github" / "scripts" / "ci_helper_parallel_runner.py"
GUIDE_PATH = REPO_ROOT / "docs" / "guides" / "ci-cache-parallel-tuning.md"
REPORT_JSON_PATH = REPO_ROOT / "tasks" / "reports" / "m25-ci-cache-parallel-tuning.json"
REPORT_MD_PATH = REPO_ROOT / "tasks" / "reports" / "m25-ci-cache-parallel-tuning.md"


class CiCacheParallelContractTests(unittest.TestCase):
    def test_unit_required_paths_exist(self):
        self.assertTrue(SCRIPT_PATH.is_file(), msg=f"missing script: {SCRIPT_PATH}")
        self.assertTrue(SCRIPT_PATH.stat().st_mode & 0o111)
        self.assertTrue(RUNNER_PATH.is_file(), msg=f"missing runner: {RUNNER_PATH}")
        self.assertTrue(GUIDE_PATH.is_file(), msg=f"missing guide: {GUIDE_PATH}")
        self.assertTrue(REPORT_JSON_PATH.is_file(), msg=f"missing report json: {REPORT_JSON_PATH}")
        self.assertTrue(REPORT_MD_PATH.is_file(), msg=f"missing report md: {REPORT_MD_PATH}")

    def test_functional_ci_workflow_has_cache_shared_keys_and_parallel_helper_step(self):
        workflow = WORKFLOW_PATH.read_text(encoding="utf-8")
        self.assertIn("shared-key: ci-quality-linux-", workflow)
        self.assertIn("shared-key: ci-wasm-smoke-", workflow)
        self.assertIn("shared-key: ci-cross-platform-", workflow)
        self.assertIn("shared-key: ci-coverage-", workflow)
        self.assertIn(
            "python3 .github/scripts/ci_helper_parallel_runner.py --workers 4 --start-dir .github/scripts --pattern \"test_*.py\"",
            workflow,
        )

    def test_integration_report_shape(self):
        report = json.loads(REPORT_JSON_PATH.read_text(encoding="utf-8"))
        self.assertEqual(report["schema_version"], 1)
        self.assertIn("serial_median_ms", report)
        self.assertIn("parallel_median_ms", report)
        self.assertIn("improvement_ms", report)
        self.assertIn("status", report)
        self.assertIn("command", report)
        self.assertIn("workers", report)

    def test_regression_workflow_keeps_validate_helper_step(self):
        workflow = WORKFLOW_PATH.read_text(encoding="utf-8")
        self.assertIn("- name: Validate CI helper scripts", workflow)
        self.assertEqual(workflow.count("Validate CI helper scripts"), 1)


if __name__ == "__main__":
    unittest.main()
