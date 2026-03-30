# Tasks: Issue #3647 - Fix tau-coding-agent cli integration training store lock race in auth provider tests

- [x] T1 Red: capture the current GitHub Actions failure from PR `#3631`
      showing `tau-coding-agent --test cli_integration` failing with
      `failed to initialize live RL training store` / `database is locked`.
- [x] T2 Green: disable live RL startup in the shared CLI integration command
      helper so auth-provider subprocess tests no longer contend on the default
      SQLite training store. Covers C-01 and C-02.
- [x] T3 Verify: rerun the auth-provider CLI integration selector, rerun the
      exact `fast-validate` reproduction, then push and watch PR `#3631` for
      advancement past the live RL store lock blocker. Covers C-03.
