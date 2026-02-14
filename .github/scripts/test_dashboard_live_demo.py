import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPTS_DIR = REPO_ROOT / "scripts" / "demo"


def write_mock_dashboard_binary(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        """#!/usr/bin/env python3
import json
import os
import sys
from pathlib import Path

args = sys.argv[1:]

if "--dashboard-contract-runner" in args:
    state_dir = Path(args[args.index("--dashboard-state-dir") + 1])
    state_dir.mkdir(parents=True, exist_ok=True)
    (state_dir / "state.json").write_text(
        json.dumps({"schema_version": 1, "widget_views": [{"widget_id": "health-summary"}]}),
        encoding="utf-8",
    )
    (state_dir / "runtime-events.jsonl").write_text(
        '{"reason_codes":["healthy_cycle"],"health_reason":"ok"}\\n',
        encoding="utf-8",
    )
    channel = state_dir / "channel-store" / "channels" / "dashboard" / "operator_ops-release-2"
    channel.mkdir(parents=True, exist_ok=True)
    control_action = None if os.environ.get("TAU_MOCK_NO_CONTROL_ACTION") == "1" else "resume"
    payload = {
        "timestamp_unix_ms": 1771042694452,
        "direction": "system",
        "event_key": "control:control-resume-streams",
        "source": "tau-dashboard-runner",
        "payload": {
            "case_id": "control-resume-streams",
            "control_action": control_action,
            "mode": "control",
            "outcome": "success",
            "upserted_widgets": 1,
        },
    }
    (channel / "log.jsonl").write_text(json.dumps(payload) + "\\n", encoding="utf-8")
    print("dashboard-runner-ok")
    raise SystemExit(0)

if "--transport-health-inspect" in args:
    print(json.dumps({"health_state": "healthy", "queue_depth": 0}))
    raise SystemExit(0)

if "--dashboard-status-inspect" in args:
    print(json.dumps({"rollout_gate": "pass", "health_state": "healthy"}))
    raise SystemExit(0)

if "--channel-store-inspect" in args:
    print(json.dumps({"status": "ok", "channel": args[args.index("--channel-store-inspect") + 1]}))
    raise SystemExit(0)

print("mock-ok " + " ".join(args))
""",
        encoding="utf-8",
    )
    path.chmod(0o755)


def write_mock_dashboard_harness(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        """#!/usr/bin/env python3
import json
import sys
from pathlib import Path

args = sys.argv[1:]
summary_out = Path(args[args.index("--summary-json-out") + 1])
summary_out.parent.mkdir(parents=True, exist_ok=True)
summary_out.write_text(
    json.dumps(
        {
            "schema_version": 1,
            "discovered_cases": 4,
            "success_cases": 4,
            "health_state": "healthy",
            "reason_codes": ["healthy_cycle"],
            "artifact_records": 6,
            "timeline": [
                {"case_id": "dashboard-load-webchat", "status": "success"},
                {"case_id": "dashboard-live-updates-snapshot", "status": "success"},
                {"case_id": "dashboard-control-refresh-status", "status": "success"},
                {"case_id": "dashboard-control-clear-output", "status": "success"},
            ],
        }
    ),
    encoding="utf-8",
)
print("mock-dashboard-harness-ok")
""",
        encoding="utf-8",
    )
    path.chmod(0o755)


def prepare_fixture_tree(repo_root: Path) -> None:
    fixture = (
        repo_root
        / "crates"
        / "tau-coding-agent"
        / "testdata"
        / "dashboard-contract"
        / "snapshot-layout.json"
    )
    fixture.parent.mkdir(parents=True, exist_ok=True)
    fixture.write_text('{"schema_version":1,"cases":[]}', encoding="utf-8")


def run_dashboard_live_script(
    repo_root: Path,
    binary_path: Path,
    harness_path: Path,
    env_overrides: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    env = dict(os.environ)
    if env_overrides:
        env.update(env_overrides)
    return subprocess.run(
        [
            str(SCRIPTS_DIR / "dashboard-live.sh"),
            "--skip-build",
            "--repo-root",
            str(repo_root),
            "--binary",
            str(binary_path),
            "--harness-bin",
            str(harness_path),
            "--timeout-seconds",
            "30",
        ],
        text=True,
        capture_output=True,
        env=env,
        check=False,
    )


class DashboardLiveDemoTests(unittest.TestCase):
    def test_unit_dashboard_live_rejects_unknown_argument(self) -> None:
        completed = subprocess.run(
            [str(SCRIPTS_DIR / "dashboard-live.sh"), "--definitely-unknown"],
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(completed.returncode, 2)
        self.assertIn("unknown argument: --definitely-unknown", completed.stderr)

    def test_functional_dashboard_live_runs_with_mock_runtime_and_harness(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            prepare_fixture_tree(root)
            binary_path = root / "bin" / "tau-coding-agent"
            harness_path = root / "bin" / "browser_automation_live_harness"
            write_mock_dashboard_binary(binary_path)
            write_mock_dashboard_harness(harness_path)

            completed = run_dashboard_live_script(root, binary_path, harness_path)
            self.assertEqual(completed.returncode, 0, msg=completed.stderr)
            self.assertIn("[demo:dashboard-live] summary: total=", completed.stdout)
            self.assertIn("failed=0", completed.stdout)

            summary_path = root / ".tau" / "demo-dashboard-live" / "dashboard-live-summary.json"
            report_path = root / ".tau" / "demo-dashboard-live" / "dashboard-live-report.json"
            self.assertTrue(summary_path.exists())
            self.assertTrue(report_path.exists())

    def test_integration_dashboard_live_report_contains_audit_and_webchat_checks(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            prepare_fixture_tree(root)
            binary_path = root / "bin" / "tau-coding-agent"
            harness_path = root / "bin" / "browser_automation_live_harness"
            write_mock_dashboard_binary(binary_path)
            write_mock_dashboard_harness(harness_path)

            completed = run_dashboard_live_script(root, binary_path, harness_path)
            self.assertEqual(completed.returncode, 0, msg=completed.stderr)

            report_path = root / ".tau" / "demo-dashboard-live" / "dashboard-live-report.json"
            payload = json.loads(report_path.read_text(encoding="utf-8"))
            self.assertEqual(payload["discovered_cases"], 4)
            self.assertEqual(payload["success_cases"], 4)
            self.assertEqual(payload["action_audit_event_count"], 1)

            webchat_check_path = Path(payload["webchat_fallback_check_path"])
            self.assertTrue(webchat_check_path.exists())
            webchat_check = json.loads(webchat_check_path.read_text(encoding="utf-8"))
            self.assertEqual(webchat_check["status"], "pass")

    def test_regression_dashboard_live_fails_closed_when_control_audit_is_missing(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            prepare_fixture_tree(root)
            binary_path = root / "bin" / "tau-coding-agent"
            harness_path = root / "bin" / "browser_automation_live_harness"
            write_mock_dashboard_binary(binary_path)
            write_mock_dashboard_harness(harness_path)

            completed = run_dashboard_live_script(
                root,
                binary_path,
                harness_path,
                env_overrides={"TAU_MOCK_NO_CONTROL_ACTION": "1"},
            )
            self.assertNotEqual(completed.returncode, 0)
            combined = completed.stdout + "\n" + completed.stderr
            self.assertIn("dashboard control action audit log entries were not captured", combined)


if __name__ == "__main__":
    unittest.main()
