# Issue #1: deck and archive evidence

Supporting evidence for [GitHub #1](https://github.com/joeloverbeck/arkham-unfolded/issues/1), inspected 2026-10-27. This is an assessment, not accepted campaign preparation or an implementation-support declaration. The [play contract](../foundation/play-contract.md) controls the fresh start and owner choices. The committed [deck projection](issue-1-decks.json) preserves the exact relevant API fields without personal descriptions or card text; its Git revision binds this assessment. Raw API response hashes are included there. Hash-verified research copies are retained in ignored `local/issue-1-evidence/decks/`, indexed by the [evidence manifest](issue-1-evidence.json). This is machine-local custody without a backup guarantee; absence does not erase the committed slot lists, but prevents checking the complete original responses.

## Starting inputs

| Investigator / public source | API version; last update (UTC) | Slot copies / distinct codes | Preparation consequence |
| --- | --- | --- | --- |
| [Tony Morgan, 5392133](https://arkhamdb.com/api/public/deck/5392133.json) | 0.5; 2025-10-07 11:56:28 | 35 / 22 | 30 deck-size cards, four signature copies, Internal Injury `02038`; secondary class explicitly Guardian |
| [Patrice Hathaway, 5391062](https://arkhamdb.com/api/public/deck/5391062.json) | 0.7; 2025-10-07 11:56:14 | 45 / 25 | 42 deck-size cards, two signatures, one unresolved basic-weakness placeholder |
| [Jacqueline Fine, 5389418](https://arkhamdb.com/api/public/deck/5389418.json) | 0.9; 2025-10-07 11:55:52 | 35 / 22 | 30 deck-size cards, two signatures, one placeholder, two Arcane Research permanents |

These counts were recomputed from `slots`, not the website's rendered totals. Across the party there are 64 distinct slot codes, or 67 with the investigator identities. `01000` is an ArkhamDB placeholder, leaving 66 actual card identities. Empty sideboards, null ignore-deck-limit fields, and zero XP adjustments are retained. All `xp` and `xp_spent` fields are null: these do not establish campaign experience. Historical `next_deck` pointers are evidence only, not selected successors. Tony's Guardian events/skills total ten copies, matching his stated off-class limit. This is a coarse legality check, not complete preparation certification. Investigator requirements and permanent status come from the pinned [Dream-Eaters records](https://github.com/Kamalisk/arkhamdb-json-data/blob/d8cc0e7e9081c0171298f8b1c07c90ddc2b08662/pack/tde/tde.json), [Jacqueline records](https://github.com/Kamalisk/arkhamdb-json-data/blob/d8cc0e7e9081c0171298f8b1c07c90ddc2b08662/pack/investigator/jac.json), and [Threads of Fate records](https://github.com/Kamalisk/arkhamdb-json-data/blob/d8cc0e7e9081c0171298f8b1c07c90ddc2b08662/pack/tfa/tof.json); these transcriptions remain secondary to printed cards and official corrections.

The API descriptions contain upgrade suggestions and Jacqueline's previous scenario trauma. None is imported as current state. Her two Arcane Research copies independently require two mental trauma at acquisition; the additional historical house-burning trauma does not carry into this fresh campaign. Unsupported suggestions such as a card outside an investigator's permitted levels cannot acquire legality from deck prose.

All three decks identify taboo **9**, effective 2025-07-11. The [live taboo endpoint](https://arkhamdb.com/api/public/taboos/) returned **10**, effective 2026-02-19, as newest on inspection. Only three starting identities have overlays in either list, with identical effects between these two versions: Quick Thinking `02229` limits its extra action to once per round; Dark Prophecy `04032` selects a non-elder-sign symbol; Voice of Ra `60416` counts symbols. Their printed levels remain unchanged. This comparison covers these starting identities only; it cannot establish that future upgrades are unaffected. New-campaign preparation must reconcile and pin list 10 under the source recommendation, rather than silently preserve the decks' list 9 or follow later updates. [Pinned packs metadata](https://github.com/Kamalisk/arkhamdb-json-data/blob/d8cc0e7e9081c0171298f8b1c07c90ddc2b08662/packs.json) identifies the newest selected pack, The Drowned City investigator expansion, as Chapter 1, released 2025-02-07.

## Preparation, successors, and refusal

Recommended handoff: preserve the current accepted campaign and deck snapshot; import a successor as a candidate; compare investigator, fixed deckbuilding choice, full card quantities, signatures, weaknesses, permanents, taboo, exiles and earned/spent/unspent XP. Compute transition costs from accepted rules and the actual changes. Do not trust an API XP total or a prose upgrade plan. Resolve ordering-sensitive discounts explicitly: both Arcane Research copies concern the first qualifying Spell upgrade, not every Spell purchase. Show the resulting cost and remaining XP before accepting the successor.

For Patrice and Jacqueline, freeze the eligible official basic-weakness pool and its selection rules, then record the actual selection before play. Never treat `01000` as playable, limit selection to already implemented weaknesses, or redraw because an unsupported result is inconvenient. Pool deduplication across reprints, printed eligibility restrictions, and quantity policy still need source-backed preparation evidence. The assessment did not draw weaknesses or select upgrades. Each selected card's behavior and material interactions need implementation checks; metadata/image presence is insufficient.

If the import is unavailable, illegal, unaffordable, ambiguous, or unsupported, retain the last accepted campaign and show the specific reason. Resume preparation from the retained candidate after a corrected import or a separately selected implementation closes the deficit; do not mutate the active deck or reset the recorded weakness selection. Useful independent transition witnesses are: a valid first discounted Spell upgrade versus a second purchase; excessive XP refusal; Tony's eleventh off-class card refusal; a legal but unsupported upgrade; and interrupted import followed by recovery without spending XP twice. These are proposed checks, not tests already executed. The applicable sources are the play contract, investigator requirements above, and the independently reconciled official rules in the [source assessment](issue-1-source-reconciliation.md).

## Behavioral pressure from the actual cards

This map is inferred from the selected text, independently of a Rust architecture. It groups obligations, not implementation units or a complete interaction matrix.

| Family | Real input pressure and independent expectation needed |
| --- | --- |
| Reveal, cancellation, replacement, and continuation | Jacqueline, Olive McBride, Dark Prophecy, Premonition, Eyes of the Dreamer, Katarina Sojka and Dark Future have different reveal/resolve/ignore/cancel semantics. Confidence counts additional reveals even when ignored. Preserve each legal choice and token lifecycle. |
| Cross-investigator skill tests | Patrice's Promise of Power adds a curse immediately after commitment, before token revelation; Jacqueline can affect a same-location investigator's reveal. Commit eligibility, curse continuation and cancellation order require source-derived traces. |
| Restricted and nested actions | Tony's bounty-only extra action, Quick Thinking's out-of-turn action, Internal Injury's two-action cost, and attacks of opportunity must coexist without treating all actions as interchangeable. |
| Persistent and triggered bookkeeping | Bounty Contracts, Tony's Quarry doom and farthest-location spawn, ammo, charges, offerings, exhaustion, and round/test limits require distinct lifetimes and trigger ordering. |
| Hidden information and zone transitions | Patrice replaces upkeep draw while retaining weaknesses; Watcher operates from her hand and reacts to deck exhaustion. Winging It and Resourceful use the discard pile; searches and reshuffles expose information relevant to undo. |
| Consequence and response windows | Ward of Protection, Deny Existence, Idol of Xanatos, Sparrow Mask, Lucky!, Live and Learn, and failure rewards distinguish cancellation, damage/horror assignment, completed failure and a subsequent test. |
| Preparation and campaign persistence | Arcane Research trauma/discounts, selected weaknesses, successor decks, earned XP, and scenario rewards or defeats persist beyond an individual test or scenario. |

Primary expectations must be checked against the original cards and official rules, not copied from these summaries. Text/provenance locators include the pinned [Innsmouth records](https://github.com/Kamalisk/arkhamdb-json-data/blob/d8cc0e7e9081c0171298f8b1c07c90ddc2b08662/pack/tic/tic.json), [Drowned City records](https://github.com/Kamalisk/arkhamdb-json-data/blob/d8cc0e7e9081c0171298f8b1c07c90ddc2b08662/pack/tdc/tdcp.json), [Heart of the Elders records](https://github.com/Kamalisk/arkhamdb-json-data/blob/d8cc0e7e9081c0171298f8b1c07c90ddc2b08662/pack/tfa/hote.json), and the investigator records above.

## External archive: identity is distinct from behavior

Read-only root: `/home/joeloverbeck/Pictures/Arkham Horror LCG Resources`. Its reference manifest pins `Kamalisk/arkhamdb-json-data` commit `d8cc0e7e9081c0171298f8b1c07c90ddc2b08662`, dated 2026-10-22T16:53:16Z. All **160** declared `pack/` JSON files matched their recorded SHA-256; together they contain **6,047** unique card codes. The archive source manifest identifies release **v1.22.11**. These observations establish local identity consistency, not publisher completeness.

The 57,699-row asset catalog contains an English entry labelled `official` for each of the 66 physical starting identities. For the first such match per identity, all **132 face/back files** existed and matched their recorded SHA-256. This is file availability/integrity evidence only: no claim that all variants, artwork details, encounter assets, or correct printed editions were visually certified. Multiple categories and repeated variants occur; an arbitrary first catalog match is unsuitable as the application's selection rule. Select by code, edition, language, category and variant, and use readable substitutes when art is absent. `01000` has no physical image and must be resolved semantically.

Recheck bindings, computed directly from the files:

| Relative archive file | SHA-256 |
| --- | --- |
| `sources/reference-manifest.json` | `6ab154484a2131675535be1d7d393770975a8abd2f693df13808be06a1a627e7` |
| `sources/source-manifest.json` | `75287859cfdb18153433917650969973e2b148d33246c14d1f426ba1fc9e06e1` |
| `catalog/cards.jsonl` | `e89f4caec5d1c9b953cb05610bea5e4348823e12ec255d459b39d37add6a3e41` |

No archive files were changed. No gameplay behavior, full weakness pool, selected successor, or campaign-wide image coverage was verified. Changed decks, taboo, source corrections, or archive manifests invalidate the corresponding observations and require a new bounded comparison.
