import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKFLOW_PATH = REPO_ROOT / ".github" / "workflows" / "ci.yml"


class CiAllowPragmaGuardContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = WORKFLOW_PATH.read_text(encoding="utf-8")

    def test_integration_guard_step_exists(self):
        self.assertIn("name: Verify allow-pragma-free policy", self.workflow)
        self.assertIn(
            "if: steps.rust_scope.outputs.rust_changed == 'true'",
            self.workflow,
        )

    def test_regression_guard_uses_rg_on_crates_for_allow_pragmas(self):
        self.assertIn("rg -n --glob '!**/target/**' '^\\s*#\\[\\s*allow\\(' crates", self.workflow)
        self.assertIn(
            "::error::#[allow(...)] pragmas are prohibited by repository policy",
            self.workflow,
        )


if __name__ == "__main__":
    unittest.main()
