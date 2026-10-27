use arkham_unfolded_experiment::continuity::{
    Replay, load_recording, load_session, restart_session, save_recording, save_session,
};
use arkham_unfolded_experiment::{Fixture, Session, Stage, TokenFace};

fn awaiting_cancellation() -> Session {
    let mut game = Session::new(Fixture::curse());
    game.choose(0, "commit-promise").unwrap();
    game.choose(1, "use-jacqueline").unwrap();
    game
}

#[test]
fn reopen_preserves_the_pending_cancellation_and_continues_without_rerolling() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("session.json");
    let original = awaiting_cancellation();
    save_session(&path, &original).unwrap();
    let mut reopened = load_session(&path).unwrap();
    assert_eq!(reopened.observe(), original.observe());
    assert_eq!(reopened.observe().stage, Stage::Cancellation);
    let cancel = reopened
        .observe()
        .choices
        .into_iter()
        .find(|choice| choice.cancel_faces == vec![TokenFace::MinusOne, TokenFace::Zero])
        .unwrap();
    reopened.choose(2, &cancel.id).unwrap();
    let result = reopened.observe();
    assert_eq!(result.stage, Stage::Finished);
    assert_eq!(result.skill_value, 5);
    assert_eq!(result.horror_dealt, 0);
    assert_eq!(result.curses_in_bag, 0);
    assert_eq!(result.curses_in_pool, 10);
    assert_eq!(result.tokens.len(), 4);
}

#[test]
fn a_modified_record_is_refused_without_changing_the_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("session.json");
    save_session(&path, &awaiting_cancellation()).unwrap();
    let original = std::fs::read_to_string(&path).unwrap();
    // Still valid JSON: accidental edits must not become a different live game.
    let modified = original.replacen("Promise of Power", "Some other card", 1);
    assert_ne!(modified, original);
    serde_json::from_str::<serde_json::Value>(&modified).unwrap();
    std::fs::write(&path, &modified).unwrap();
    assert!(load_session(&path).is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), modified);
}

#[test]
fn replacing_a_save_keeps_a_loadable_previous_decision() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("session.json");
    let previous = directory.path().join("session.json.previous");
    let mut game = Session::new(Fixture::curse());
    let before = game.observe();
    save_session(&path, &game).unwrap();
    game.choose(0, "commit-promise").unwrap();
    save_session(&path, &game).unwrap();
    save_session(&path, &game).unwrap();
    assert_eq!(load_session(&path).unwrap().observe(), game.observe());
    assert_eq!(load_session(&previous).unwrap().observe(), before);
}

#[test]
fn replay_moves_through_recorded_moments_without_altering_the_live_game() {
    let mut game = Session::new(Fixture::auto_fail());
    game.choose(0, "commit-promise").unwrap();
    game.choose(1, "use-jacqueline").unwrap();
    let cancel = game
        .observe()
        .choices
        .into_iter()
        .find(|choice| choice.cancel_faces == vec![TokenFace::Curse, TokenFace::Zero])
        .unwrap();
    game.choose(2, &cancel.id).unwrap();
    let record = game.recording();
    let live = game.observe();
    let mut replay = Replay::new(record.clone()).unwrap();
    assert_eq!(replay.current().stage, Stage::Commit);
    assert!(!replay.back());
    for expected in record.moments.iter().skip(1) {
        assert!(replay.next());
        assert_eq!(replay.current(), expected);
    }
    assert_eq!(replay.current().horror_dealt, 3);
    assert_eq!(replay.current().investigators[2].horror, 5);
    assert!(!replay.next());
    assert!(replay.back());
    assert_eq!(replay.current().stage, Stage::Cancellation);
    assert_eq!(game.observe(), live);
    assert_eq!(game.recording(), record);
}

#[test]
fn a_separate_recording_reopens_as_stored_presentation_moments() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("replay.json");
    let mut game = Session::new(Fixture::auto_fail());
    game.choose(0, "decline-promise").unwrap();
    game.choose(1, "reveal").unwrap();
    let recording = game.recording();
    save_recording(&path, &recording).unwrap();
    let mut replay = Replay::new(load_recording(&path).unwrap()).unwrap();
    while replay.next() {}
    assert_eq!(replay.current().stage, Stage::Finished);
    assert_eq!(replay.current().horror_dealt, 3);
    assert_eq!(replay.current(), recording.moments.last().unwrap());
    assert!(load_session(&path).is_err());
}

#[test]
fn a_stale_session_cannot_replace_a_later_committed_choice() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("session.json");
    let mut committed = Session::new(Fixture::curse());
    let mut stale = committed.clone();
    save_session(&path, &committed).unwrap();
    committed.choose(0, "commit-promise").unwrap();
    save_session(&path, &committed).unwrap();
    let original_bytes = std::fs::read(&path).unwrap();
    stale.choose(0, "decline-promise").unwrap();
    assert!(save_session(&path, &stale).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original_bytes);
    assert_eq!(load_session(&path).unwrap().observe(), committed.observe());
}

#[test]
fn explicit_restart_keeps_the_active_path_and_preserves_the_entire_prior_run() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("session.json");
    let previous = awaiting_cancellation();
    save_session(&path, &previous).unwrap();
    let mut fresh = Session::new(Fixture::auto_fail());
    restart_session(&path, &fresh).unwrap();
    assert_eq!(load_session(&path).unwrap(), fresh);
    // Later autosaves may rotate .previous, but the prior run still survives.
    fresh.choose(0, "commit-promise").unwrap();
    save_session(&path, &fresh).unwrap();
    let archives: Vec<_> = std::fs::read_dir(directory.path())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .contains(".archived-")
        })
        .collect();
    assert_eq!(archives.len(), 1);
    assert_eq!(load_session(&archives[0]).unwrap(), previous);
    assert_eq!(load_session(&path).unwrap(), fresh);
}

fn reseal(envelope: &mut serde_json::Value) {
    use sha2::{Digest, Sha256};
    let checksum = Sha256::digest(
        serde_json::to_vec(&serde_json::json!([
            envelope["format_version"],
            envelope["kind"],
            envelope["source_revision"],
            envelope["app_version"],
            envelope["payload"],
        ]))
        .unwrap(),
    );
    envelope["checksum"] = serde_json::json!(format!("{checksum:x}"));
}

#[test]
fn structurally_invalid_and_unrecorded_session_states_are_refused_on_load() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("session.json");
    save_session(&path, &awaiting_cancellation()).unwrap();
    let original: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let mut invalid_index = original.clone();
    invalid_index["payload"]["group"] = serde_json::json!([255]);
    let mut invalid_pool = original.clone();
    invalid_pool["payload"]["curses_in_pool"] = serde_json::json!(0);
    invalid_pool["payload"]["moments"]
        .as_array_mut()
        .unwrap()
        .last_mut()
        .unwrap()["curses_in_pool"] = serde_json::json!(0);
    let mut unrecorded = original;
    unrecorded["payload"]["history"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!("An unrecorded change"));
    for mut invalid in [invalid_index, invalid_pool, unrecorded] {
        reseal(&mut invalid);
        let bytes = serde_json::to_vec(&invalid).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        assert!(load_session(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
}

#[test]
fn an_exported_recording_is_immutable_and_identical_exports_are_idempotent() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("replay.json");
    let mut game = awaiting_cancellation();
    let recording = game.recording();
    save_recording(&path, &recording).unwrap();
    let original = std::fs::read(&path).unwrap();
    save_recording(&path, &recording).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert!(!directory.path().join("replay.json.previous").exists());
    game.choose(2, "end").unwrap();
    assert!(save_recording(&path, &game.recording()).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
}

#[test]
fn incompatible_bindings_are_refused_even_with_valid_checksums() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("session.json");
    let game = awaiting_cancellation();
    save_session(&path, &game).unwrap();
    let original: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    for (key, value) in [
        ("format_version", serde_json::json!(999)),
        ("source_revision", serde_json::json!("another-source")),
        ("app_version", serde_json::json!("999.0.0")),
    ] {
        let mut incompatible = original.clone();
        incompatible[key] = value;
        reseal(&mut incompatible);
        let bytes = serde_json::to_vec(&incompatible).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        assert!(load_session(&path).is_err());
        assert!(save_session(&path, &game).is_err());
        assert!(restart_session(&path, &Session::new(Fixture::curse())).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
}

#[test]
fn a_damaged_recovery_file_is_preserved_and_blocks_replacement() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("session.json");
    let previous = directory.path().join("session.json.previous");
    let mut game = Session::new(Fixture::curse());
    save_session(&path, &game).unwrap();
    game.choose(0, "commit-promise").unwrap();
    save_session(&path, &game).unwrap();
    let current_bytes = std::fs::read(&path).unwrap();
    std::fs::write(&previous, b"damaged recovery bytes").unwrap();
    game.choose(1, "use-jacqueline").unwrap();
    assert!(save_session(&path, &game).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), current_bytes);
    assert_eq!(std::fs::read(&previous).unwrap(), b"damaged recovery bytes");
}
