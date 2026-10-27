# First-campaign feasibility assessment

Evidence as of 2026-10-27. Supports [GitHub #1](https://github.com/joeloverbeck/arkham-unfolded/issues/1); the issue owns selection, acceptance and next work. This is a decision-only assessment by Codex and subordinate source researchers, based on repository `3776b1319ca1de91dab8448088cb51eb7f0bf3d2`. The resulting Git commit binds this report and its evidence. Joel remains the acceptance authority. No rules engine, playable campaign or Bevy application was built or verified.

## Recommendation and decision

**Proceed to one bounded Rust/3D experiment, if Joel accepts the candidate source baseline and the experiment below. Do not activate full-campaign implementation yet.** The evidence supports a concrete difficult interaction with independently derived outcomes. It does not establish that the whole campaign can be implemented and maintained within a tolerable budget.

The meaningful next result is to control a real cross-investigator skill test, understand its token choices and consequences, interrupt it, and resume and narrate its replay. It would test whether this path can reduce rules administration while preserving the player's decisions. It cannot establish campaign completeness or sustained enjoyment.

The strongest alternative is further paper-only reconciliation. That would reduce some ambiguity, but cannot answer whether the intended Rust path preserves suspended choices or whether the tabletop is readable. Recommend one narrow consumer experiment, with source preflight as a stop condition. Parking remains a reasonable choice if the estimated burden below exceeds appetite; no technical failure need be invented to justify it.

The owner decision is acceptance or rejection of this assessment's **candidate legacy source baseline** and selection of the **12-hour experiment with at most two 20-minute human sessions**. These are proposed commitments, not already authorized work. No broad architecture, campaign implementation, readiness label or source-fidelity certification follows from accepting the report.

## What the investigation established

- [Source reconciliation](issue-1-source-reconciliation.md) identifies the original campaign guide, Chapter One Rules Reference, Legacy FAQ 2.5, the Innsmouth token rules and Grimoire 1.1 applicability. Grimoire's legacy delegation is material: replacing all older semantics with the newest book would be unjustified. The candidate uses original card identities plus applicable errata and the pinned taboo overlay; unresolved interpretations remain explicit.
- [Input evidence](issue-1-inputs.md) binds the three live decks and the existing archive. Their versions are Tony **0.5**, Patrice **0.7**, Jacqueline **0.9**, all updated on 2025-10-07 and selecting taboo **9**. The latest returned taboo is **10**, effective **2026-02-19**. The three starting-card overlays affected by taboo are unchanged between those two records. This comparison is an observation at this retrieval, not a permanent latest-version rule.
- [Deck projections](issue-1-decks.json) preserve exact codes and quantities without importing historical descriptions as campaign truth. There are 64 distinct slot codes, including the random-weakness placeholder; with investigators, 66 physical identities. Inventory coverage is useful for binding inputs, not a behavioral estimate or legality proof.
- [Runtime evidence](issue-1-runtime-feasibility.md) inspects the pinned Haskell implementation as secondary evidence and identifies a concrete Bevy candidate. Neither implementation agreement nor documentation constitutes proof of the proposed Rust behavior. No local graphics, save or replay test was performed.

The [evidence manifest](issue-1-evidence.json) gives exact URLs, byte counts and SHA-256 digests for nine fetched inputs. Their raw bytes are retained under ignored `local/issue-1-evidence/`, independently of the unchanged external archive. These are machine-local research copies, not a backed-up evidence release. A clean clone has the report and minimal deck projections, but not the raw files. Reacquire each manifest URL into its named path and compare its digest, or restore a matching retained copy; never silently substitute changed bytes. If exact evidence becomes unavailable, the corresponding provenance claim must be withdrawn. Preserve these copies in owner-controlled durable storage before relying on long-term fidelity or replay claims. No original card images or bulk card text are added to Git.

## Behavioral obligations, derived before architecture

This coarse map combines the original campaign and actual selected-card families. It is deliberately independent of Rust types, ECS components or upstream message constructors. All rows remain unimplemented; selected upgrades, weaknesses and replacement investigators add obligations when known. Source locators and card examples are in the linked appendices.

| Family | Actual pressure and independent checks required |
| --- | --- |
| Source and preparation | Distinguish original/reprinted identities, effective text, printed level and taboo-adjusted cost; reject missing semantics or mismatched input revisions before play. |
| Timing and choices | Rules Reference timing charts, forced/reaction ordering, optional windows, costs and targets; Ward of Protection and Deny Existence must not erase a genuine decision or refund an already-paid cost. |
| Skill tests and replacements | Jacqueline, Promise of Power, Olive McBride, Dark Prophecy, Premonition, Arbiter of Fates and Dark Future require reveal groups, nested replacement choices, cancellation, limits and later continuation. The first worked case covers only a declared subset. |
| Action accounting | Tony's bounty action has restrictions; Quick Thinking, multi-action card effects, fast actions, enemy engagement and attacks of opportunity need separately justified legal/refused choices. |
| Persistent effects and lifecycle | Arcane Research trauma/discounts, slots, charges, exhaustion, durations and once-per-round limits must survive relevant boundaries and expire at the correct time. |
| Hand, discard and deck effects | Patrice's upkeep draw, Watcher from Another Dimension, Winging It, Resourceful and Live and Learn stress simultaneous draw, weakness retention, out-of-play permissions and repeat-test continuations. |
| Cross-investigator effects | Same-location commits, shared locations, damage/horror assignment, another investigator's enemy, ownership/control and unique assets must retain actor identity even with one human controlling everyone. |
| Encounter progression | Three scenarios' acts/agendas, doom, clues, locations, enemy spawn/engage/hunter behavior, encounter searches, victory display and per-investigator scaling need source-derived consequences, including refused actions. |
| Campaign branches | House/Lita/Ghoul Priest outcomes, cultists interrogated/escaped, midnight, the four possible final-scenario Agents sets, randomized woods, trauma, XP, sacrifice and defeat must affect later setup and final records. Successful play alone cannot cover these. |
| Selected content and death | Resolve actual basic weaknesses and legal successor decks before play. The Gathering can kill investigators while survivors continue; the rules require replacement investigators/decks. The final sacrifice resolution adds random Madness weaknesses. These cannot be silently omitted from an all-branches claim. |
| Hidden information and randomness | Preserve deck order, hidden identities, random outcomes and pending decisions; undo stops before new hidden information. Separate fixture control from player permissions. |
| Continuity and presentation | Offline save/load must resume the exact continuation; replay must reproduce the recorded rules/events and expose Next/back/camera controls. Missing art must retain readable rules, legal choices and semantic symbol names. |

Full-campaign deficits include a complete basic-weakness eligibility census and quantity policy, detailed combined replacement expectations beyond the worked case, selected future upgrades, and the replacement-investigator preparation boundary. Recommend supporting only replacements actually selected when required, with an honest preparation block until supported; this is a proposed interpretation of the full-delivery scope, not an automatic expansion to the roster. Until accepted and implemented, the original all-branches promise remains unmet.

## Preparation, successor decks and recovery

Start a fresh campaign from the bound lists, not their descriptive history or linked successor decks. Jacqueline's two Arcane Research copies still impose their legitimate starting trauma. Tony's Internal Injury is fixed; Patrice and Jacqueline each require a real basic-weakness selection. Snapshot the eligible official pool and selection, including restrictions and quantities, then check all resulting behavior support. Do not silently redraw an unsupported selection, ignore it, or sample only implemented cards.

The application must own earned/spent/unspent XP and campaign consequences. An ArkhamDB successor is a proposed replacement list: compare it with the last accepted deck and current campaign record, adjudicate additions/removals, upgrade costs, printed-level legality, taboo adjustments, discounts and mandatory story cards/weaknesses. ArkhamDB's XP fields and predecessor links are evidence, not authority. An unsupported card, overspend, missing required card or source mismatch leaves the prior deck and save intact and the next scenario unavailable. Report the exact cause; recover through a corrected import or separately implemented selected content, then recheck the whole transition before committing it. Never replay spending or reroll weaknesses on retry.

## Proposed first consumer experiment

Use the source appendix's worked **Patrice / Promise of Power / Jacqueline / Rotting Remains** case, from the real starting lists and The Gathering encounter set. Tony is present but need not take a decision. Other replacement assets are absent in this bounded fixture. This deliberately constructed test state is not evidence that campaign setup or an entire scenario works.

Patrice at Jacqueline's location commits Promise of Power to her willpower test. The commit adds a curse; the test has skill 9 against difficulty 3. Jacqueline may decline her ability or use it to reveal three tokens and choose legal cancellations. The appendix derives four traces involving curse and auto-fail, including the adverse choice that deals three horror. The expected consequences come from printed cards and official rules, not the Haskell output. The canceled curse's disposition is explicitly identified as a source-derived interpretation, not a bespoke publisher ruling.

Proposed pass criteria, all independently inspectable:

1. The actual Rust rules path offers legal commit, reaction, cancellation and decline choices, rejects remote/excess commits and reuse after the round limit, resolves the four traces and retains correct token destinations. Source review settles any ambiguity before a trace becomes an asserted oracle.
2. A small Bevy tabletop shows the three investigators, inspected cards and individually distinguishable tokens. Clicks submit the same decisions available to an agent. The player can finish or intentionally terminate the test, understand why horror/token results occurred, and retain manual camera control. Readable substitutes work with the art root absent.
3. Save at the pending cancellation decision, close/reopen offline, and finish with the same options and outcome, without adding another curse or revealing another initial group. Replay the completed record through Next, back and camera override; replay consumes recorded events rather than current rules or new randomness. Corrupt or incompatible records fail without overwriting the original.
4. A bounded input change, such as moving Patrice away before the test, changes commit availability through that same rules path. Privileged state construction and scripted randomness are confined to fixtures. An agent using player observations cannot inspect hidden fixture data.
5. Joel inspects playing and narrating separately, within two sessions of at most 20 minutes total each, including any retry. Automated evidence cannot supply that acceptance. Stop if readability, control burden or repeated reconstruction makes the task unpleasant rather than simply adding more machinery.

Proposed allowance: **12 active agent/engineering hours total**, including at most one hour to reconfirm source/trace sufficiency, and the human ceiling above. Return at the bound whether passing, failing or inconclusive. No paid services, generalized engine/DSL, asset pipeline, bulk card authoring or campaign progression. A small local Rust implementation and minimal scene are sufficient; architecture remains replaceable. Preserve the attempt as isolated experimental work or remove it if rejected. Success permits judging this case only; combined Olive/Dark Prophecy behavior, a materially different real use and return-use evidence remain due before broader/recurrent adoption.

The incumbent is manual rules administration or not playing, with the physical setup burden named in the mission. For this experiment, compare decision clarity and effort with the small worked tabletop case and a simple fixed-camera presentation before investing in camera choreography. This is a comparison proposal; no superiority or experienced benefit was measured in this assessment.

## Burden and feasibility limits

These are uncalibrated planning ranges, not commitments or measured coding rates. They assume this one offline product and selected content, with no broad catalog promise.

| Work | Initial planning range |
| --- | --- |
| Source authoring and adjudication | 40–100 focused hours for independent expectations and interaction review; unresolved rulings can dominate. |
| Rules and selected-content implementation | 150–400 hours for timing/continuations plus scenario, party and campaign behavior. |
| Independent verification and repair | 100–250 hours across legal/refused decisions, adverse branches and interaction families. |
| Tabletop, offline preparation and continuity | 80–200 hours including readable absence of art, actual save/replay recovery and packaging. |
| Human review | First probe capped at 40 minutes; complete delivery likely needs several full three-investigator campaign runs plus adverse/recovery inspections. Budget is not yet accepted. |
| Maintenance | Allow 4–12 hours per material source/dependency update as a provisional reserve; a semantic or persistence change can exceed it and requires a new bounded decision. |

That is roughly **370–950 focused engineering hours** before ongoing maintenance, with at least factor-of-two uncertainty and no finite upper bound while future content selections remain open. Parallel agents do not remove adjudication or human review. The 12-hour probe should replace this speculation with observed authoring, debugging and review costs for one hard case; it cannot validate the total estimate.

The assessment supports a narrow next decision. Complete source fidelity, full campaign execution, reliable long-term compatibility, graphics performance and enjoyable play/narration remain unproven. The issue stays open for the owner's disposition; no dependent work is activated by this document.
