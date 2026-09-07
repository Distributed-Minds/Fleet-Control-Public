# Agent state record

Append one line per run to the fleet coordination issue:

```text
AGENT_STATE | agent=<id> | mode=<PLAN|PREDICT|AUDIT|BUILD|INTEGRATE> | phase=<SOLO|ANALYTIC|BOOTSTRAP|FREE> | noop_streak=<n> | last_result=<MATERIAL|NOOP|BOOTSTRAP> | last_build=<PROGRESS|NO-PROGRESS|none> | ts=<ISO8601> | note=<short>
```

Do not edit or replace another agent's state history.
