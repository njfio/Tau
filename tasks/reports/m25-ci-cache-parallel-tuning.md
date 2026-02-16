# M25 CI Cache + Parallel Tuning

Generated: `2026-02-16T16:00:00Z`
Repository: `njfio/Tau`
Source mode: `fixture`

## Summary

| Status | Serial median ms | Parallel median ms | Improvement ms | Improvement % |
|---|---:|---:|---:|---:|
| improved | 8000 | 5100 | 2900 | 36.25 |

## Timing Samples

- command: `python3 -m unittest discover -s .github/scripts -p "test_*.py"`
- parallel command: `python3 .github/scripts/ci_helper_parallel_runner.py --workers 4 --start-dir .github/scripts --pattern "test_*.py"`
- workers: 4
- serial samples ms: [8200, 8000, 7800]
- parallel samples ms: [5200, 5000, 5100]
