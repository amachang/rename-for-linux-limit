# Session Reflections

This directory captures structured reflections on session failures and improvement opportunities.

## Structure

This is a **Workspace**: README.md provides overview, dated files contain individual reflections.

## Workflow

1. When a failure occurs during a session, run `/side-reflect <brief description>`
2. The orchestrator analyzes the session and creates `YYYYMMDD-<reflection-slug>.md`
3. Periodically review accumulated reflections to identify patterns

## Reflection Format

Each reflection includes:

- **Trigger**: The user's original observation or complaint
- **Session Context**: What was happening in the session
- **What Happened**: Analysis of the failure
- **Scope Analysis**: Where the issue lives (system prompt, command, tooling, or combination)
- **Improvement Ideas**: Subjective suggestions for prevention

## Scope Categories

Reflections may identify issues across multiple scopes:

- **System Prompt**: Core orchestration behavior
- **Command**: Specific workflow procedures
- **Tooling**: Opportunities for automation (hooks, checks, etc.)
- **Deferred**: Issues better addressed by future tooling improvements

Not all issues require immediate action. Some are best left for future tooling capabilities.

## Documentation Lifecycle

This directory is a Workspace: individual reflection files are dated and become historical records over time. The README.md provides the current overview.
