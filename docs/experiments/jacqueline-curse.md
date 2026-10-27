# Jacqueline's token choice: running the experiment

[GitHub #2](https://github.com/joeloverbeck/arkham-unfolded/issues/2) owns this experiment and its acceptance. Joel selected it after the [assessment](../research/issue-1-assessment.md) at `ef905f5`. It is one scripted skill-test case, not a playable campaign or an adopted game engine.

The task is to decide whether Patrice commits Promise of Power, decide when Jacqueline intervenes, choose which tokens to cancel, and understand the resulting horror and token destinations. You can stop at any pending decision. Then reopen a saved choice and narrate the same events through replay. The source expectations and four worked outcomes are in the [source assessment](../research/issue-1-source-reconciliation.md).

## Build and run

Start with the [README launch walkthrough](../../README.md#launch-and-play-the-current-experiment) for the first clicks and basic controls. Requires Rust/Cargo 1.97.1 or later and a working Ubuntu X11/XWayland graphics session with Vulkan support. The lockfile pins the experiment's dependencies, including Bevy 0.20.0. First dependency acquisition/build requires network access and can take several minutes; the resulting application, saving and replay run offline.

Open Terminal (`Ctrl` + `Alt` + `T`), then run the following. Replace the checkout directory if yours is elsewhere. Keep the terminal open while the native window is running.

```sh
cd /home/joeloverbeck/src/arkham-unfolded
cargo run --locked --features tabletop --bin arkham-tabletop
```

The window is titled **Arkham Unfolded — Scripted Rules Experiment**. For development verification, run `cargo test --locked --features tabletop` from the same directory; tests are not a prerequisite to playing.

For the other prepared case or a deliberate artwork-absence check:

```sh
cargo run --locked --features tabletop --bin arkham-tabletop -- --case autofail
cargo run --locked --features tabletop --bin arkham-tabletop -- --no-art
```

The normal artwork root is `/home/joeloverbeck/Pictures/Arkham Horror LCG Resources`; `--art-root PATH` selects another read-only root. Missing files use readable substitutes. Original images stay outside Git. A card's presence or absence never changes the legal choices or rules outcome. See the [evidence manifest](../research/issue-1-evidence.json) for the three exact original faces used in the worked case.

The five revealed token faces use the archive's consistent 1024-pixel circular set under `sce-base-save`, with identities and hashes in [token-art.json](token-art.json). English face and status labels remain visible beside the graphics. Missing or changed token images use the simple labeled pieces. Alternate designs were found under `lovecrafter_3077` and `rise_rapture_rise`, including smaller square renditions; the selected circular set is pinned for this experiment. Archive categories describe provenance and do not certify publisher artwork.

Card inspection and results use the archive's Arkham font for wild, curse, willpower and horror symbols; the test summary shows willpower on both the current value and difficulty. [symbol-art.json](symbol-art.json) binds the exact font bundle, glyph mappings and extraction. The application decodes this one verified bundle in memory at launch; no manual extraction or archive writes are required. Missing or changed font data uses English words, and accessible labels retain the plain-English meaning.

Tony, Patrice and Jacqueline use their original miniature-card portraits as upright standees with a small holder. Both sides show the color portrait without mirroring; camera position does not imply a state change. [investigator-art.json](investigator-art.json) records the selected images, pixel-art alternatives and model search. No investigator-specific sculpt or UV mapping was found among the catalogued model resources; Unity geometry was not decoded. Missing portraits individually use the labeled simple stand-in.

Jacqueline's investigator card is displayed in landscape on the table and in inspection by rotating its unchanged source image counterclockwise. Inspection enlarges the original art alongside readable rules; on smaller windows its content scrolls while the Close control stays visible. Close card or Escape returns to the table.

Promise of Power gains a highlighted frame when committed. A small labeled reminder to the right of Jacqueline's card flips between available and used, following her once-per-round reaction in live play and replay. It uses the archive's Universal Action / Ability Token as an optional visual base; it is an application reminder, and does not exhaust Jacqueline's investigator card. Canceled tokens move to a separate group with faded faces and visible crosses, while resolved tokens retain their faces. Replay navigation appears only when another moment exists in that direction.

Use the visible buttons for decisions, Save, Reopen and recorded replay. Right-drag or arrow keys orbit the camera; the wheel or `+`/`-` zooms; Home resets it. Replay's Next and Previous controls pause on recorded moments. Moving the camera manually overrides automatic following. Click a card for its text and available original face; Explain opens the action history. F12 captures the window under `local/experiment/`.

A completed test opens a centered result window with centered outcome and skill values, horror and recorded explanation. Zero additional horror is marked unchanged; a horror increase shows the before and after totals. Opening or closing panels preserves the token objects and any reveal animation already in progress, so settled tokens do not appear again. Return to the table to continue inspecting or replaying; Show test result opens it again. Replay presents the recorded result through the same window. A resolved curse applies −2 and requires another reveal: in the prepared curse example, the additional −2 token makes the result `9 − 2 − 2 = 5`. Canceling the curse prevents that additional reveal, as described in the [Innsmouth rules, page 1](https://cdn.svc.asmodee.net/production-fantasyflightgames/uploads/2026/09/ahc81_rules_insert.pdf).

The tabletop resumes its existing case-specific save on the next launch. Start another example retains the prior run and replaces the active save only after the explicit button press. Reopen reports an incompatible/corrupt file rather than starting over. To inspect a particular session directly, use `--load PATH`.

## Player-equivalent agent interface

The JSON-lines driver uses the same `Session::observe` and `Session::choose` functions as the tabletop:

```sh
cargo run --locked --bin arkham-driver -- --case curse --save local/experiment/agent.session.json
```

It emits an initial `observation`, including its revision and currently legal choice IDs. Send a command on each line, for example `{"op":"observe"}` or `{"op":"choose","revision":0,"id":"commit-promise"}`. Always use the current returned revision and a returned choice ID. Other operations are `save`, `replay`, `next`, `back`, `live` and `quit`. A refused command returns an error and the unchanged observation. Replay refuses decisions and past-state saves. Successful choices autosave before being acknowledged.

The driver refuses to start over an existing save; use `--load PATH` to resume it. `--patrice-away` and `--used-jacqueline` are privileged starting-fixture variants for the independent refusal checks, unavailable as player operations and incompatible with `--load`. No future draw order is returned in player observations. The script is intentionally preserved privately inside saves so reopening does not change future outcomes.

## Scope of the prepared state

All three investigators are at the Study unless the separate fixture setup moves Patrice to the Hallway. Jacqueline has 2 starting horror from Arcane Research, no other relevant modifiers, and an unused reaction. Patrice has only Promise of Power available to commit; the other investigators have no committable cards. Rotting Remains tests willpower 3. The poor legal auto-fail choice deals 3 additional horror, leaving Jacqueline at 5 of 9 sanity.

The bag retains the original 16-token Standard composition. This experiment uses a privileged **priority draw order**, not random sampling: either curse/−1/0/−2 or auto-fail/curse/0/−2. If Promise is declined, the curse was never added and is skipped. Every actual reveal removes an available token; no unsupported symbol effect is improvised. The untouched bag symbols are not an implementation claim. The player observation omits future draws, and player commands cannot replace the fixture.

Declining Jacqueline for one reveal does not spend her round limit. If a resolved curse requires another reveal, she may still intervene then. All three cancellation pairs are available for a group without auto-fail; when auto-fail is present, she can cancel it alone or the other two tokens. A canceled curse's effects do not occur; a resolved curse goes to the pool at test end. These are source-derived interpretations within the accepted baseline, not bespoke publisher rulings.

## Continuity and evidence limits

Saves include the pending choice, held tokens, per-round use and remaining fixture order, bound to the application and source versions. The player protocol exposes observations rather than the full private save. Replays retain the observed moments and do not execute today's rules again. Going back in a replay does not undo or edit the original match.

Runtime saves, recordings and screenshots belong under ignored `local/experiment/` or another explicitly selected local path. Preserve original records and a compatible build; there is no general migration promise. An unsupported version or corrupt record must be reported without overwriting it. A clean clone can run the source-independent checks and display substitutes without the owner's archive. The raw reference PDFs/API responses remain separately retained under ignored `local/issue-1-evidence/`; their custody limitations remain as stated in the assessment.

The preceding valid save is retained as `<save-path>.previous`. An explicit new example also archives the prior run to a unique sibling file. To recover, reopen a valid retained file with `--load`; do not edit or overwrite a corrupt original to make it parse. Recording exports are immutable and named by their content; an attempt to replace one with a different match is refused. This experiment verifies ordinary close/reopen and refusal behavior, not power-loss durability or long-term cross-version migration.

Passing automated checks establishes the exercised decisions and continuity boundaries only. Native rendering evidence establishes only the inspected session. Human acceptance of playing and narrating remains separate; the current review allowance and any explicitly selected extensions are recorded on #2. Full campaign branches, other token-replacement cards, unrestricted random play and recurrent usefulness remain unproven. At the 12-active-agent-hour bound, or if the governing assumptions fail, return the evidence to #2 without activating more work.
