# Finding A Real First Issue In Iggy, In Plain English

## Big Idea

After the last commit on April 9, 2026 added [the connector explainer](./connectors-ecosystem-contracts-eli5.md), we did a follow-up search for simple non-connector issues and learned a very practical rule:

**an open issue is not the same thing as an available first issue.**

## Why It Matters

Think of the GitHub issue list like a workshop wall covered in job cards.

Some cards are fresh.
Some cards already belong to someone.
Some cards were partly handled when a nearby change landed.
Some cards are old and no longer match the real machine.

So if you only read the title or the `good first issue` label, you can accidentally pick a job that is already half-done, socially claimed, or riskier than it looks.

That matters even more in Iggy, because the last commit already showed us something important:

- connector work is active
- connector work also touches trust, replay, and delivery rules
- that makes connector work a worse first lane for a newcomer who just wants a clean win

The last commit was:

- `d1db7635d`
- `Add connector contracts ELI5 note`

That note said, in simple terms:

- connectors are not just "build one more bridge"
- they are now also about the rules that say when the bridge really worked

So our follow-up question became:

> if connectors are the tricky lane, what is actually a safe first issue outside connectors?

## Core Ideas Made Simple

### 1. The last commit taught one lesson

Connector work now feels less like snapping together toy blocks and more like writing traffic rules for a busy road.

The hard part is not only making the connector exist.

It is also deciding:

- when the work counts as done
- what happens after retries
- what happens after restarts
- whether the system can say "delivered" honestly

That is why "avoid connectors for a first PR" is a sensible instinct here.

### 2. This follow-up taught a second lesson

A `good first issue` label is like an old sticky note on a drawer.

Sometimes it is still right.
Sometimes someone forgot to peel it off.

So we had to stop doing "label reading" and start doing "reality checking."

### 3. Reality checking means reading more than the title

For each candidate issue, the useful checklist was:

1. Read the issue body, not just the title.
2. Read the comments to see if someone already claimed it.
3. Check the issue timeline for cross-referenced PRs.
4. Search for related PRs by issue number or keywords.
5. Peek at the actual code to see whether the work is already partly there.

This is slower than guessing.

It is also much cheaper than doing duplicate work.

### 4. Why issue `#3000` was a good correction

At first, issue [#3000](https://github.com/apache/iggy/issues/3000) looked like a neat first task:

- add `context show`
- add `session status`
- stay outside connector work

But after checking more carefully, it stopped looking like a clean beginner pick.

The issue timeline shows a cross-reference to PR [#2998](https://github.com/apache/iggy/pull/2998), and that PR merged on **March 30, 2026** after changing the same general CLI workflow area.

That does **not** prove `#3000` is fully solved.

It **does** mean:

- this area was already being reshaped
- the issue is more "adjacent to recent work" than "fresh empty task"
- it was a weaker recommendation than it first looked

That was the right correction to make.

### 5. The best remaining picks were smaller and cleaner

After the wider pass, two issues stood out more clearly.

#### Pick A: [#2148](https://github.com/apache/iggy/issues/2148) `Add more BDD test scenarios`

Why it still looks reasonable:

- still open
- still unassigned
- no obvious open PR tied to it
- maintainer comment suggests it is still wanted

Why it is not perfect:

- the issue is broad if you treat it like "do all the BDD work"

Why it can still be a good first issue:

- you can scope it down to one small scenario
- shared BDD structure already exists

Simple analogy:

Do not promise to build the whole school.
Offer to paint one classroom properly.

#### Pick B: [#3075](https://github.com/apache/iggy/issues/3075) `Address validation for QuicClientConfigBuilder and HttpClientConfigBuilder`

Why it looks technically clean:

- TCP and WebSocket builders already validate addresses
- QUIC and HTTP builders still do not
- the gap is visible in the code

Why it is not a "just start coding" pick:

- the opener said they wanted to be assigned

So the technical answer is:

- small and real

But the social answer is:

- comment first before touching it

### 6. Several tempting issues turned out to be bad picks

These are useful examples because they show why checking matters.

- [#3030](https://github.com/apache/iggy/issues/3030): someone already said PR `#3069` was opened for it
- [#2396](https://github.com/apache/iggy/issues/2396): another contributor already said they wanted to work on it
- [#2776](https://github.com/apache/iggy/issues/2776): there is already related version-sync machinery in the repo, and the issue already has a detailed "I will take this" plan comment
- [#1986](https://github.com/apache/iggy/issues/1986): the Go BDD suite already uses shared scenarios, so the issue is stale
- [#2699](https://github.com/apache/iggy/issues/2699): it looked tiny, but the earlier PR was closed after review because the obvious fix risked breaking HTTP API behavior

This is the heart of the lesson:

some issues are open in GitHub, but closed in spirit

## Tiny Example

Bad shortcut:

```text
This title looks small.
This label says "good first issue."
I should start coding now.
```

Better shortcut:

```text
Read the issue.
Read the comments.
Check the timeline.
Search for related PRs.
Look at the code.
Then decide.
```

Real example:

- `#3000` looked easy from the title
- the timeline showed nearby merged work in the same CLI area
- so it stopped being the best beginner recommendation

That is not failure.

That is just better map-reading.

## What To Remember

The last commit taught us that connector work is now about rules and trust, not just adding more pieces.

This follow-up taught us the matching GitHub lesson:

**the best first issue is not the one with the nicest label, but the one that is still small, real, and socially unclaimed.**

If you want the shortest version:

- avoid connector work for your first PR
- do not trust labels by themselves
- verify the issue like a detective before treating it like an invitation

Sticky sentence:

**A real first issue is not the card that looks easiest on the wall, but the one that is still truly empty.**
