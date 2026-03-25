---
name: example-unified-skill
description: Template skill demonstrating the unified format with tools, commands, and hooks.
runtime: process
entrypoint: ./handler.sh
permissions:
  - run-commands
  - read-files
tools:
  - name: example.greet
    description: Greet a user by name and return a formatted message.
    parameters:
      type: object
      properties:
        name:
          type: string
          description: The name of the person to greet.
      required:
        - name
    handler: greet_handler
  - name: example.summarize
    description: Summarize a block of text into a shorter form.
    parameters:
      type: object
      properties:
        text:
          type: string
          description: The text to summarize.
        max_length:
          type: integer
          description: Maximum character length of the summary.
      required:
        - text
commands:
  - name: example-status
    description: Show the status of the example skill.
    template: "status --format json"
    arguments: {}
hooks:
  - run-start
  - run-end
---

# Example Unified Skill

This skill demonstrates the unified skill manifest format that replaces the
deprecated extension system (`tau-extensions`). It declares tools, commands,
and lifecycle hooks in a single YAML frontmatter block.

## Migration from Extensions

Extensions previously used `extension.json` manifests with separate `hooks`,
`tools`, `commands`, and `permissions` arrays. The unified skill format
consolidates everything into the YAML frontmatter of a Markdown skill file:

| Extension field   | Skill equivalent       |
|-------------------|------------------------|
| `runtime`         | `runtime`              |
| `entrypoint`      | `entrypoint`           |
| `hooks`           | `hooks` (same values)  |
| `permissions`     | `permissions`          |
| `tools`           | `tools` (with handler) |
| `commands`        | `commands`             |

## Tool Dispatch

When a tool call arrives for `example.greet`, the skill runtime:
1. Loads this skill from the catalog.
2. Verifies the tool name exists in the `tools` list.
3. Spawns the `entrypoint` process with the tool request as JSON on stdin.
4. Reads the JSON response from stdout.

## Hook Dispatch

On `AgentStart`, the runtime dispatches `run-start` to all skills that
declare it. On `AgentEnd`, `run-end` is dispatched. Hooks receive a JSON
payload on stdin describing the event context.

## Runtime Protocol

The entrypoint receives a JSON object on stdin:

```json
{
  "kind": "tool-call",
  "tool": {
    "name": "example.greet",
    "arguments": { "name": "Alice" }
  },
  "skill_name": "example-unified-skill"
}
```

And must respond on stdout with a JSON object:

```json
{
  "content": { "text": "Hello, Alice!" },
  "is_error": false
}
```
