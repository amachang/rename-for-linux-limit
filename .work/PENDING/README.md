# Pended Sprints

This directory stores pended sprints via `/sprint-pend` command.

## File Naming Convention

Files are named with date prefix for easy sorting:
- `YYYYMMDD-<sprint-name>.md`

## Lifecycle

1. **Pend**: `/sprint-pend` moves `CURRENT_SPRINT.md` here with date prefix
2. **Resume**: `/sprint-resume` moves a pended sprint back to `CURRENT_SPRINT.md`
