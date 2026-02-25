# PR Prep - MongoDB Source (Issue #2739)

## Goal
Prepare a maintainer-friendly, source-only PR from this repo (`iggy_2739_source`) with explicit behavior guarantees and low review friction.

This document is the execution guide for branch `codex/2739-source-sync`.

---

## Current Baseline

- Repo path: `/Users/amuldotexe/Desktop/A01_20260131/iggy-issue02/iggy_2739_source`
- Working branch: `codex/2739-source-sync`
- Base branch: `upstream/master`
- Expected start state: clean tree, zero diff vs `upstream/master`

Quick verify:

```bash
git fetch upstream origin --prune
git status --short --untracked-files=all
git diff --name-only upstream/master...HEAD
```

---

## Maintainer Expectations (Observed)

- Narrow scope and clear rationale are favored.
- Connector PRs with broad mixed scope trigger longer review cycles.
- Local execution evidence matters more than broad claims.
- Behavior risks in sources (state/checkpoint correctness) get high scrutiny.

For this PR:
- keep scope source-only
- include exact local checks in PR body
- explicitly state delivery semantics and known limitations

---

## Scope Contract (Source PR)

Include:

- `core/connectors/sources/mongodb_source/**`
- source integration tests/fixtures/wiring only
- minimal shared file changes required to compile and run source tests
- dependency metadata updates required by source diff

Exclude:

- all sink connector code (`mongodb_sink`)
- unrelated docs/journal artifacts
- unrelated runtime refactors

---

## Transplant Plan (From Reference Branch)

Source of truth branch: `codex/2739-ref` (in sibling repo `iggy_2739_ref`).

```bash
git restore --source codex/2739-ref -- \
  Cargo.toml \
  core/integration/Cargo.toml \
  core/connectors/sources/mongodb_source \
  core/integration/tests/connectors/mod.rs \
  core/integration/tests/connectors/mongodb/mod.rs \
  core/integration/tests/connectors/mongodb/mongodb_source.rs \
  core/integration/tests/connectors/mongodb/source.toml \
  core/integration/tests/connectors/fixtures/mod.rs \
  core/integration/tests/connectors/fixtures/mongodb/container.rs \
  core/integration/tests/connectors/fixtures/mongodb/mod.rs \
  core/integration/tests/connectors/fixtures/mongodb/source.rs
```

Fallback (if local ref branch unavailable):

```bash
git restore --source origin/ab_202602_issue02 -- <same-path-list>
```

---

## Mandatory Cleanup After Transplant

Shared files often pull both sink and source. Keep source-only content:

- `core/integration/tests/connectors/mongodb/mod.rs`
  - keep only: `mod mongodb_source;`
- `core/integration/tests/connectors/fixtures/mongodb/mod.rs`
  - keep `container` + `source` only
- `core/integration/tests/connectors/fixtures/mod.rs`
  - export source fixtures only
- `Cargo.toml`
  - add source connector member only (no sink member)

Leak check:

```bash
rg -n "mongodb_sink|MongoDbSink|ENV_SINK|sink.toml" \
  core/integration/tests/connectors core/connectors Cargo.toml
```

Expected output: no sink references relevant to this PR.

---

## Source-Specific Correctness Checklist

Before opening PR, verify these are true:

- Checkpoint advances only after successful mark/delete operation.
- Mark/delete use the current batch max offset (not stale persisted offset).
- Mismatch telemetry covers:
  - complete mismatch (`actual == 0`)
  - partial mismatch (`0 < actual < expected`)
- ObjectId conversion is applied only when tracking field is `_id`.
- ObjectId mark test asserts both:
  - all processed documents set to `true`
  - unprocessed count is `0`

---

## Quality Gates (Run Before Commit)

```bash
cargo fmt --all -- --check
cargo clippy -p iggy_connector_mongodb_source --all-targets -- -D warnings
cargo test -p iggy_connector_mongodb_source
cargo test -p integration --test mod -- mongodb_source
```

If dependency graph changed:

```bash
cargo check -p iggy_connector_mongodb_source
```

---

## Commit Hygiene

Stage explicit paths only:

```bash
git add core/connectors/sources/mongodb_source \
  core/integration/Cargo.toml \
  core/integration/tests/connectors \
  Cargo.toml Cargo.lock DEPENDENCIES.md
```

Never use:

- `git add .`
- `git add -A`

Commit message:

```text
feat(connectors): add MongoDB source connector
```

Push:

```bash
git push -u origin codex/2739-source-sync
```

---

## PR Body Template (Use Upstream Structure)

Use upstream `PULL_REQUEST_TEMPLATE.md` sections exactly:

```md
## Which issue does this PR close?
Closes #2739

## Rationale
Need a MongoDB source connector to ingest documents into Iggy with explicit checkpointing and delivery semantics.

## What changed?
Added MongoDB source connector crate, config, and README.
Added source-focused integration tests and required fixture wiring.
Checkpoint/mark/delete behavior is validated and telemetry is explicit for mismatch scenarios.
Kept scope source-only; MongoDB sink is intentionally excluded to a separate PR.

## Local Execution
- Passed: `cargo fmt --all -- --check`
- Passed: `cargo clippy -p iggy_connector_mongodb_source --all-targets -- -D warnings`
- Passed: `cargo test -p iggy_connector_mongodb_source`
- Passed: `cargo test -p integration --test mod -- mongodb_source`
- Pre-commit hooks ran: yes

## AI Usage
Tools: Codex/ChatGPT
Scope: implementation support, test drafting, review prep
Verification: local checks above + integration tests
Can explain all changes: yes
```

Open PR:

```bash
gh pr create \
  --repo apache/iggy \
  --base master \
  --head amuldotexe:codex/2739-source-sync \
  --title "feat(connectors): add MongoDB source connector" \
  --body-file /tmp/pr-2739-source.md
```

---

## Should We Add GHCR Demo Image Here?

No, for this PR.

Reason:
- adds non-essential CI/release complexity
- source connector merge criteria are mostly correctness + tests + scope discipline

If desired, propose GHCR/demo packaging in a separate follow-up.

---

## Final Pre-Open Checklist

- [ ] Diff is source-only and reviewable.
- [ ] No sink connector paths in this PR.
- [ ] Source correctness checklist is satisfied.
- [ ] All quality gates passed locally.
- [ ] PR body includes exact local execution evidence.
- [ ] Branch pushed and tracking origin.

---

## Fast Triage Commands

```bash
git status --short --untracked-files=all
git diff --name-only upstream/master...HEAD
gh pr checks --watch
gh pr view --comments
```
