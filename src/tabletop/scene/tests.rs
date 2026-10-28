use super::super::*;
use super::*;
use std::time::Duration;

fn pending_app(path: PathBuf) -> App {
    let mut session = Session::new(Fixture::curse());
    session.choose(0, "commit-promise").unwrap();
    session.choose(1, "use-jacqueline").unwrap();
    let config = AssetConfig::load_at(path.with_extension("toml"), path.with_extension("assets"));
    let mut app = App::new();
    app.insert_resource(Tabletop {
        fixture: session.fixture.clone(),
        session,
        replay: None,
        save_path: path,
        notice: String::new(),
        inspect: None,
        history: false,
        result_open: false,
        dirty: true,
    })
    .insert_resource(Orbit::default())
    .insert_resource(Actions::default())
    .insert_resource(AssetSettings::new(config))
    .init_resource::<Assets<Mesh>>()
    .init_resource::<Assets<StandardMaterial>>()
    .init_resource::<Time>()
    .insert_resource(Art {
        handles: [None, None, None],
        available: 0,
        token_handles: vec![],
        reminder: None,
        investigators: Vec::new(),
        symbol_font: None,
        status: "Readable substitutes are active.".into(),
    })
    .add_systems(
        Update,
        (
            mark_window_resized,
            poll_asset_settings,
            handle_actions,
            rebuild_panel,
            button_colors,
        )
            .chain(),
    );
    app.world_mut().spawn(Window::default());
    install(&mut app);
    app
}

fn act(app: &mut App, action: Action) {
    app.world_mut().resource_mut::<Actions>().0.push(action);
    app.update();
}

#[test]
fn card_inspection_blocks_underlying_decisions_until_closed() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = pending_app(dir.path().join("live.json"));
    act(&mut app, Action::Inspect(2));
    act(&mut app, Action::Choose(2, "cancel-1-2".into()));
    assert_eq!(
        app.world().resource::<Tabletop>().view().stage,
        Stage::Cancellation
    );
    act(&mut app, Action::CloseInspect);
    act(&mut app, Action::Choose(2, "cancel-1-2".into()));
    assert_eq!(
        app.world().resource::<Tabletop>().view().stage,
        Stage::Finished
    );
}

#[test]
fn showing_and_dismissing_result_do_not_replay_token_reveal_animation() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = pending_app(dir.path().join("live.json"));
    act(&mut app, Action::Choose(2, "cancel-1-2".into()));
    assert_eq!(
        appearing_count(&mut app),
        4,
        "newly revealed tokens animate"
    );
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_millis(20));
    app.update();
    let moving_tokens: Vec<_> = {
        let world = app.world_mut();
        world
            .query_filtered::<(Entity, &Transform), With<Appearing>>()
            .iter(world)
            .map(|(entity, transform)| (entity, transform.scale))
            .collect()
    };
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::ZERO);
    act(&mut app, Action::DismissResult);
    act(&mut app, Action::ShowResult);
    for (entity, scale) in moving_tokens {
        assert_eq!(
            app.world().get::<Transform>(entity).unwrap().scale,
            scale,
            "a modal redraw must preserve an in-flight token and its progress"
        );
    }
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs(1));
    app.update();
    assert_eq!(
        appearing_count(&mut app),
        0,
        "tokens have reached full size"
    );
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::ZERO);
    for action in [
        Action::DismissResult,
        Action::ShowResult,
        Action::DismissResult,
    ] {
        act(&mut app, action);
        assert_eq!(
            appearing_count(&mut app),
            0,
            "modal actions must not replay an existing reveal"
        );
    }
}

#[test]
fn committed_result_can_be_dismissed_saved_and_requested_again() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("live.json");
    let mut app = pending_app(path.clone());
    act(&mut app, Action::Choose(2, "cancel-1-2".into()));
    assert!(app.world().resource::<Tabletop>().result_open);
    assert_eq!(
        continuity::load_session(&path)
            .unwrap()
            .observe()
            .skill_value,
        5
    );
    // A queued underlying-table action cannot replace the completed run through the modal.
    act(&mut app, Action::New);
    assert_eq!(
        app.world().resource::<Tabletop>().view().stage,
        Stage::Finished
    );
    act(&mut app, Action::DismissResult);
    act(&mut app, Action::Save);
    assert!(!app.world().resource::<Tabletop>().result_open);
    act(&mut app, Action::ShowResult);
    assert!(app.world().resource::<Tabletop>().result_open);
    act(&mut app, Action::DismissResult);
    act(&mut app, Action::Reopen);
    assert!(app.world().resource::<Tabletop>().result_open);
}

#[test]
fn failed_autosave_does_not_present_an_uncommitted_result() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("live.json");
    std::fs::write(&path, "damaged save").unwrap();
    let mut app = pending_app(path.clone());
    act(&mut app, Action::Choose(2, "cancel-1-2".into()));
    let table = app.world().resource::<Tabletop>();
    assert!(!table.result_open);
    assert_eq!(table.view().stage, Stage::Cancellation);
    assert!(table.notice.contains("not committed"));
    assert_eq!(std::fs::read_to_string(path).unwrap(), "damaged save");
}

#[test]
fn replay_result_pauses_dismissibly_without_mutating_live_save() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("live.json");
    let mut app = pending_app(path.clone());
    act(&mut app, Action::Choose(2, "cancel-1-2".into()));
    let saved = std::fs::read(&path).unwrap();
    act(&mut app, Action::DismissResult);
    act(&mut app, Action::Replay);
    assert!(!app.world().resource::<Tabletop>().result_open);
    for _ in 0..3 {
        act(&mut app, Action::Next);
    }
    assert!(app.world().resource::<Tabletop>().result_open);
    act(&mut app, Action::DismissResult);
    act(&mut app, Action::Next); // Already at the last moment: no new presentation.
    assert!(!app.world().resource::<Tabletop>().result_open);
    act(&mut app, Action::Back);
    assert!(!app.world().resource::<Tabletop>().result_open);
    act(&mut app, Action::Next);
    assert!(app.world().resource::<Tabletop>().result_open);
    assert_eq!(std::fs::read(path).unwrap(), saved);
}

#[test]
fn replay_controls_and_reminder_follow_recorded_state_without_resetting_the_flip() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = pending_app(dir.path().join("live.json"));
    act(&mut app, Action::Choose(2, "cancel-1-2".into()));
    act(&mut app, Action::DismissResult);
    let reminder = app
        .world_mut()
        .query_filtered::<Entity, With<ReminderPose>>()
        .single(app.world())
        .unwrap();
    let has_text = |app: &mut App, needle: &str| {
        let world = app.world_mut();
        world
            .query::<&Text>()
            .iter(world)
            .any(|text| text.0.contains(needle))
    };
    act(&mut app, Action::Replay);
    assert!(has_text(&mut app, "Rotting Remains"));
    assert!(has_text(&mut app, "Next moment"));
    assert!(!has_text(&mut app, "Previous moment"));
    assert!(has_text(&mut app, "READY"));
    act(&mut app, Action::Next);
    assert!(has_text(&mut app, "Committed"));
    act(&mut app, Action::Next);
    assert!(has_text(&mut app, "USED THIS ROUND"));
    assert_eq!(
        app.world().get::<ReminderPose>(reminder).unwrap().target,
        std::f32::consts::PI
    );
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_millis(20));
    app.update();
    let angle = app.world().get::<ReminderPose>(reminder).unwrap().angle;
    assert!(angle > 0.0 && angle < std::f32::consts::PI);
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::ZERO);
    act(&mut app, Action::History); // An unrelated rebuild must not restart the physical flip.
    assert_eq!(
        app.world().get::<ReminderPose>(reminder).unwrap().angle,
        angle
    );
    act(&mut app, Action::Next);
    assert!(!has_text(&mut app, "Next moment"));
    assert!(has_text(&mut app, "Previous moment"));
}

fn advance(app: &mut App, millis: u64) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_millis(millis));
    app.update();
}

fn scene_snapshot(app: &mut App) -> Vec<(Entity, Option<Transform>, Option<String>)> {
    let world = app.world_mut();
    let mut entities: Vec<_> = world
        .query_filtered::<(Entity, Option<&Transform>, Option<&Text>), With<TransientScene>>()
        .iter(world)
        .map(|(entity, transform, text)| {
            (entity, transform.copied(), text.map(|text| text.0.clone()))
        })
        .collect();
    entities.sort_by_key(|(entity, _, _)| *entity);
    entities
}

fn reminder_state(app: &mut App) -> (Entity, Transform, f32) {
    let world = app.world_mut();
    let (entity, transform, pose) = world
        .query::<(Entity, &Transform, &ReminderPose)>()
        .single(world)
        .unwrap();
    (entity, *transform, pose.angle)
}

#[test]
fn panel_refreshes_preserve_scene_entities_and_in_flight_motion() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = pending_app(dir.path().join("live.json"));
    app.update();
    advance(&mut app, 20);
    let before = scene_snapshot(&mut app);
    let reminder = reminder_state(&mut app);
    assert!(reminder.2 > 0.0 && reminder.2 < std::f32::consts::PI);
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::ZERO);
    for action in [
        Action::Inspect(2),
        Action::CloseInspect,
        Action::History,
        Action::History,
    ] {
        act(&mut app, action);
        assert_eq!(scene_snapshot(&mut app), before);
        assert_eq!(reminder_state(&mut app), reminder);
    }
    app.world_mut()
        .query::<&mut Window>()
        .single_mut(app.world_mut())
        .unwrap()
        .resolution
        .set(1100.0, 700.0);
    app.update();
    assert_eq!(scene_snapshot(&mut app), before);
    assert_eq!(reminder_state(&mut app), reminder);

    // A real failed selection completes through the settings poll, without opening a dialog.
    let config_path = dir.path().join("invalid.toml");
    std::fs::write(&config_path, "invalid = [").unwrap();
    app.insert_resource(AssetSettings::new(AssetConfig::load_at(
        config_path,
        dir.path().join("assets"),
    )));
    act(&mut app, Action::AssetFolder);
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while app.world().resource::<AssetSettings>().is_busy() {
        assert!(
            std::time::Instant::now() < deadline,
            "settings worker did not finish"
        );
        std::thread::yield_now();
        app.update();
    }
    assert!(
        app.world()
            .resource::<Tabletop>()
            .notice
            .contains("Asset folder was not saved")
    );
    assert_eq!(scene_snapshot(&mut app), before);
    assert_eq!(reminder_state(&mut app), reminder);
    advance(&mut app, 20);
    let moving = reminder_state(&mut app);
    assert_eq!(moving.0, reminder.0);
    assert!(moving.2 > reminder.2);

    act(&mut app, Action::Replay);
    act(&mut app, Action::Next);
    act(&mut app, Action::Next);
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::ZERO);
    let before_follow = scene_snapshot(&mut app);
    let reminder = reminder_state(&mut app);
    assert!(app.world().resource::<Orbit>().follow);
    act(&mut app, Action::Follow);
    assert!(!app.world().resource::<Orbit>().follow);
    assert_eq!(scene_snapshot(&mut app), before_follow);
    assert_eq!(reminder_state(&mut app), reminder);
}

#[test]
fn three_displayed_tokens_continue_to_four_with_only_the_new_token_revealing() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = pending_app(dir.path().join("live.json"));
    app.update();
    advance(&mut app, 1000);
    let previous = scene_snapshot(&mut app);
    let reminder = reminder_state(&mut app).0;
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_millis(20));
    act(&mut app, Action::Choose(2, "cancel-1-2".into()));
    for (entity, _, _) in previous {
        assert!(
            app.world().get_entity(entity).is_err(),
            "old scene pieces and labels must be removed"
        );
    }
    assert_eq!(
        reminder_state(&mut app).0,
        reminder,
        "the physical reminder persists"
    );
    let world = app.world_mut();
    let appearing: Vec<_> = world
        .query_filtered::<&Transform, With<Appearing>>()
        .iter(world)
        .copied()
        .collect();
    assert_eq!(appearing.len(), 1);
    assert!(
        appearing[0].scale.x > 0.5 && appearing[0].scale.x < 1.0,
        "new entities animate in the synchronization frame"
    );
    assert!((appearing[0].translation.x - 0.7).abs() < 0.001);
    let mut pieces: Vec<_> = world
        .query_filtered::<(
            &Transform,
            &MeshMaterial3d<StandardMaterial>,
            Option<&Children>,
        ), With<TransientScene>>()
        .iter(world)
        .filter(|(transform, _, _)| transform.translation.z == -0.20)
        .map(|(transform, material, children)| {
            (
                *transform,
                material.0.clone(),
                children.map_or(0, Children::len),
            )
        })
        .collect();
    pieces.sort_by(|a, b| a.0.translation.x.total_cmp(&b.0.translation.x));
    assert_eq!(pieces.len(), 4);
    for (index, (transform, material, children)) in pieces.iter().enumerate() {
        assert!((transform.translation.x - [-4.5, -3.2, -0.6, 0.7][index]).abs() < 0.001);
        if index < 3 {
            assert_eq!(transform.scale, Vec3::ONE);
        }
        let expected_color = if index < 2 {
            Color::srgb(0.37, 0.25, 0.28)
        } else {
            Color::srgb(0.28, 0.62, 0.51)
        };
        assert_eq!(
            world
                .resource::<Assets<StandardMaterial>>()
                .get(material)
                .unwrap()
                .base_color,
            expected_color
        );
        assert_eq!(
            *children,
            if index < 2 { 2 } else { 0 },
            "canceled tokens retain their crosses"
        );
    }
    let labels: Vec<_> = world
        .query_filtered::<&Text, With<WorldLabel>>()
        .iter(world)
        .map(|text| text.0.as_str())
        .collect();
    assert!(labels.contains(&"#1  Curse\nResolved"));
    assert!(
        labels
            .iter()
            .any(|text| text.starts_with("#4") && text.ends_with("Resolved"))
    );
    assert_eq!(
        labels
            .iter()
            .filter(|text| text.ends_with("Canceled"))
            .count(),
        2
    );
}

fn appearing_count(app: &mut App) -> usize {
    let world = app.world_mut();
    world
        .query_filtered::<Entity, With<Appearing>>()
        .iter(world)
        .count()
}

#[test]
fn replay_back_next_and_live_compare_complete_observations_and_only_previous_tokens() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("live.json");
    let mut app = pending_app(path.clone());
    app.update();
    act(&mut app, Action::Choose(2, "cancel-1-2".into()));
    let saved = std::fs::read(&path).unwrap();
    act(&mut app, Action::DismissResult);
    act(&mut app, Action::Replay);
    act(&mut app, Action::Next);
    act(&mut app, Action::Next);
    assert_eq!(appearing_count(&mut app), 3);
    act(&mut app, Action::Next);
    assert_eq!(appearing_count(&mut app), 1);
    act(&mut app, Action::DismissResult);
    act(&mut app, Action::Back);
    assert_eq!(app.world().resource::<Tabletop>().view().tokens.len(), 3);
    assert_eq!(
        appearing_count(&mut app),
        0,
        "tokens held in the previous moment do not reveal again"
    );
    act(&mut app, Action::Back);
    assert!(app.world().resource::<Tabletop>().view().tokens.is_empty());
    act(&mut app, Action::Next);
    assert_eq!(
        appearing_count(&mut app),
        3,
        "returning from a token-free moment reveals again; there is no seen-token cache"
    );
    act(&mut app, Action::Live);
    assert_eq!(
        app.world().resource::<Tabletop>().view().stage,
        Stage::Finished
    );
    assert_eq!(appearing_count(&mut app), 1);
    assert_eq!(std::fs::read(&path).unwrap(), saved);
    act(&mut app, Action::DismissResult);
    act(&mut app, Action::Replay);
    act(&mut app, Action::Next);
    act(&mut app, Action::Next);

    // Privileged setup: a distinct live session at the same revision as the replay.
    let mut other = Session::new(Fixture::auto_fail());
    other.choose(0, "commit-promise").unwrap();
    other.choose(1, "use-jacqueline").unwrap();
    let expected = other.observe();
    assert_eq!(
        expected.revision,
        app.world().resource::<Tabletop>().view().revision
    );
    assert_ne!(expected, app.world().resource::<Tabletop>().view());
    let previous = scene_snapshot(&mut app);
    app.world_mut().resource_mut::<Tabletop>().session = other;
    act(&mut app, Action::Live);
    assert_eq!(app.world().resource::<Tabletop>().view(), expected);
    assert_eq!(
        appearing_count(&mut app),
        2,
        "same IDs with different faces are new; unchanged ID/face is retained"
    );
    for (entity, _, _) in previous {
        assert!(app.world().get_entity(entity).is_err());
    }

    // Scene synchronization also runs when the panel has already consumed its refresh.
    let mut other = Session::new(Fixture::curse());
    other.choose(0, "commit-promise").unwrap();
    other.choose(1, "use-jacqueline").unwrap();
    assert!(!app.world().resource::<Tabletop>().dirty);
    app.world_mut().resource_mut::<Tabletop>().session = other;
    app.update();
    assert_eq!(appearing_count(&mut app), 2);
    let world = app.world_mut();
    assert!(
        world
            .query_filtered::<&Text, With<WorldLabel>>()
            .iter(world)
            .any(|text| text.0 == "#1  Curse\nRevealed")
    );
}

#[test]
fn world_labels_are_projected_on_spawn_and_follow_the_camera_without_scene_replacement() {
    use bevy::camera::{CameraProjection, ComputedCameraValues, RenderTargetInfo};
    let dir = tempfile::tempdir().unwrap();
    let mut app = pending_app(dir.path().join("live.json"));
    let camera = app
        .world_mut()
        .spawn((
            Camera {
                computed: ComputedCameraValues {
                    clip_from_view: OrthographicProjection {
                        near: 0.1,
                        far: 100.0,
                        area: Rect::new(-8.0, -6.0, 8.0, 6.0),
                        ..OrthographicProjection::default_3d()
                    }
                    .get_clip_from_view(),
                    target_info: Some(RenderTargetInfo {
                        physical_size: UVec2::new(1600, 1200),
                        scale_factor: 1.0,
                    }),
                    ..default()
                },
                ..default()
            },
            GlobalTransform::from(
                Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::NEG_Z),
            ),
            MainCamera,
        ))
        .id();
    app.update();
    let world = app.world_mut();
    let labels: Vec<_> = world
        .query_filtered::<(Entity, &Node, &Visibility), With<WorldLabel>>()
        .iter(world)
        .map(|(entity, node, visibility)| {
            assert_eq!(
                *visibility,
                Visibility::Visible,
                "new labels must reach projection in their spawn frame"
            );
            let Val::Px(left) = node.left else {
                panic!("label was not projected")
            };
            (entity, left)
        })
        .collect();
    assert_eq!(labels.len(), 11);
    let reminder = reminder_state(&mut app).0;
    // Pan one world unit right: this orthographic camera maps that to 100 pixels left.
    app.world_mut()
        .entity_mut(camera)
        .insert(GlobalTransform::from(
            Transform::from_xyz(1.0, 10.0, 0.0).looking_at(Vec3::X, Vec3::NEG_Z),
        ));
    app.update();
    for (entity, left) in &labels {
        let Val::Px(new_left) = app.world().get::<Node>(*entity).unwrap().left else {
            panic!("label was not projected")
        };
        assert!((new_left - (left - 100.0)).abs() < 0.01);
    }
    act(&mut app, Action::Choose(2, "cancel-1-2".into()));
    for (entity, _) in labels {
        assert!(app.world().get_entity(entity).is_err());
    }
    assert_eq!(reminder_state(&mut app).0, reminder);
    let world = app.world_mut();
    let labels: Vec<_> = world
        .query_filtered::<(&Node, &Visibility), With<WorldLabel>>()
        .iter(world)
        .collect();
    assert_eq!(labels.len(), 12);
    assert!(
        labels
            .iter()
            .all(|(node, visibility)| matches!(node.left, Val::Px(_))
                && **visibility == Visibility::Visible)
    );
}

#[test]
fn world_labels_stay_below_panels_when_observations_change_with_history_open() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = pending_app(dir.path().join("live.json"));
    // Use Bevy's actual UI stack without a display or GPU renderer.
    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
        bevy::input::InputPlugin,
        bevy::input_focus::InputFocusPlugin,
        WindowPlugin {
            primary_window: None,
            ..default()
        },
        bevy::a11y::AccessibilityPlugin,
        bevy::image::ImagePlugin::default(),
        bevy::mesh::MeshPlugin,
        bevy::camera::CameraPlugin,
        bevy::transform::TransformPlugin,
        bevy::sprite::SpritePlugin,
        bevy::text::TextPlugin,
        bevy::ui::UiPlugin,
        bevy::picking::DefaultPickingPlugins,
    ));
    app.finish();
    app.cleanup();
    app.update();
    let assert_layers = |app: &mut App| {
        let world = app.world_mut();
        let labels: Vec<_> = world
            .query_filtered::<Entity, With<WorldLabel>>()
            .iter(world)
            .collect();
        let panels: Vec<_> = world
            .query_filtered::<Entity, With<Panel>>()
            .iter(world)
            .collect();
        assert!(!labels.is_empty() && !panels.is_empty());
        let stack = &world.resource::<bevy::ui::UiStack>().uinodes;
        let top_label = labels
            .iter()
            .map(|entity| stack.iter().position(|node| node == entity).unwrap())
            .max()
            .unwrap();
        for panel in panels {
            assert!(
                stack.iter().position(|node| *node == panel).unwrap() > top_label,
                "world labels must stay beneath panel content"
            );
        }
    };
    assert_layers(&mut app);
    act(&mut app, Action::History);
    assert_layers(&mut app);
    act(&mut app, Action::Choose(2, "cancel-1-2".into()));
    assert_layers(&mut app);
    act(&mut app, Action::DismissResult);
    assert_layers(&mut app);
}
