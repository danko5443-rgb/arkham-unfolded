# Arkham Unfolded

An English-language Arkham Horror: The Card Game application for Ubuntu, planned with a Rust rules core and an animated 3D tabletop.

**Status: one bounded, scripted Rust/3D experiment. The campaign is not playable yet.** The source assessment led to a selected Jacqueline/Patrice skill-test experiment; its implementation remains separate from acceptance of the full game.

## Launch and play the current experiment

On this Ubuntu machine, open **Terminal** (`Ctrl` + `Alt` + `T`) and paste:

```sh
cd /home/joeloverbeck/src/arkham-unfolded
cargo run --locked --features tabletop --bin arkham-tabletop
```

If your checkout is elsewhere, replace the directory in the first line. This requires Rust/Cargo 1.97.1 or later and an Ubuntu graphics session with X11/XWayland and Vulkan. The first launch downloads and compiles dependencies, which can take several minutes. Later launches reuse the build. Keep the terminal open while playing; the native window is titled **Arkham Unfolded — Scripted Rules Experiment**.

In a fresh case:

1. Click **Commit Promise of Power**, then **Use Jacqueline: reveal 3, then choose cancellations**.
2. Choose which tokens to cancel. The centered result window explains the outcome; click **Return to table** to continue.
3. Click **Open recorded replay**. Use **Next moment** and **Previous moment** to step through it. Those buttons appear only when another moment exists in that direction.

Click any card to inspect its text. Right-drag or use the arrow keys to orbit the camera; the wheel or `+`/`-` zooms, and `Home` resets it. Moving the camera manually overrides replay's automatic following.

**Close and resume:** decisions autosave to `local/experiment/curse.session.json`. Close the window and run the same command to resume. Before any decision, use **Save live session** if you want to retain that initial state. If a completed result opens on launch, click **Return to table**, then **Start another example** for a fresh run; the previous run is archived. From an unfinished case, **End this experiment** makes that restart control available.

The supplied card and token artwork loads from the external archive when available. Missing artwork uses readable substitutes. See the [experiment guide](docs/experiments/jacqueline-curse.md) for the alternate case, selecting a different save or artwork folder, and recovery instructions.

## Project aim

The aim is to let one player control their investigators while the program handles setup, legal actions, timing, card effects, tokens, movement, and campaign bookkeeping. Cards, decks, and tokens should retain the physical presence of a tabletop game. Rules explanations will be available through an inspectable action history.

## First complete delivery

Play the original **Night of the Zealot** campaign from beginning to end on **Standard**, controlling these three investigators together:

| Investigator | Starting ArkhamDB deck |
| --- | --- |
| Tony Morgan | [5392133](https://arkhamdb.com/deck/view/5392133) |
| Patrice Hathaway | [5391062](https://arkhamdb.com/deck/view/5391062) |
| Jacqueline Fine | [5389418](https://arkhamdb.com/deck/view/5389418) |

The delivery includes normal campaign consequences and progression, upgrades made in ArkhamDB, save/resume, and recorded sessions that can be replayed in meaningful steps with automatic camera direction for narration. Smaller experiments will investigate feasibility; they will not count as delivering this campaign.

Content support follows the owner's actual use. Starting decks, campaign content, and selected upgrades and weaknesses must work before a play session begins. This is not a promise to support the entire published card pool. Public development and possible YouTube session recordings do not change that scope.

## Project documents

- [Mission and scope](docs/foundation/mission.md)
- [Play contract](docs/foundation/play-contract.md)
- [Sources and verification](docs/foundation/sources-and-verification.md)
- [Issue #1 feasibility assessment](docs/research/issue-1-assessment.md) — accepted source baseline and evidence for the selected bounded experiment
- [Run the selected skill-test experiment](docs/experiments/jacqueline-curse.md) — native tabletop, player-equivalent driver, save/reopen and recorded replay under [issue #2](https://github.com/joeloverbeck/arkham-unfolded/issues/2)
- [Repository principles](docs/principles/principles-for-repos.md)

[GitHub Issues](https://github.com/joeloverbeck/arkham-unfolded/issues) and pull requests are the work-control plane. Foundations record the intended product and accepted decisions; they are not a second work queue. `CLAUDE.md` contains the agent bootstrap; both `AGENTS.md` and `AGENT.md` link to it.

## Current use and external inputs

The built experiment runs offline. A clean clone needs no artwork or credentials for the checks or readable tabletop substitutes. Run the rules and continuity checks with `cargo test --locked`, or include the tabletop checks with `cargo test --locked --features tabletop`. See the experiment guide for external inputs, records and evidence limits.

The future application will read a separately configured resource archive. Card images and token textures are not included in Git. Gameplay must remain usable with readable English substitutes when artwork is absent; absent or unsupported rules behavior must be identified before play. See the source contract for the archive layout and source identities.

Once required game data is imported, gameplay, saves, and replays will work offline. ArkhamDB remains the deck editor; imports and update checks require network access. Personal saves, replays, captures, and credentials remain local and outside Git.

## Attribution

Arkham Horror: The Card Game is published by Fantasy Flight Games. This is an unofficial independent project. Third-party game content and artwork remain distinct from the project's original code.

The public [Haskell implementation](https://github.com/halogenandtoast/ArkhamHorror) is a secondary implementation reference. Official game sources and independently established expectations determine intended rules behavior.

Original project code is available under the [MIT license](LICENSE). This does not grant rights to the game's artwork, card text, rules publications, or other third-party material. The copied repository constitution remains separately identified governing text.
