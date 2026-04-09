# Connector Contracts And Why Things Feel Slow, In Plain English

## Big Idea

Apache Iggy can already build real connectors, but it has now reached the stage where the hardest problem is not "more connectors." It is "clear rules for what a connector is allowed to promise."

## Why It Matters

People do not usually wake up wanting "a message broker."

They want something more concrete:

- take data from one system
- move it into Iggy
- send it somewhere else
- trust that it did not silently vanish on the way

That is why connectors matter so much.

They are the bridges between Iggy and real jobs.

But once a project has a few real bridges, the next question is not just "can we build another bridge?"

It becomes:

- when do we say the bridge worked?
- what happens if the truck breaks halfway across?
- after a restart, do we retry, skip, duplicate, or lose cargo?

That is the stage Iggy is in now.

## The Simple Story

Think of Iggy like a big mail building.

- A **source connector** is a robot that brings letters into the building.
- A **sink connector** is a robot that carries letters out to somewhere else.
- The **runtime** is the building manager.

The manager starts the robots, gives them instructions, watches if they are healthy, and stops them when needed.

So when people say "connector problems," they usually mean one of three things:

1. the robot itself
2. the rules the robot must follow
3. the manager who supervises the robot

## What The Latest GitHub History Says

From the latest GitHub capture on April 9, 2026:

- **121 PRs** merged in the last month
- most work was in:
  - quality and release hardening
  - SDK surface
  - core runtime / protocol work
- connector work was active, but it was a smaller slice

The important connector moves in that same window were:

- [MongoDB sink #2815](https://github.com/apache/iggy/pull/2815)
- [HTTP sink #2925](https://github.com/apache/iggy/pull/2925)
- [InfluxDB sink and source #2933](https://github.com/apache/iggy/pull/2933)
- [restart connector without runtime restart #2781](https://github.com/apache/iggy/pull/2781)

There was also a lot of connector-related cleanup:

- flaky integration test fixes
- readiness fixes
- semantics documentation fixes
- race-condition fixes in source tests

So the repo is not frozen.

It is moving.

But the movement is telling us something important:

> the team is growing the connector ecosystem, while also spending a lot of time making the floor under it less shaky

## What We Now Know More Clearly

### 1. Source connectors are usable, but only for a certain shape of work

The current source framework is best for:

- poll-based connectors
- simple checkpointing
- one main destination flow

That means it is good for robots that:

- wake up
- check one shelf
- pick up new boxes
- carry them to one room

It is much worse for robots that must:

- accept push traffic from the outside
- sort each message to different rooms
- keep complicated live routing rules

This is not just a feeling.

The local code still shows the same framework pressure:

- [ProducedMessage](/Users/amuldotexe/Desktop/notebook-gh/hogwarts202603/research20250315/iggy/core/connectors/sdk/src/lib.rs#L309) does not carry per-message destination routing
- the runtime loops configured source streams but keeps the **last** producer and encoder in [source.rs](/Users/amuldotexe/Desktop/notebook-gh/hogwarts202603/research20250315/iggy/core/connectors/runtime/src/source.rs#L246)
- source shutdown still removes the callback sender before closing the source in [source manager](/Users/amuldotexe/Desktop/notebook-gh/hogwarts202603/research20250315/iggy/core/connectors/runtime/src/manager/source.rs#L151)

ELI5:

- a simple delivery cart works
- a smart sorting machine is still awkward

That is why shiny source ideas like webhook gateways look exciting but quickly hit framework limits.

### 2. Sink connectors have the bigger trust problem

The sink side has the scarier question:

> did we mark the work as done before the outside system really got it?

The shared runtime still shows the same problem:

- sink consumers use `PollingMessages` auto-commit in [sink.rs](/Users/amuldotexe/Desktop/notebook-gh/hogwarts202603/research20250315/iggy/core/connectors/runtime/src/sink.rs#L421)
- the runtime still calls the sink `consume()` callback in [sink.rs](/Users/amuldotexe/Desktop/notebook-gh/hogwarts202603/research20250315/iggy/core/connectors/runtime/src/sink.rs#L585)
- and that exact area is what [#2927](https://github.com/apache/iggy/issues/2927), [#2928](https://github.com/apache/iggy/issues/2928), and [#2940](https://github.com/apache/iggy/issues/2940) are about

Simple analogy:

- imagine a courier app marking a package "delivered"
- before the courier reaches the house

If the courier crashes after that, the app lies.

That is why sink problems are not just "bugs."

They are trust problems.

## The Two Big Missing Pieces

### Missing Piece 1: Framework Fit

This means:

- does the current SDK/runtime naturally fit the connector idea?

Right now, the answer is:

- **yes** for simple poll-based sources
- **yes-ish** for many bounded sinks
- **no** for more advanced push-based or multi-route sources

This is why source selection feels so annoying.

A connector can be:

- important
- popular
- serious

and still be a bad first move if the current framework shape fights it.

That is what happened with the HTTP source discussion in [#3039](https://github.com/apache/iggy/discussions/3039).

It did not fail because the idea was bad.

It hit real framework edges.

### Missing Piece 2: Definition Of Done

This means:

- what exact behavior must every connector prove before everyone says "yes, this is good enough to merge"?

That shared mental model still is not fully landed.

You can see the project trying to build it through:

- [Pareto Source Suite #2892](https://github.com/apache/iggy/issues/2892)
- [Pareto Sink Suite #2893](https://github.com/apache/iggy/issues/2893)

These look like test issues, but they are actually asking something deeper:

- when does progress move forward?
- when is a write successful?
- what can replay?
- what can duplicate?
- what can be lost?

ELI5:

they are not just more exams

they are trying to define the syllabus

Without that syllabus, every serious connector PR quietly reopens the same argument.

## Why Things Suddenly Feel Slow

This is the heart of it.

A few months ago, the connector story was closer to:

```text
find missing connector
build connector
merge connector
```

Now it is more like:

```text
find missing connector
check if it is already owned
check if the framework can express it cleanly
check what success and replay mean
check whether the tests prove the claim
then maybe merge connector
```

So the project has moved from:

- **construction mode**

to:

- **constitution mode**

Construction mode means:

- build more things

Constitution mode means:

- decide the shared rules that all future things must follow

That always feels slower.

But it is also more real.

## What The Repo Seems Focused On Right Now

The last month does **not** look like "connectors are everything."

It looks more like:

1. make the core stronger
2. make test and release machinery more reliable
3. keep external surfaces moving

In connector land specifically, the pattern looks like this:

### What is clearly happening

- more connectors are still being added
- restart/config/lifecycle experience is improving
- integration tests are getting less flaky
- readiness checks and docs are being corrected

### What is not fully attacked yet

- shared sink truth model
- shared source routing model
- a fully settled connector definition-of-done

So the project direction is:

> wider ecosystem, stronger harness, slower contract settlement

## The Ecosystem Discussion Is Really About Two Things

The main tracker is:

- [Connector ecosystem tracking #2756](https://github.com/apache/iggy/discussions/2756)

At first glance it looks like a big shopping list of connectors.

But when you follow the issues and PRs around it, it is really about two different jobs:

1. deciding **what connectors to have**
2. deciding **what a trustworthy connector means**

That second job is the harder one.

It is why the Pareto suites matter so much.

It is why the HTTP sink discussion in [#2919](https://github.com/apache/iggy/discussions/2919) became important.

And it is why the HTTP source discussion in [#3039](https://github.com/apache/iggy/discussions/3039) exposed framework-level questions instead of being "just another connector idea."

## The Dr Strange Timelines

This part is an informed simulation, not a GitHub fact.

It is the "what probably happens next if we choose this path?" view.

### Timeline A: Keep Adding Connectors Fast

What happens:

- the ecosystem list gets longer
- contributors feel visible progress
- more connector names show up in the repo

What likely goes wrong:

- reviews keep reopening the same behavior questions
- semantics stay connector-by-connector
- trust grows slower than breadth

This timeline looks good from far away and messy up close.

### Timeline B: Land The Pareto Suites First

What happens:

- the project gets a small common exam for connectors
- reviews become more consistent
- newcomers can validate work more easily

What likely improves:

- less repeated debate
- clearer source and sink behavior
- easier "what does done mean?" conversations

This is the highest leverage with the lowest drama.

### Timeline C: Fix Sink Failure Visibility First

What happens:

- failures stop being silent
- metrics and logs become more honest
- operators understand what is going wrong

What it does **not** solve by itself:

- true at-least-once sink behavior
- partial-write policy
- manual commit timing

This is a strong first aid step, but not the whole cure.

### Timeline D: Solve Sink Contract First

What happens:

- the hardest sink questions get addressed head-on
- the project gets more honest replay and commit rules
- sink credibility jumps the most

What likely costs:

- slower short-term connector shipping
- more design debate
- more careful reviews

This is the bravest timeline and the hardest one.

### Timeline E: Fix Source Framework Fit First

What happens:

- advanced source ideas become more realistic
- webhook-style and multi-route sources become less awkward
- the source side becomes easier to extend

What likely costs:

- SDK/runtime design work
- slower near-term visible connector additions

This is the best path if the project wants stronger source ambitions.

## If You Are New And Want Good Reputation

The safest lesson is:

do not try to become the hero who rewrites all connector semantics in one PR

The better move is:

1. make one fuzzy rule visible
2. make one small test prove it
3. help the team talk about connector behavior in the same language

That is why the most reputation-safe high-value work is still:

- [Pareto Source Suite #2892](https://github.com/apache/iggy/issues/2892)
- [Pareto Sink Suite #2893](https://github.com/apache/iggy/issues/2893)
- narrow follow-ups around runtime visibility
- small framework fixes like safer source shutdown order

Those are not flashy.

But they make the whole ecosystem easier to trust.

## If You Still Want A New Source Connector

If the goal is "serious but low-hanging," the earlier reasoning still mostly stands:

- best serious first bet: **Amazon S3 source**
- other serious lower-hanging ideas:
  - **Amazon SQS source**
  - **RabbitMQ source**
  - **local filesystem source** as a simpler but less strategic option

These fit the current source model better because they can be scoped as:

- poll
- batch
- checkpoint
- send to one topic

The ideas that still look exciting but not low-hanging are:

- HTTP source
- MQTT source
- Kafka source
- JDBC/MySQL source
- MongoDB source

Not because they are bad ideas.

Because they either:

- are already owned
- need stronger framework fit
- or reopen the same hard behavior questions

## Tiny Example

Here is the whole connector-contract problem in toy form.

```text
source reads 10 messages
    |
    v
runtime tries to move them
    |
    +--> if sending succeeds, save progress
    |
    +--> if sending fails, do not pretend progress moved
    |
    +--> if stopping, do not drop the last few messages
```

And here is the sink version:

```text
sink receives 10 messages
    |
    v
sink writes outside
    |
    +--> if write succeeds, move progress
    |
    +--> if write fails, do not lie
    |
    +--> if only 5 succeed, say clearly what happens next
```

Without a shared contract, each connector tells this story a little differently.

With a shared contract, every connector has to tell the same truth.

## What To Remember

- Iggy connectors are real and moving forward.
- The slowdown is not laziness. It is a sign that the project is now deciding shared rules.
- The biggest missing pieces are:
  - framework fit for advanced sources
  - definition of done for all connectors
- The repo is still shipping connector breadth, but it is also spending real energy on validation and stabilization.
- The best leverage now is often not "one more connector."
- It is "make connector truth easier to state, easier to test, and easier to review."

The sticky sentence is:

**Iggy is no longer just building more connector robots; it is finally writing the traffic laws they all have to obey.**
