# Tasks: Issue #3652 - Retry mutating gateway turns when the model promises work without using tools

- [ ] T1 Red: add gateway regressions for retry-success and retry-exhaustion on
      zero-tool mutating action turns.
- [ ] T2 Green: implement bounded corrective retries and discard failed-attempt
      assistant text from final output/streaming.
- [ ] T3 Verify: rerun the targeted gateway retry and control tests.
