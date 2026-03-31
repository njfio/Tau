# Tasks: Issue #3654 - Define the governed Tau Ralph supervisor loop across gateway, session, memory, and learning

- [ ] T1 Specify: publish the mission-supervisor loop contract, verifier
      contract, and state ownership boundaries across mission/session/memory.
- [ ] T2 Plan: break the architecture into implementation slices covering
      supervisor state, outer-loop execution, verifier adapters, memory/learning
      writeback, and operator surfaces.
- [ ] T3 Align: map existing Tau subsystems (`tau-session`, `tau-memory`,
      cortex, `tau-orchestrator`, gateway/TUI) into the loop and identify
      compatibility/migration boundaries.
