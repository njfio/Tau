import json
import re
import subprocess
import tempfile
import unittest
from pathlib import Path


SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parents[1]
PUBLISH_SCRIPT = REPO_ROOT / "scripts" / "dev" / "hierarchy-graph-publish.sh"
POLICY_PATH = REPO_ROOT / "tasks" / "policies" / "hierarchy-graph-publication-policy.json"
ROADMAP_SYNC_GUIDE = REPO_ROOT / "docs" / "guides" / "roadmap-status-sync.md"
DOCS_INDEX = REPO_ROOT / "docs" / "README.md"


class HierarchyGraphPublicationContractTests(unittest.TestCase):
    def test_unit_policy_and_script_contract_exists(self):
        self.assertTrue(PUBLISH_SCRIPT.is_file())
        self.assertTrue(PUBLISH_SCRIPT.stat().st_mode & 0o111)
        self.assertTrue(POLICY_PATH.is_file())

        policy = json.loads(POLICY_PATH.read_text(encoding="utf-8"))
        self.assertEqual(policy["schema_version"], 1)
        self.assertEqual(policy["policy_id"], "hierarchy-graph-publication-policy")
        self.assertEqual(policy["artifact_basename"], "issue-hierarchy-graph")
        self.assertIn("snapshot_dir_pattern", policy)
        self.assertIn("retention_days", policy)
        self.assertGreater(policy["retention_days"], 0)

    def test_functional_publish_emits_snapshot_and_discoverability_index(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            tmp = Path(temp_dir)
            graph_json = tmp / "issue-hierarchy-graph.json"
            graph_md = tmp / "issue-hierarchy-graph.md"
            history_dir = tmp / "history"

            graph_json.write_text(
                json.dumps(
                    {
                        "schema_version": 1,
                        "generated_at": "2026-02-15T12:00:00Z",
                        "root_issue_number": 1678,
                        "summary": {
                            "in_scope_nodes": 3,
                            "in_scope_edges": 2,
                            "missing_links": 0,
                            "orphan_nodes": 0,
                        },
                    }
                ),
                encoding="utf-8",
            )
            graph_md.write_text("# Issue Hierarchy Graph\n", encoding="utf-8")

            completed = subprocess.run(
                [
                    "bash",
                    str(PUBLISH_SCRIPT),
                    "--graph-json",
                    str(graph_json),
                    "--graph-md",
                    str(graph_md),
                    "--history-dir",
                    str(history_dir),
                    "--retention-days",
                    "30",
                    "--now-utc",
                    "2026-02-15T12:10:00Z",
                    "--quiet",
                ],
                text=True,
                capture_output=True,
                check=False,
            )
            self.assertEqual(completed.returncode, 0, msg=completed.stderr)

            index_path = history_dir / "index.json"
            self.assertTrue(index_path.is_file())
            index_payload = json.loads(index_path.read_text(encoding="utf-8"))
            self.assertEqual(index_payload["policy_id"], "hierarchy-graph-publication-policy")
            self.assertEqual(index_payload["retention_days"], 30)
            self.assertEqual(len(index_payload["snapshots"]), 1)

            snapshot = index_payload["snapshots"][0]
            self.assertTrue(re.match(r"^20260215T120000Z-root1678$", snapshot["snapshot_id"]))
            snapshot_dir = history_dir / snapshot["snapshot_id"]
            self.assertTrue((snapshot_dir / "issue-hierarchy-graph.json").is_file())
            self.assertTrue((snapshot_dir / "issue-hierarchy-graph.md").is_file())

    def test_regression_publish_prunes_expired_snapshots(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            tmp = Path(temp_dir)
            graph_json = tmp / "issue-hierarchy-graph.json"
            graph_md = tmp / "issue-hierarchy-graph.md"
            history_dir = tmp / "history"
            history_dir.mkdir(parents=True, exist_ok=True)

            old_snapshot_id = "20251201T000000Z-root1678"
            old_snapshot_dir = history_dir / old_snapshot_id
            old_snapshot_dir.mkdir(parents=True, exist_ok=True)
            (old_snapshot_dir / "issue-hierarchy-graph.json").write_text("{}", encoding="utf-8")
            (old_snapshot_dir / "issue-hierarchy-graph.md").write_text("# old\n", encoding="utf-8")

            index_path = history_dir / "index.json"
            index_path.write_text(
                json.dumps(
                    {
                        "schema_version": 1,
                        "policy_id": "hierarchy-graph-publication-policy",
                        "retention_days": 30,
                        "snapshots": [
                            {
                                "snapshot_id": old_snapshot_id,
                                "generated_at": "2025-12-01T00:00:00Z",
                                "root_issue_number": 1678,
                                "json_path": f"{old_snapshot_id}/issue-hierarchy-graph.json",
                                "markdown_path": f"{old_snapshot_id}/issue-hierarchy-graph.md",
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )

            graph_json.write_text(
                json.dumps(
                    {
                        "schema_version": 1,
                        "generated_at": "2026-02-15T12:00:00Z",
                        "root_issue_number": 1678,
                        "summary": {
                            "in_scope_nodes": 3,
                            "in_scope_edges": 2,
                            "missing_links": 0,
                            "orphan_nodes": 0,
                        },
                    }
                ),
                encoding="utf-8",
            )
            graph_md.write_text("# Issue Hierarchy Graph\n", encoding="utf-8")

            completed = subprocess.run(
                [
                    "bash",
                    str(PUBLISH_SCRIPT),
                    "--graph-json",
                    str(graph_json),
                    "--graph-md",
                    str(graph_md),
                    "--history-dir",
                    str(history_dir),
                    "--retention-days",
                    "30",
                    "--now-utc",
                    "2026-02-15T12:10:00Z",
                    "--quiet",
                ],
                text=True,
                capture_output=True,
                check=False,
            )
            self.assertEqual(completed.returncode, 0, msg=completed.stderr)

            updated_index = json.loads(index_path.read_text(encoding="utf-8"))
            snapshot_ids = {entry["snapshot_id"] for entry in updated_index["snapshots"]}
            self.assertNotIn(old_snapshot_id, snapshot_ids)
            self.assertFalse(old_snapshot_dir.exists())

    def test_integration_docs_reference_publication_workflow(self):
        guide_text = ROADMAP_SYNC_GUIDE.read_text(encoding="utf-8")
        docs_index_text = DOCS_INDEX.read_text(encoding="utf-8")

        self.assertIn("hierarchy-graph-publish.sh", guide_text)
        self.assertIn("hierarchy-graph-publication-policy.json", guide_text)
        self.assertIn("Hierarchy Graph Publication", docs_index_text)


if __name__ == "__main__":
    unittest.main()
