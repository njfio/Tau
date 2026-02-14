import unittest
from pathlib import Path


SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parents[1]

PR_TEMPLATE = REPO_ROOT / ".github" / "pull_request_template.md"
DOCS_INDEX = REPO_ROOT / "docs" / "README.md"
SIGNOFF_GUIDE = REPO_ROOT / "docs" / "guides" / "release-signoff-checklist.md"
VALIDATION_GUIDE = REPO_ROOT / "docs" / "guides" / "final-validation-package.md"
DRY_RUN_GUIDE = REPO_ROOT / "docs" / "guides" / "final-validation-package-dry-run.md"


class PrTemplateValidationPackageTests(unittest.TestCase):
    def test_unit_pr_template_contains_required_surface_and_validation_sections(self):
        content = PR_TEMPLATE.read_text(encoding="utf-8")
        self.assertIn("## Mandatory live-run evidence", content)
        self.assertIn("## Validation matrix evidence", content)
        self.assertIn("voice", content)
        self.assertIn("browser automation", content)
        self.assertIn("dashboard", content)
        self.assertIn("custom command", content)
        self.assertIn("memory", content)

    def test_functional_validation_guide_defines_required_package_contents(self):
        content = VALIDATION_GUIDE.read_text(encoding="utf-8")
        self.assertIn("## Required package contents", content)
        self.assertIn("Execution transcripts", content)
        self.assertIn("Logs/traces/screenshots/audio", content)
        self.assertIn("Artifact manifest", content)
        self.assertIn("Go/no-go summary", content)
        self.assertIn(".tau/live-run-unified/manifest.json", content)
        self.assertIn(".tau/live-run-unified/report.json", content)

    def test_integration_docs_index_and_signoff_guide_link_validation_package(self):
        docs_index = DOCS_INDEX.read_text(encoding="utf-8")
        self.assertIn("guides/final-validation-package.md", docs_index)

        signoff = SIGNOFF_GUIDE.read_text(encoding="utf-8")
        self.assertIn("final-validation-package.md", signoff)

    def test_regression_dry_run_includes_real_command_and_artifact_evidence(self):
        content = DRY_RUN_GUIDE.read_text(encoding="utf-8")
        self.assertIn("rehearsal-2026-02-14", content)
        self.assertIn(
            "./scripts/demo/live-run-unified.sh --skip-build --timeout-seconds 180 --keep-going",
            content,
        )
        self.assertIn("total=5 passed=5 failed=0", content)
        self.assertIn(".tau/live-run-unified/manifest.json", content)
        self.assertIn(".tau/live-run-unified/report.json", content)


if __name__ == "__main__":
    unittest.main()
