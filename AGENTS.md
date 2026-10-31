# AGENTS.md: offcut agent contract

Estate contract: WyattAu/engineering-standards (AGENTS.md + OMNI-CORE
invariants). Pointer, not fork; code gates and supply chain are owned by
engineering-standards until this repo grows its own matrix (ROADMAP R4/R0).

## Non-negotiables

1. **Vendored upstream code stays under its LICENSE.** Do not relicense,
   strip headers, or reword attribution. NOTICE.md and docs/UPSTREAM.md are
   load-bearing.
2. **Cherry-pick discipline.** Upstream fixes are applied against the seed
   commit recorded in docs/UPSTREAM.md; keep the diff to upstream reviewable.
3. **Measured claims only.** Benchmarks, gate results, and comparisons to
   upstream carry the run date and the command that produced them.
4. **Roadmap discipline.** docs/ROADMAP.md is the plan of record; a change
   that contradicts it gets an ROADMAP edit in the same commit.
5. **Secrets:** key-file pattern per estate-keys skill; never committed.
