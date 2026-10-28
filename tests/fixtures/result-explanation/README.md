# Pre-refactor result records

Compatibility inputs for [issue #14](https://github.com/joeloverbeck/arkham-unfolded/issues/14), captured on 2026-10-28 from the clean baseline `55baa22447ab2e416ad26a2b5a41780b5efc7977` before editing production code. These are scripted experiment saves, not owner gameplay. Keep these original bytes; do not regenerate them with the formatter being tested.

Built with `cargo build --locked --offline --bin arkham-driver`. Each capture used a fresh temporary `--save` path and JSON-lines `choose` commands with successive revisions from zero, followed by `quit`:

| File | Case | Choice IDs | Expected result-dialog continuation |
| --- | --- | --- | --- |
| `early.session.json` | `curse` | `commit-promise`, `use-jacqueline`, `cancel-1-2` | `Revealed -2.` |
| `late.session.json` | `curse` | `commit-promise`, `reveal`, `use-jacqueline`, `cancel-1-2` | `Revealed -1, 0, -2.` |
| `autofail.session.json` | `autofail` | `commit-promise`, `use-jacqueline`, `cancel-1-2` | None; Curse canceled, auto-fail resolved |

The two curse paths both finish at skill 5 versus difficulty 3 with no additional horror (Jacqueline stays at 2 of 9), but record different reveal groups. The adverse auto-fail finishes at skill 0, deals 3 additional horror, and moves Jacqueline from 2 to 5 of 9. Stored history retains Unicode minus and citations; the renderer normalizes the minus to ASCII and omits citations from the continuation detail. The expectations come from the issue's preservation contract and baseline observations, not the new formatter.

SHA-256 of the unmodified captures:

```text
0356cb31f82e8ef77602c37770f5ff402e5521d163ab6bb52a58331678035126  early.session.json
e1cfb52c20c82b4ec45a7305009961688994fc3230d4bac20d37f6f32b14650b  late.session.json
b10d15c6449cd139f5300eeb452baa9a759e64f93f9450aabce03a4ac8bf0e32  autofail.session.json
```
