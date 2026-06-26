# Cursor adapter

Config: `~/.cursor/hooks.json` (or project `.cursor/hooks.json`)

| Event | Light | Notes |
|-------|-------|-------|
| `beforeSubmitPrompt` | yellow | User sent a prompt |
| `afterAgentThought` | yellow | Thinking block completed |
| `preToolUse` (Write\|Edit) | red | File tool about to run |
| `afterFileEdit` | red | File was edited |
| `postToolUse` (Write\|Edit\|Shell) | yellow | Back to thinking |
| `beforeShellExecution` | green (blinking) | Terminal command waiting for Run/Skip |
| `afterShellExecution` | yellow | Shell command finished |
| `beforeMCPExecution` | green (blinking) | MCP tool waiting for approval |
| `afterMCPExecution` | yellow | MCP tool finished |
| `stop` | green (blinking in Agent) / green (idle in Ask) | Turn finished — Agent waits for your reply; Ask goes idle |
| `sessionEnd` | green | Session closed |

Docs: https://cursor.com/docs/hooks
