import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path


SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))

import ci_helper_parallel_runner  # noqa: E402


class CiHelperParallelRunnerTests(unittest.TestCase):
    def test_unit_discover_modules_returns_sorted_matches(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "test_b.py").write_text("print('b')\n", encoding="utf-8")
            (root / "test_a.py").write_text("print('a')\n", encoding="utf-8")
            (root / "notes.txt").write_text("ignore\n", encoding="utf-8")

            modules = ci_helper_parallel_runner.discover_modules(root, "test_*.py")
            self.assertEqual([path.name for path in modules], ["test_a.py", "test_b.py"])

    def test_functional_cli_runs_discovered_modules_in_parallel(self):
        script_path = SCRIPT_DIR / "ci_helper_parallel_runner.py"
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "test_alpha.py").write_text(
                textwrap.dedent(
                    """
                    import unittest

                    class Alpha(unittest.TestCase):
                        def test_alpha(self):
                            self.assertTrue(True)

                    if __name__ == "__main__":
                        unittest.main()
                    """
                ),
                encoding="utf-8",
            )
            (root / "test_beta.py").write_text(
                textwrap.dedent(
                    """
                    import unittest

                    class Beta(unittest.TestCase):
                        def test_beta(self):
                            self.assertEqual(2 + 2, 4)

                    if __name__ == "__main__":
                        unittest.main()
                    """
                ),
                encoding="utf-8",
            )

            completed = subprocess.run(
                [
                    sys.executable,
                    str(script_path),
                    "--workers",
                    "2",
                    "--start-dir",
                    str(root),
                    "--pattern",
                    "test_*.py",
                    "--quiet",
                ],
                text=True,
                capture_output=True,
                check=False,
            )
            self.assertEqual(completed.returncode, 0, msg=completed.stderr)

    def test_regression_cli_rejects_zero_workers(self):
        script_path = SCRIPT_DIR / "ci_helper_parallel_runner.py"
        completed = subprocess.run(
            [
                sys.executable,
                str(script_path),
                "--workers",
                "0",
            ],
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(completed.returncode, 2)
        self.assertIn("--workers must be greater than zero", completed.stderr)

    def test_regression_cli_fails_when_module_fails(self):
        script_path = SCRIPT_DIR / "ci_helper_parallel_runner.py"
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "test_failure.py").write_text(
                textwrap.dedent(
                    """
                    import unittest

                    class Failure(unittest.TestCase):
                        def test_fail(self):
                            self.fail("intentional")

                    if __name__ == "__main__":
                        unittest.main()
                    """
                ),
                encoding="utf-8",
            )
            completed = subprocess.run(
                [
                    sys.executable,
                    str(script_path),
                    "--workers",
                    "1",
                    "--start-dir",
                    str(root),
                    "--pattern",
                    "test_*.py",
                    "--quiet",
                ],
                text=True,
                capture_output=True,
                check=False,
            )
            self.assertEqual(completed.returncode, 1)
            self.assertIn("--- failure:", completed.stderr)


if __name__ == "__main__":
    unittest.main()
