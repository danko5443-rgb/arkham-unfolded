# Issue #1: candidate sources and first rules case

Supporting evidence for [GitHub #1](https://github.com/joeloverbeck/arkham-unfolded/issues/1), inspected 2026-10-27 against repository base `3776b1319ca1de91dab8448088cb51eb7f0bf3d2`. This is a Codex research proposal for Joel's feasibility decision, not an accepted rules release or gameplay verification. No Rust consumer was executed.

## Candidate source baseline

Use the Chapter One rules for the original Night of the Zealot and the supplied pre-2026 card printings. The live [publisher support page](https://www.fantasyflightgames.com/product/arkham-horror-the-card-game/) still separates Chapter One rules/FAQ from Chapter Two. Downloads below were inspected as PDF text; document names without numbered editions are bound by their bytes, not their upload-folder date.

| Source | Identity and relevant locators |
| --- | --- |
| [Rules Reference](https://cdn.svc.asmodee.net/production-fantasyflightgames/uploads/2026/09/ahc60_rules_reference_eng-compressed.pdf) | AHC60 English, 32 pages; p2 Golden Rules, pp5–6 campaign play, p6 Cancel, p17 simultaneous resolution, pp22–26 timing, p27 setup. This is the publisher's currently linked Chapter One reference, not a 2026 rules edition inferred from its URL. |
| [Original Night of the Zealot guide](https://cdn.svc.asmodee.net/production-fantasyflightgames/uploads/2026/09/night_of_the_zealot_campaign_guide.pdf) | Eight pages; p1 campaign/bag setup, pp2–3 Gathering, pp4–5 Midnight Masks, pp6–7 Devourer Below, p8 campaign log. Original campaign, without Return To or optional refractions. |
| [Legacy FAQ 2.5](https://images-cdn.fantasyflightgames.com/filer_public/c1/d0/c1d0fab6-7fa6-4ce2-af6a-16416381a19b/ahc_faq_v25_february_2026-web.pdf) | February 2026, 30 pages; p2 original-English/errata precedence; §1.5 choices; §2.5 multiple tokens; §2.9 automatic outcomes; §2.20 encounter/player conflict; p17 nested reveal example; pp26–27 taboo list. |
| [Innsmouth Investigator Expansion insert](https://cdn.svc.asmodee.net/production-fantasyflightgames/uploads/2026/09/ahc81_rules_insert.pdf) | AHC81, two pages; p1 bless/curse rules. Required even though the campaign is Night of the Zealot: Patrice's actual Promise of Power introduces curses. |
| [Arkham Grimoire 1.1](https://images-cdn.fantasyflightgames.com/filer_public/5c/c7/5cc79eb6-3d9c-4345-a5f1-27051251a6e4/arkham_grimoire_v11_web.pdf) | 48 pages; pp2,41 delimit pre-2026 material; p14 Ignore; p44 modified reprints; p46 optional legacy encounter substitution. Reconciliation evidence, not blanket replacement of the Chapter One baseline. |

Grimoire pp2 and 41 explicitly direct pre-2026 material to the retired FAQ. Its p46 identifies original-core encounter sets as the intended legacy experience and presents new-core substitutions as an alternative. The [Grimoire launch announcement](https://www.fantasyflightgames.com/hidden-truths/) calls it the resource for current/future clarifications while retaining the earlier reference for legacy content. Together these support the proposed baseline; they do not justify silently rewriting this campaign under every Chapter Two change.

The candidate precedence is: apply applicable errata to original English card/rule identities, then the selected taboo overlay; follow specific card/scenario exceptions over general rules; preserve absolute prohibitions and the FAQ's encounter-over-player conflict rule. The Rules Reference outranks Learn to Play. Expansion rules supply mechanics absent from the core. ArkhamDB is transport/transcription evidence, and the Haskell app is secondary behavioral evidence; neither overrides these sources. Unresolved contradiction stops the affected preparation or fidelity claim and returns a cited interpretation proposal. The Grim Rule is an in-play expedient, not permission to manufacture permanent engine rulings or remove legal player choices.

Newest ArkhamDB taboo observed by the input assessment is ID `10`, effective `2026-02-19`; the supplied decks select `9`. FAQ 2.5 pp26–27 supply the official overlay. Its relevant changes match the input assessment: Quick Thinking's once-per-round maximum, Voice of Ra's symbol category, and Dark Prophecy's non-elder-sign symbol category. Chaining changes purchase experience, not printed level. Pin the entire selected list for a campaign; recheck availability only when preparing a new campaign.

## Source-derived difficult case

Recommend **Jacqueline resolves Rotting Remains while Patrice commits Promise of Power at the same location**. This uses cards actually present in the selected party and campaign, combining another investigator's commitment, an immediate token-pool change, an optional replacement, cancellation, a further draw, and an adverse outcome. It challenges a flat “draw one token and add a modifier” implementation without adding unrelated cards.

The English card faces were visually inspected in the read-only archive: Jacqueline `60401`, Promise of Power `07032`, Rotting Remains `01163`. The [evidence manifest](issue-1-evidence.json) binds each inspected catalog row, variant, face path and SHA-256; the [input assessment](issue-1-inputs.md) explains the archive's provenance. The images verify the printed text; archive provenance does not become publisher certification. These identities, their original text, and the cited rule sections must remain available for a future independent review.

Candidate initial state: during The Gathering, Jacqueline draws Rotting Remains while Patrice is at her location. Jacqueline has printed willpower 5, no relevant modifiers or assets, and her reaction is unused this round. Patrice has Promise of Power in hand; the bag initially has no curses and otherwise uses the Standard setup. Neither Dark Future nor Arbiter of Fates nor Olive is active. This is a deliberately constructed legal-state fixture, not proof that a whole scenario reached it correctly.

At ST.2, Patrice may commit one appropriate card. Promise supplies four wild icons, so Jacqueline's pre-token skill is 9. Its effect adds one curse immediately after commitment, before ST.3. Jacqueline may decline her reaction, or use it before the reveal and choose the required cancellations after seeing the three tokens. These are separate player decisions. Derivation: original card faces, Rules Reference p26 ST.2–8, and Innsmouth insert p1.

| Controlled reveal and player choice | Independently derived expected consequence |
| --- | --- |
| Reveal curse, −1, 0; cancel curse and −1 | Resolve 0; skill 9; succeed against 3; no horror and no additional curse draw. Cancellation prevents the curse effects, including its removal to the pool. The curse remains available for subsequent tests. |
| Same reveal; cancel −1 and 0; additional draw −2 | Resolve curse, then its additional −2; total modifier −4; skill 5; succeed with no horror. Return the resolved curse to the token pool at test end. |
| Reveal auto-fail, curse, 0; cancel auto-fail; additional draw −2 | Both remaining tokens resolve; curse generates the additional draw. Skill 5, no horror. Canceling auto-fail removes only that token, not a second token too. |
| Same initial reveal; cancel curse and 0 | Auto-fail remains: modified skill is treated as 0, failure margin 3, and Jacqueline takes 3 horror. A strategically poor legal choice must remain available. |

The table is a worked derivation, not a quoted publisher example or output from another engine. Cancellation follows the card and Rules Reference p6; aggregate resolved modifiers and automatic failure use FAQ §2.5/§2.9 and ST.5–7. Token cleanup follows ST.8 and the Innsmouth exception. Exact handling of canceled-token custody during more complicated nested reveals remains outside this worked case; do not infer a universal immediate-return rule from it.

Negative expectations: refuse a second ordinary Jacqueline use during the curse's extra draw; refuse combining cancellation of auto-fail with another cancellation; refuse a remote Patrice commitment or two committed cards from Patrice. If all ten curse tokens are already in the bag or sealed, Promise instead makes **Patrice**, its committer, take two horror; the printed skill icons still contribute if the state continues normally. Do not turn declining the optional reaction into an automatic failure.

A future experiment should run these choices through the actual Rust rules path and tabletop, pause/save at the cancellation choice, resume without new draws, and replay the resulting moments. Test fixtures may prescribe randomness; ordinary play and agent actions may not. Headless checks establish these consequences only. Human inspection must judge whether choices, token grouping, explanations, and narration controls are understandable.

## Deficits retained

- This is not complete deck/campaign source adjudication. The original prints, errata, and selected versions still require reconciliation for every supported behavior before the full delivery claim.
- Olive, Dark Prophecy, Arbiter, Premonition, and Eyes of the Dreamer/Katarina produce additional real reveal/ignore/limit interactions. FAQ p17 provides a nested replacement example and p22 addresses Eyes/Katarina, but those are not proof of every combined sequence. Grimoire p14 also differentiates ignored from canceled tokens; applying it to older cards requires explicit reconciliation. Keep that larger interaction family unmet.
- Unknown basic weaknesses, successors, and a replacement investigator after death can enlarge the selected content. Gathering R3 and the general campaign rules can require replacements; full campaign support cannot silently reinterpret every death as campaign termination. Devourer setup can select any of four Agents encounter sets, all within reachable scope.
- No source-derived fixture has executed in Rust; no gameplay, rendering, save/resume, or replay fidelity is established here. Accepting this assessment selects only evidence and any explicitly chosen next experiment.

## Download bindings and custody

The [evidence manifest](issue-1-evidence.json) records source URLs and SHA-256 bindings. Verified PDF copies reside in ignored `local/issue-1-evidence/sources/`; they are not committed or independently backed up. A URL and digest alone do not guarantee durable availability. Before a source release is relied upon, protect these bytes in declared external custody and verify retrieval. If unavailable, mark the baseline unverifiable and reacquire matching bytes or review a changed source. The owner's resource archive was not modified.
