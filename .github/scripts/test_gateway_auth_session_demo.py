import subprocess
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
DEMO_SCRIPT = REPO_ROOT / "scripts" / "demo" / "gateway-auth-session.sh"
RUNBOOK_DOC = REPO_ROOT / "docs" / "guides" / "gateway-auth-session-smoke.md"
QUICKSTART_DOC = REPO_ROOT / "docs" / "guides" / "quickstart.md"


class GatewayAuthSessionDemoTests(unittest.TestCase):
    def test_unit_gateway_auth_session_script_has_expected_step_labels(self):
        contents = DEMO_SCRIPT.read_text(encoding="utf-8")
        self.assertIn("wait-for-gateway-startup", contents)
        self.assertIn("gateway-auth-session-issue-valid-password", contents)
        self.assertIn("gateway-status-authorized-with-issued-token", contents)
        self.assertIn("gateway-auth-session-invalid-password-fails-closed", contents)
        self.assertIn("gateway-status-expired-token-fails-closed", contents)
        self.assertIn("/gateway/auth/session", contents)
        self.assertIn("/gateway/status", contents)

    def test_functional_gateway_auth_session_help_prints_usage(self):
        completed = subprocess.run(
            [str(DEMO_SCRIPT), "--help"],
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(completed.returncode, 0)
        self.assertIn("Usage:", completed.stdout)
        self.assertIn("gateway-auth-session", completed.stdout)

    def test_integration_gateway_auth_session_runbook_and_quickstart_reference_command(self):
        runbook = RUNBOOK_DOC.read_text(encoding="utf-8")
        quickstart = QUICKSTART_DOC.read_text(encoding="utf-8")
        self.assertIn("./scripts/demo/gateway-auth-session.sh", runbook)
        self.assertIn("./scripts/demo/gateway-auth-session.sh", quickstart)

    def test_regression_gateway_auth_session_skip_build_requires_binary(self):
        completed = subprocess.run(
            [
                str(DEMO_SCRIPT),
                "--skip-build",
                "--repo-root",
                str(REPO_ROOT),
                "--binary",
                "/tmp/tau-missing-binary-for-gateway-auth-session",
            ],
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertNotEqual(completed.returncode, 0)
        self.assertIn("missing tau-coding-agent binary", completed.stderr)


if __name__ == "__main__":
    unittest.main()
