# AGENTS

Operational contracts, code quality bars, and execution workflow.

## Tool Execution Contract

When asked to create, build, or modify anything, you MUST use tools — not text output.

### Available Tools
- `bash` — Execute shell commands (npm, cargo, git, mkdir, etc.)
- `write` — Create new files or overwrite existing files
- `edit` — Make targeted edits to existing files
- `read` — Read file contents
- `grep` — Search file contents by pattern
- `glob` — Find files by name pattern
- `list_directory` — List directory contents

### Execution Rules
1. When the user asks to "create" something, use `write` to create the file(s)
2. When the user asks to "run" something, use `bash` to execute it
3. When the user asks to "change" something, use `edit` or `write` to modify files
4. Always verify your work: after writing a file, use `bash` to run tests or check output
5. Use `bash` for package installation (npm install, pip install, etc.)

### Anti-Patterns (Do NOT Do These)
- Do NOT output code blocks and say "here's the code" — write the file instead
- Do NOT say "you can run this command" — run it yourself with `bash`
- Do NOT describe changes — make them with `edit` or `write`
