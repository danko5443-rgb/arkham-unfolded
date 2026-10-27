# Arkham Unfolded

An English-language Arkham Horror: The Card Game application for Ubuntu, planned with a Rust rules core and an animated 3D tabletop.

**Status: repository foundations. There is no playable application yet.** The initial work establishes the source and verification contract before implementing the game.

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
- [Issue #1 feasibility assessment](docs/research/issue-1-assessment.md) — candidate sources, evidence and a proposed first experiment; awaiting owner disposition
- [Repository principles](docs/principles/principles-for-repos.md)

[GitHub Issues](https://github.com/joeloverbeck/arkham-unfolded/issues) and pull requests are the work-control plane. Foundations record the intended product and accepted decisions; they are not a second work queue. `CLAUDE.md` contains the agent bootstrap; both `AGENTS.md` and `AGENT.md` link to it.

## Current use and external inputs

There is no build, installation, or launch command at this stage. A clean clone contains the foundations and requires no artwork or credentials to read them. Gameplay implementation is not yet ready to activate.

The future application will read a separately configured resource archive. Card images and token textures are not included in Git. Gameplay must remain usable with readable English substitutes when artwork is absent; absent or unsupported rules behavior must be identified before play. See the source contract for the archive layout and source identities.

Once required game data is imported, gameplay, saves, and replays will work offline. ArkhamDB remains the deck editor; imports and update checks require network access. Personal saves, replays, captures, and credentials remain local and outside Git.

## Attribution

Arkham Horror: The Card Game is published by Fantasy Flight Games. This is an unofficial independent project. Third-party game content and artwork remain distinct from the project's original code.

The public [Haskell implementation](https://github.com/halogenandtoast/ArkhamHorror) is a secondary implementation reference. Official game sources and independently established expectations determine intended rules behavior.

Original project code is available under the [MIT license](LICENSE). This does not grant rights to the game's artwork, card text, rules publications, or other third-party material. The copied repository constitution remains separately identified governing text.
