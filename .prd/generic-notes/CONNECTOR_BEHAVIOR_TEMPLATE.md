# Connector Behavior Template

Use this before writing any new connector PRD, implementation, or PR.

The goal is simple:

- force the behavioral boundary to be explicit
- prevent runtime-vs-connector confusion
- convert expectations into tests before code review

This template is intentionally biased toward execution and correctness, not narrative product prose.

## 1. Connector Identity

### Basic facts

- Connector name:
- Direction: `source` | `sink` | `source+sink`
- External system:
- Connector family: `sql` | `document_db` | `object_store` | `search` | `broker` | `timeseries` | `other`
- Planned crate path:
- Owner:
- Related issue / discussion:

### Why this connector exists

- Primary user story:
- Secondary user story:
- Why now:
- Why this backend instead of another one:

## 2. Capability Manifest

Fill this in before implementation.

```yaml
direction: source | sink
delivery: at_most_once | at_least_once | effectively_once_with_idempotent_key
restart_resume: committed_state_only | best_effort | not_supported
ordering_scope: none | per_partition | per_key | global
state_model: none | local_file | runtime_managed | external_system
idempotency_basis: none | deterministic_key | upsert | destination_native
partial_batch_behavior: impossible | possible_detectable | possible_ambiguous
destructive_side_effects: none | delete_after_commit | mark_after_commit | other
requires_unique_tracking_field: true | false
supports_projection: true | false
supports_filter_pushdown: true | false
supports_schema_evolution: true | false
```

## 3. Mandatory Questions

These are the questions that should be asked for every connector.

### Product and scope

1. What is the smallest useful version of this connector?
2. What specific user workflow should work on day one?
3. What is explicitly out of scope for v1?
4. Is this connector a thin adapter over an existing family pattern, or a new behavioral shape?

### Data boundary

5. What is the unit of data?
   - row
   - document
   - object
   - event
   - metric point
   - file
6. How does external data map to an Iggy message?
7. Is the mapping one-to-one, one-to-many, or many-to-one?
8. What metadata must be preserved?
9. What identity exists on the external side?
10. Can that identity be made deterministic?

### Delivery semantics

11. What delivery semantics are honestly achievable on the current runtime?
12. Where is the commit frontier?
13. What exact event means “success”?
14. Can success be partial?
15. What happens if the process crashes:
    - before polling
    - after polling
    - after partial external success
    - after full external success but before state save
16. Can the connector safely retry the same logical batch?
17. If duplicates happen, are they acceptable, visible, or harmful?
18. If loss happens, under which exact failure window can it happen?

### Source-specific questions

19. What field or cursor defines progress?
20. Is that field unique?
21. Is that field monotonic?
22. What happens if many records share the same boundary value?
23. What side effects exist after successful delivery?
24. Can side effects partially succeed?
25. Does state need typed persistence to preserve comparison semantics?

### Sink-specific questions

26. When does the destination durably acknowledge a write?
27. Can a batch partially succeed?
28. Can the destination return exact successful-write counts?
29. Does the sink support idempotent upsert or deterministic keys?
30. If a retry happens, how do we avoid duplicate writes or permanent loops?

### Query and filtering

31. What filtering can be pushed down?
32. Can user filters conflict with checkpoint filters?
33. How are filters composed?
34. What projection rules exist?
35. Can projection accidentally remove required tracking fields?

### Types and schema

36. What payload formats are supported?
37. What field or column types are supported for progress tracking?
38. Which types are explicitly unsupported?
39. Are unsupported types rejected at startup or at runtime?
40. Is schema evolution tolerated, ignored, or validated?

### Ordering and partitioning

41. What ordering, if any, is guaranteed?
42. Is ordering global or scoped?
43. Can parallelism violate ordering expectations?
44. How does partitioning map between Iggy and the external system?

### Operations

45. What configuration is mandatory?
46. What defaults are safe?
47. What defaults are only convenient but potentially dangerous?
48. What metrics should be exposed?
49. What errors should become warnings vs hard failures?
50. What does an operator need to know during incident response?

## 4. Required Decisions

These must be stated explicitly. No blanks.

### Delivery statement

`This connector provides __________ semantics because __________.`

### Restart statement

`After restart, this connector resumes from __________.`

### Duplicate statement

`Duplicate delivery/write may occur when __________.`

### Loss statement

`Message loss may occur when __________.`

### Partial success statement

`Partial success is handled by __________.`

### Identity statement

`External record identity is derived from __________.`

## 5. Executable Requirements

Write only measurable contracts.

### REQ-CONN-001.0: Happy-path delivery

**WHEN** the connector processes a valid batch under normal conditions  
**THEN** the system SHALL deliver all records to the configured destination  
**AND** SHALL preserve the declared payload mapping  
**SHALL** expose success through connector metrics or observable destination state.

### REQ-CONN-002.0: Failure boundary

**WHEN** the connector fails during batch processing  
**THEN** the system SHALL behave according to the declared delivery semantics  
**AND** SHALL NOT over-report success  
**SHALL** surface failure clearly in logs, status, or metrics.

### REQ-CONN-003.0: Restart behavior

**WHEN** the connector restarts after previously processing data  
**THEN** the system SHALL resume from the declared restart frontier  
**AND** SHALL match the documented duplicate/loss behavior.

### REQ-CONN-004.0: Unsupported configuration

**WHEN** the user configures an unsupported mode, type, or field  
**THEN** the connector SHALL fail with an explicit error  
**SHALL** identify the unsupported option  
**AND** SHALL document the supported alternatives.

Add connector-specific requirements after these.

## 6. Test Matrix

Every connector should have these tests at minimum.

| req_id | test_id | type | assertion | target |
| --- | --- | --- | --- | --- |
| REQ-CONN-001.0 | TEST-CONN-001 | integration | happy-path batch completes | every connector |
| REQ-CONN-002.0 | TEST-CONN-002 | integration | injected failure does not over-report success | every connector |
| REQ-CONN-003.0 | TEST-CONN-003 | restart | restart behavior matches documented frontier | every connector with state |
| REQ-CONN-004.0 | TEST-CONN-004 | unit/integration | unsupported config fails clearly | every connector |

### Additional required tests for sources

| req_id | test_id | type | assertion | target |
| --- | --- | --- | --- | --- |
| REQ-SRC-001.0 | TEST-SRC-001 | integration | checkpoint advances only after successful downstream send | every source |
| REQ-SRC-002.0 | TEST-SRC-002 | restart | same batch is re-offered after injected send failure | every source with state |
| REQ-SRC-003.0 | TEST-SRC-003 | integration | side effects happen only after commit | sources with delete/mark behavior |
| REQ-SRC-004.0 | TEST-SRC-004 | unit/integration | duplicate boundary handling matches spec | sources with non-unique tracking risk |

### Additional required tests for sinks

| req_id | test_id | type | assertion | target |
| --- | --- | --- | --- | --- |
| REQ-SINK-001.0 | TEST-SINK-001 | integration | sink documents current semantics honestly | every sink |
| REQ-SINK-002.0 | TEST-SINK-002 | integration | partial external failure is not silent | sinks with partial-ack risk |
| REQ-SINK-003.0 | TEST-SINK-003 | restart | replay/restart behavior matches documented frontier | every stateful sink |
| REQ-SINK-004.0 | TEST-SINK-004 | integration | deterministic identity prevents simple duplicate collisions | sinks with keyed writes |

## 7. Minimal TDD Plan

### STUB

1. Write the happy-path integration test.
2. Write the failure-boundary test.
3. Write the restart test.
4. Write the unsupported-config test.

### RED

1. Run the tests and confirm the failure reason matches the intended contract gap.

### GREEN

1. Implement the smallest code needed to satisfy the contracts.

### REFACTOR

1. Simplify code and consolidate conversion, checkpoint, or identity logic.

### VERIFY

1. Run unit tests.
2. Run integration tests.
3. Verify docs match actual runtime behavior.

## 8. Review Checklist

Before opening a PR, answer yes or no to each:

- Does the doc state delivery semantics without hand-waving?
- Is the commit frontier explicit?
- Is duplicate behavior documented?
- Is loss behavior documented?
- Is partial success behavior documented?
- Are unsupported types/configurations rejected clearly?
- Are source/sink side effects aligned with the runtime boundary?
- Do tests prove behavior, not just storage success?
- Does the README avoid stronger claims than the runtime can provide?

## 9. Fast Start Version

If the maintainer wants a short version, answer these 12 questions only:

1. Source or sink?
2. What is the external unit of data?
3. What is the Iggy message mapping?
4. What is the progress/identity field?
5. Is it unique and monotonic?
6. What does success mean exactly?
7. Can success be partial?
8. When can duplicates happen?
9. When can loss happen?
10. What happens on restart?
11. What is the smallest failure-injection test?
12. What semantics can the current runtime honestly claim?

## 10. Suggested File Naming

Use one of these:

- `CONNECTOR_<NAME>_SPEC.md`
- `CONNECTOR_<NAME>_BEHAVIOR_SPEC.md`
- `CONNECTOR_<NAME>_EXECUTABLE_SPEC.md`

Examples:

- `CONNECTOR_MONGODB_SOURCE_BEHAVIOR_SPEC.md`
- `CONNECTOR_S3_SINK_EXECUTABLE_SPEC.md`
- `CONNECTOR_REDIS_SOURCE_SPEC.md`
