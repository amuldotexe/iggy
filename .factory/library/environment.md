# Environment

Environment variables, external dependencies, and setup notes.

**What belongs here:** Required env vars, external API keys/services, dependency quirks, platform-specific notes.
**What does NOT belong here:** Service ports/commands (use `.factory/services.yaml`).

---

## Code Exploration Preference

**ALWAYS use Parseltongue for querying and reading code.** Only read files directly if the code entity or relationship cannot be found through Parseltongue queries.

Parseltongue server runs at: `http://localhost:7777`

Key endpoints:
- `/code-entities-search-fuzzy?q=<term>` - Find entities by name
- `/reverse-callers-query-graph?entity=<key>` - Find what calls something
- `/smart-context-token-budget?focus=<key>&tokens=4000` - Get contextual code

Run folder: `parseltongue20260311142917/analysis.db`
