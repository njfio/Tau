import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))

import ci_quality_mode  # noqa: E402


class QualityModeTests(unittest.TestCase):
    def test_unit_resolve_quality_mode_codex_non_heavy_uses_light_lane(self):
        decision = ci_quality_mode.resolve_quality_mode(
            event_name="pull_request",
            head_ref="codex/issue-123",
            heavy_changed=False,
        )
        self.assertEqual(decision.mode, "codex-light")
        self.assertEqual(decision.reason, "codex-branch-non-heavy-pr")
        self.assertFalse(decision.heavy_changed)

    def test_functional_render_summary_includes_cost_governance_fields(self):
        decision = ci_quality_mode.QualityModeDecision(
            mode="full",
            reason="codex-branch-heavy-pr",
            heavy_changed=True,
        )
        summary = ci_quality_mode.render_summary(decision)
        self.assertIn("### CI Cost Governance", summary)
        self.assertIn("- Mode: full", summary)
        self.assertIn("- Reason: codex-branch-heavy-pr", summary)
        self.assertIn("- Heavy paths changed: true", summary)

    def test_integration_cli_writes_workflow_output_and_summary(self):
        script_path = SCRIPT_DIR / "ci_quality_mode.py"
        with tempfile.TemporaryDirectory() as temp_dir:
            output_path = Path(temp_dir) / "github_output.txt"
            summary_path = Path(temp_dir) / "summary.md"
            subprocess.run(
                [
                    sys.executable,
                    str(script_path),
                    "--event-name",
                    "pull_request",
                    "--head-ref",
                    "codex/issue-456",
                    "--heavy-changed",
                    "false",
                    "--output",
                    str(output_path),
                    "--summary",
                    str(summary_path),
                ],
                check=True,
            )
            output_raw = output_path.read_text(encoding="utf-8")
            summary_raw = summary_path.read_text(encoding="utf-8")
            self.assertIn("mode=codex-light", output_raw)
            self.assertIn("reason=codex-branch-non-heavy-pr", output_raw)
            self.assertIn("heavy_changed=false", output_raw)
            self.assertIn("Lane: light (codex smoke)", summary_raw)

    def test_regression_non_codex_or_heavy_pr_never_downgrades(self):
        non_codex = ci_quality_mode.resolve_quality_mode(
            event_name="pull_request",
            head_ref="feature/new-lane",
            heavy_changed=False,
        )
        self.assertEqual(non_codex.mode, "full")
        self.assertEqual(non_codex.reason, "pull-request-default")

        heavy_codex = ci_quality_mode.resolve_quality_mode(
            event_name="pull_request",
            head_ref="codex/issue-789",
            heavy_changed=True,
        )
        self.assertEqual(heavy_codex.mode, "full")
        self.assertEqual(heavy_codex.reason, "codex-branch-heavy-pr")


if __name__ == "__main__":
    unittest.main()
