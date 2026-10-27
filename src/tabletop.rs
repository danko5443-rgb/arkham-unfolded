//! The replaceable tabletop for the single, explicitly scripted rules experiment.
//! Every displayed decision and replay moment comes from the shared rules interface.

use crate::{
    Fixture, Observation, Session, Stage, TokenFace, TokenStatus,
    continuity::{self, Replay},
};
use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit},
    input_focus::tab_navigation::{TabGroup, TabIndex},
    picking::hover::Hovered,
    prelude::*,
    render::view::screenshot::{Capturing, Screenshot, save_to_disk},
    ui::Pressed,
    ui_widgets::{Activate, Button, ModalDialog, ModalDialogBarrier, RequestClose},
};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

mod inspection;
mod standees;
mod symbols;
use symbols::Symbol;

const PANEL: f32 = 390.0;
const INK: Color = Color::srgb(0.035, 0.055, 0.065);
const PAPER: Color = Color::srgb(0.91, 0.89, 0.81);
const MUTED: Color = Color::srgb(0.62, 0.71, 0.70);
const GOLD: Color = Color::srgb(0.86, 0.69, 0.37);
const TEAL: Color = Color::srgb(0.13, 0.37, 0.36);
const BUTTON: Color = Color::srgb(0.12, 0.19, 0.21);
const ARCHIVE: &str = "/home/joeloverbeck/Pictures/Arkham Horror LCG Resources";

struct Card {
    code: &'static str,
    title: &'static str,
    kind: &'static str,
    text: &'static str,
    color: Color,
    landscape: bool,
}

const CARDS: [Card; 3] = [
    Card {
        code: "01163",
        title: "Rotting Remains",
        kind: "Treachery · The Gathering",
        text: "Revelation — Test willpower (3). For each point you fail by, take 1 horror.",
        color: Color::srgb(0.40, 0.22, 0.18),
        landscape: false,
    },
    Card {
        code: "07032",
        title: "Promise of Power",
        kind: "Skill · Four wild icons · Level 0",
        text: "After you commit Promise of Power to a skill test, add 1 curse token to the chaos bag. If you cannot, take 2 horror instead.",
        color: Color::srgb(0.29, 0.20, 0.42),
        landscape: false,
    },
    Card {
        code: "60401",
        title: "Jacqueline Fine",
        kind: "Investigator · Willpower 5",
        text: "Reaction — When an investigator at your location would reveal any number of chaos tokens: Reveal 2 additional tokens. Of the revealed tokens, choose and cancel 2 non-auto-fail tokens, or 1 auto-fail token. Limit once per round.\n\nElder sign: +1. If this effect is canceled or ignored, draw 1 card.\n\nThis experiment has no elder sign reveal or other replacement assets.",
        color: Color::srgb(0.30, 0.24, 0.48),
        landscape: true,
    },
];

#[derive(Resource)]
struct Tabletop {
    session: Session,
    fixture: Fixture,
    replay: Option<Replay>,
    save_path: PathBuf,
    notice: String,
    inspect: Option<usize>,
    history: bool,
    result_open: bool,
    dirty: bool,
}

impl Tabletop {
    fn view(&self) -> Observation {
        self.replay
            .as_ref()
            .map_or_else(|| self.session.observe(), |r| r.current().clone())
    }
}

#[derive(Clone)]
enum Action {
    Choose(u64, String),
    Save,
    Reopen,
    Replay,
    Back,
    Next,
    Live,
    New,
    Inspect(usize),
    CloseInspect,
    History,
    Follow,
    ShowResult,
    DismissResult,
}

#[derive(Resource, Default)]
struct Actions(Vec<Action>);

#[derive(Resource)]
struct Orbit {
    yaw: f32,
    pitch: f32,
    distance: f32,
    desired_distance: f32,
    target: Vec3,
    desired_target: Vec3,
    follow: bool,
}

impl Default for Orbit {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.88,
            distance: 13.8,
            desired_distance: 13.8,
            target: Vec3::new(1.9, 0.0, 0.0),
            desired_target: Vec3::new(1.9, 0.0, 0.0),
            follow: false,
        }
    }
}

#[derive(Resource)]
struct Art {
    handles: [Option<Handle<Image>>; 3],
    available: usize,
    token_handles: Vec<(TokenFace, Handle<Image>)>,
    reminder: Option<Handle<Image>>,
    investigators: Vec<(String, Handle<Image>)>,
    symbol_font: Option<Handle<Font>>,
}

#[derive(serde::Deserialize)]
struct TokenArtManifest {
    faces: Vec<TokenArtFace>,
}

#[derive(serde::Deserialize)]
struct TokenArtFace {
    semantic_face: TokenFace,
    path: String,
    sha256: String,
}

#[derive(Resource)]
struct Capture {
    path: Option<PathBuf>,
    quit: bool,
    frame: u32,
    requested: bool,
}

#[derive(Component)]
struct MainCamera;
#[derive(Component)]
struct Panel;
#[derive(Component)]
struct TransientScene;
#[derive(Component)]
struct Appearing;
#[derive(Component)]
struct ReminderPose {
    angle: f32,
    target: f32,
}
#[derive(Component)]
struct WorldLabel(Vec3);
#[derive(Component)]
struct ButtonTone(Color);

struct Options {
    fixture: Fixture,
    save: PathBuf,
    load: Option<PathBuf>,
    art_root: Option<PathBuf>,
    screenshot: Option<PathBuf>,
    quit_after_screenshot: bool,
}

impl Options {
    fn parse() -> Result<Option<Self>, String> {
        let mut options = Self {
            fixture: Fixture::curse(),
            save: "local/experiment/curse.session.json".into(),
            load: None,
            art_root: Some(ARCHIVE.into()),
            screenshot: None,
            quit_after_screenshot: false,
        };
        let mut explicit_save = false;
        let mut selected_case = "curse";
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            let mut value = || args.next().ok_or_else(|| format!("{arg} needs a value"));
            match arg.as_str() {
                "--case" => {
                    options.fixture = match value()?.as_str() {
                        "curse" => {
                            selected_case = "curse";
                            Fixture::curse()
                        }
                        "autofail" => {
                            selected_case = "autofail";
                            Fixture::auto_fail()
                        }
                        _ => return Err("--case must be curse or autofail".into()),
                    }
                }
                "--save" => {
                    options.save = value()?.into();
                    explicit_save = true;
                }
                "--load" => options.load = Some(value()?.into()),
                "--art-root" => options.art_root = Some(value()?.into()),
                "--no-art" => options.art_root = None,
                "--screenshot" => options.screenshot = Some(value()?.into()),
                "--quit-after-screenshot" => options.quit_after_screenshot = true,
                "--help" | "-h" => {
                    println!(
                        "Arkham Unfolded — scripted rules experiment\n\n--case curse|autofail  Choose the privileged starting fixture\n--save PATH            Session save and autosave path\n--load PATH            Reopen a saved pending decision\n--art-root PATH        Read the external artwork archive\n--no-art               Use readable substitutes\n--screenshot PATH      Capture the actual window once ready\n--quit-after-screenshot\n\nCamera: right drag or arrow keys; wheel or +/- to zoom; Home to reset. F12 saves a screenshot."
                    );
                    return Ok(None);
                }
                _ => return Err(format!("Unknown option: {arg}")),
            }
        }
        if !explicit_save {
            options.save = format!("local/experiment/{selected_case}.session.json").into();
        }
        if options.quit_after_screenshot && options.screenshot.is_none() {
            return Err("--quit-after-screenshot requires --screenshot PATH".into());
        }
        Ok(Some(options))
    }
}

/// Launch the actual native consumer. Errors preserve existing saves.
pub fn run() -> Result<(), String> {
    let Some(options) = Options::parse()? else {
        return Ok(());
    };
    let load_path = options
        .load
        .or_else(|| options.save.is_file().then(|| options.save.clone()));
    let resumed = load_path.is_some();
    let session = if let Some(path) = &load_path {
        continuity::load_session(path).map_err(|e| format!("Could not reopen the session: {e}"))?
    } else {
        Session::new(options.fixture.clone())
    };
    let save_path = load_path.unwrap_or(options.save);
    let art_root = options.art_root;
    let asset_root = art_root.as_ref().map_or_else(
        || "local/experiment/absent-art".to_string(),
        |p| p.to_string_lossy().into_owned(),
    );
    let mut app = App::new();
    app.insert_resource(ClearColor(INK))
        .insert_resource(Tabletop {
            fixture: session.fixture.clone(),
            result_open: session.observe().stage == Stage::Finished,
            session,
            replay: None,
            save_path,
            notice: if resumed {
                "Saved experiment reopened at its exact pending decision."
            } else {
                "Choose freely. The rules handle the consequences."
            }
            .into(),
            inspect: None,
            history: false,
            dirty: true,
        })
        .insert_resource(Orbit::default())
        .insert_resource(Actions::default())
        .insert_resource(Capture {
            path: options.screenshot,
            quit: options.quit_after_screenshot,
            frame: 0,
            requested: false,
        })
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Arkham Unfolded — Scripted Rules Experiment".into(),
                        resolution: (1600, 1000).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: asset_root,
                    ..default()
                }),
        )
        .add_plugins(MeshPickingPlugin)
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                mark_window_resized,
                handle_actions,
                camera_controls,
                rebuild_view,
                label_positions,
                animate_tokens,
                animate_reminder,
                button_colors,
                capture_window,
            )
                .chain(),
        );
    let handles = load_art(app.world().resource::<AssetServer>(), art_root.as_deref());
    let available = handles.iter().filter(|h| h.is_some()).count();
    let token_handles = load_token_art(app.world().resource::<AssetServer>(), art_root.as_deref());
    let reminder = art_root.as_deref().and_then(|root| {
        let manifest: serde_json::Value =
            serde_json::from_str(include_str!("../docs/experiments/token-art.json"))
                .expect("valid art manifest");
        let face = &manifest["reaction_reminder"];
        verified_image(
            app.world().resource::<AssetServer>(),
            root,
            face["path"].as_str()?,
            face["sha256"].as_str()?,
        )
    });
    let investigators = standees::load(app.world().resource::<AssetServer>(), art_root.as_deref());
    let symbol_font = symbols::load(
        art_root.as_deref(),
        &mut app.world_mut().resource_mut::<Assets<Font>>(),
    );
    app.insert_resource(Art {
        handles,
        available,
        token_handles,
        reminder,
        investigators,
        symbol_font,
    });
    app.run();
    Ok(())
}

fn load_art(server: &AssetServer, root: Option<&Path>) -> [Option<Handle<Image>>; 3] {
    let evidence: serde_json::Value =
        serde_json::from_str(include_str!("../docs/research/issue-1-evidence.json"))
            .expect("the committed evidence manifest is valid JSON");
    std::array::from_fn(|i| {
        let root = root?;
        let face = evidence["inspected_card_faces"]["faces"]
            .as_array()?
            .iter()
            .find(|face| face["code"].as_str() == Some(CARDS[i].code))?;
        let path = face["face_path"].as_str()?;
        verified_image(server, root, path, face["sha256"].as_str()?)
    })
}

fn load_token_art(server: &AssetServer, root: Option<&Path>) -> Vec<(TokenFace, Handle<Image>)> {
    let Some(root) = root else { return Vec::new() };
    let manifest: TokenArtManifest =
        serde_json::from_str(include_str!("../docs/experiments/token-art.json"))
            .expect("the committed token art manifest is valid JSON");
    manifest
        .faces
        .into_iter()
        .filter_map(|face| {
            verified_image(server, root, &face.path, &face.sha256)
                .map(|image| (face.semantic_face, image))
        })
        .collect()
}

fn verified_image(
    server: &AssetServer,
    root: &Path,
    path: &str,
    sha256: &str,
) -> Option<Handle<Image>> {
    let bytes = std::fs::read(root.join(path)).ok()?;
    // A changed or missing image is a presentation substitute, never a rules block.
    if format!("{:x}", Sha256::digest(&bytes)) != sha256 {
        return None;
    }
    Some(server.load(path.to_owned()))
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    art: Res<Art>,
) {
    let velvet = materials.add(StandardMaterial {
        base_color: Color::srgb(0.035, 0.16, 0.15),
        perceptual_roughness: 0.95,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(12.4, 0.3, 8.2))),
        MeshMaterial3d(velvet),
        Transform::from_xyz(0.0, -0.2, 0.0),
        Pickable::IGNORE,
    ));
    let brass = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.31, 0.13),
        metallic: 0.7,
        perceptual_roughness: 0.5,
        ..default()
    });
    for (size, at) in [
        (Vec3::new(12.6, 0.15, 0.12), Vec3::new(0.0, -0.01, -4.1)),
        (Vec3::new(12.6, 0.15, 0.12), Vec3::new(0.0, -0.01, 4.1)),
        (Vec3::new(0.12, 0.15, 8.2), Vec3::new(-6.2, -0.01, 0.0)),
        (Vec3::new(0.12, 0.15, 8.2), Vec3::new(6.2, -0.01, 0.0)),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::from_size(size))),
            MeshMaterial3d(brass.clone()),
            Transform::from_translation(at),
            Pickable::IGNORE,
        ));
    }
    let zone = materials.add(Color::srgb(0.065, 0.22, 0.21));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(10.6, 0.035, 2.45))),
        MeshMaterial3d(zone),
        Transform::from_xyz(0.0, -0.02, -2.4),
        Pickable::IGNORE,
    ));
    let bag = materials.add(Color::srgb(0.18, 0.10, 0.16));
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.65, 0.9))),
        MeshMaterial3d(bag),
        Transform::from_xyz(-4.5, 0.48, 1.75),
        Pickable::IGNORE,
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.46, 0.025))),
        MeshMaterial3d(materials.add(Color::BLACK)),
        Transform::from_xyz(-4.5, 0.95, 1.75),
        Pickable::IGNORE,
    ));
    for (i, card) in CARDS.iter().enumerate() {
        let x = -2.0 + i as f32 * 2.4;
        // The source stores Jacqueline sideways in a portrait-shaped image.
        // Rotating the original face preserves both its artwork and proportions.
        let rotation = Quat::from_rotation_y(if card.landscape {
            std::f32::consts::FRAC_PI_2
        } else {
            0.0
        });
        let material = materials.add(StandardMaterial {
            base_color: if art.handles[i].is_some() {
                Color::WHITE
            } else {
                card.color
            },
            base_color_texture: art.handles[i].clone(),
            perceptual_roughness: 0.9,
            unlit: true,
            ..default()
        });
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.85, 0.07, 2.60))),
            MeshMaterial3d(materials.add(Color::srgb(0.14, 0.12, 0.10))),
            Transform::from_xyz(x, 0.08, 1.9).with_rotation(rotation),
            Pickable::IGNORE,
        ));
        commands
            .spawn((
                Mesh3d(meshes.add(Plane3d::default().mesh().size(1.80, 2.52))),
                MeshMaterial3d(material),
                Transform::from_xyz(x, 0.125, 1.9).with_rotation(rotation),
            ))
            .observe(move |_: On<PointerClick>, mut queue: ResMut<Actions>| {
                queue.0.push(Action::Inspect(i))
            });
    }
    commands.spawn((
        DirectionalLight {
            illuminance: 7_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, -0.6, 0.0)),
    ));
    commands.spawn((
        PointLight {
            intensity: 250_000.0,
            color: Color::srgb(1.0, 0.76, 0.48),
            ..default()
        },
        Transform::from_xyz(-5.0, 6.0, 4.0),
    ));
    commands.spawn((Camera3d::default(), Transform::default(), MainCamera));
    commands
        .spawn((
            Mesh3d(meshes.add(Cylinder::new(0.20, 0.06))),
            MeshMaterial3d(materials.add(Color::srgb(0.35, 0.35, 0.35))),
            Transform::from_xyz(4.55, 0.22, 1.9),
            Pickable::IGNORE,
            ReminderPose {
                angle: 0.0,
                target: 0.0,
            },
        ))
        .with_children(|parent| {
            for (y, rotation, color) in [
                (
                    0.035,
                    -std::f32::consts::FRAC_PI_2,
                    Color::srgb(0.35, 0.95, 0.65),
                ),
                (
                    -0.035,
                    std::f32::consts::FRAC_PI_2,
                    Color::srgb(0.44, 0.35, 0.47),
                ),
            ] {
                parent.spawn((
                    Mesh3d(meshes.add(Circle::new(0.19))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: color,
                        base_color_texture: art.reminder.clone(),
                        alpha_mode: AlphaMode::Mask(0.5),
                        unlit: true,
                        ..default()
                    })),
                    Transform::from_xyz(0.0, y, 0.0).with_rotation(Quat::from_rotation_x(rotation)),
                    Pickable::IGNORE,
                ));
            }
        });
}

fn handle_actions(
    mut queue: ResMut<Actions>,
    mut table: ResMut<Tabletop>,
    mut orbit: ResMut<Orbit>,
) {
    for action in queue.0.drain(..) {
        // A modal consumes interaction with the underlying table, including queued clicks.
        if (table.result_open && !matches!(action, Action::DismissResult))
            || (table.inspect.is_some() && !matches!(action, Action::CloseInspect))
        {
            continue;
        }
        match action {
            Action::Choose(revision, id) if table.replay.is_none() => {
                let mut candidate = table.session.clone();
                let result = candidate
                    .choose(revision, &id)
                    .map_err(|e| e.to_string())
                    .and_then(|()| {
                        continuity::save_session(&table.save_path, &candidate)
                            .map_err(|e| e.to_string())
                    });
                match result {
                    Ok(()) => {
                        table.result_open = candidate.observe().stage == Stage::Finished;
                        table.session = candidate;
                        table.notice = "Decision saved. You can close and reopen safely.".into();
                    }
                    Err(error) => table.notice = format!("Decision was not committed: {error}"),
                }
            }
            Action::Save => {
                table.notice = match continuity::save_session(&table.save_path, &table.session) {
                    Ok(()) => "Live session saved at its current decision.".into(),
                    Err(e) => format!("Save failed; the current session is unchanged: {e}"),
                }
            }
            Action::Reopen => match continuity::load_session(&table.save_path) {
                Ok(session) => {
                    table.fixture = session.fixture.clone();
                    table.result_open = session.observe().stage == Stage::Finished;
                    table.session = session;
                    table.replay = None;
                    orbit.follow = false;
                    table.notice = "Saved session reopened. The pending choice and revealed tokens are preserved.".into();
                }
                Err(e) => {
                    table.notice =
                        format!("Could not reopen; the current session is unchanged: {e}")
                }
            },
            Action::Replay => {
                let recording = table.session.recording();
                let digest = format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(&recording).expect("recordings serialize"))
                );
                let stem = table
                    .save_path
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .unwrap_or("experiment");
                let path = table
                    .save_path
                    .with_file_name(format!("{stem}.replay-{}.json", &digest[..16]));
                match continuity::save_recording(&path, &recording)
                    .and_then(|()| continuity::load_recording(&path))
                    .and_then(Replay::new)
                {
                    Ok(replay) => {
                        table.replay = Some(replay);
                        table.result_open = false;
                        orbit.follow = true;
                        table.notice = "Replay reopened from its saved record. Next advances one moment and pauses.".into();
                    }
                    Err(e) => table.notice = format!("Could not reopen the replay: {e}"),
                }
            }
            Action::Back => {
                if let Some(replay) = &mut table.replay
                    && replay.back()
                {
                    table.result_open = replay.current().stage == Stage::Finished;
                }
            }
            Action::Next => {
                if let Some(replay) = &mut table.replay
                    && replay.next()
                {
                    table.result_open = replay.current().stage == Stage::Finished;
                }
            }
            Action::Live => {
                table.replay = None;
                table.result_open = table.session.observe().stage == Stage::Finished;
                orbit.follow = false;
                table.notice = "Returned to the live session; replay made no changes to it.".into();
            }
            Action::New => {
                let session = Session::new(table.fixture.clone());
                match continuity::restart_session(&table.save_path, &session) {
                    Ok(()) => {
                        table.session = session;
                        table.replay = None;
                        table.result_open = false;
                        orbit.follow = false;
                        table.notice =
                            "New example ready. The previous session has been retained.".into();
                    }
                    Err(e) => {
                        table.notice = format!(
                            "Could not start another example; the current one is unchanged: {e}"
                        )
                    }
                }
            }
            Action::Inspect(i) => {
                table.inspect = Some(i);
                table.history = false;
            }
            Action::CloseInspect => table.inspect = None,
            Action::History => {
                table.history = !table.history;
                table.inspect = None;
            }
            Action::Follow => {
                if table.replay.is_some() {
                    orbit.follow = !orbit.follow;
                }
            }
            Action::ShowResult => table.result_open = table.view().stage == Stage::Finished,
            Action::DismissResult => table.result_open = false,
            Action::Choose(_, _) => {}
        }
        if !orbit.follow {
            orbit.desired_target = orbit.target;
            orbit.desired_distance = orbit.distance;
        }
        if orbit.follow && table.replay.is_some() {
            let (target, distance) = match table.view().stage {
                Stage::Commit => (Vec3::new(1.5, 0.2, -1.5), 13.8),
                Stage::BeforeReveal => (Vec3::new(1.6, 0.0, 0.5), 13.0),
                Stage::Cancellation => (Vec3::new(1.6, 0.0, 0.2), 11.7),
                Stage::Finished | Stage::Ended => (Vec3::new(1.7, 0.2, -1.2), 13.2),
            };
            orbit.desired_target = target;
            orbit.desired_distance = distance;
        }
        table.dirty = true;
    }
}

fn mark_window_resized(
    windows: Query<&Window>,
    mut previous_size: Local<Option<Vec2>>,
    mut table: ResMut<Tabletop>,
) {
    if let Some(window) = windows.iter().next() {
        let size = Vec2::new(window.width(), window.height());
        if previous_size.as_ref() != Some(&size) {
            *previous_size = Some(size);
            table.dirty = true;
        }
    }
}

// Bevy injects each input, resource and query independently into this system.
#[allow(clippy::too_many_arguments)]
fn camera_controls(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    window: Single<&Window>,
    mut table: ResMut<Tabletop>,
    time: Res<Time>,
    mut orbit: ResMut<Orbit>,
    mut camera: Single<&mut Transform, With<MainCamera>>,
) {
    let modal_open = table.result_open || table.inspect.is_some();
    let scene_pointer = !modal_open
        && !table.history
        && window
            .cursor_position()
            .is_some_and(|p| p.x < window.width() - PANEL);
    let was_following = orbit.follow;
    let mut changed = false;
    if buttons.pressed(MouseButton::Right) && scene_pointer && motion.delta != Vec2::ZERO {
        orbit.yaw -= motion.delta.x * 0.006;
        orbit.pitch += motion.delta.y * 0.004;
        changed = true;
    }
    if scene_pointer && scroll.delta.y != 0.0 {
        orbit.distance -= scroll.delta.y * 0.6;
        changed = true;
    }
    for (key, yaw, pitch, zoom) in [
        (KeyCode::ArrowLeft, 1.0, 0.0, 0.0),
        (KeyCode::ArrowRight, -1.0, 0.0, 0.0),
        (KeyCode::ArrowUp, 0.0, 0.7, 0.0),
        (KeyCode::ArrowDown, 0.0, -0.7, 0.0),
        (KeyCode::Equal, 0.0, 0.0, -5.0),
        (KeyCode::Minus, 0.0, 0.0, 5.0),
    ] {
        if !modal_open && keys.pressed(key) {
            orbit.yaw += yaw * time.delta_secs();
            orbit.pitch += pitch * time.delta_secs();
            orbit.distance += zoom * time.delta_secs();
            changed = true;
        }
    }
    if !modal_open && keys.just_pressed(KeyCode::Home) {
        *orbit = Orbit::default();
        changed = true;
    }
    if changed {
        if was_following {
            table.dirty = true;
        }
        orbit.follow = false;
        orbit.desired_target = orbit.target;
        orbit.desired_distance = orbit.distance;
    }
    orbit.pitch = orbit.pitch.clamp(0.35, 1.35);
    orbit.distance = orbit.distance.clamp(8.0, 22.0);
    let blend = (time.delta_secs() * 4.0).min(1.0);
    orbit.distance += (orbit.desired_distance - orbit.distance) * blend;
    let desired = orbit.desired_target;
    orbit.target = orbit
        .target
        .lerp(desired, (time.delta_secs() * 4.0).min(1.0));
    let direction = Vec3::new(
        orbit.yaw.sin() * orbit.pitch.cos(),
        orbit.pitch.sin(),
        orbit.yaw.cos() * orbit.pitch.cos(),
    );
    **camera = Transform::from_translation(orbit.target + direction * orbit.distance)
        .looking_at(orbit.target, Vec3::Y);
}

fn text(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    // The embedded font is intentionally sufficient without the external archive.
    // Spell punctuation with supported glyphs so token modifiers never become boxes.
    (
        Text::new(readable_text(value.into())),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    )
}

fn readable_text(value: String) -> String {
    value
        .replace(['−', '–'], "-")
        .replace('§', "section ")
        .replace('·', " / ")
        .replace('—', " - ")
        .replace('→', ">")
        .replace('←', "<")
}

enum Run<'a> {
    Words(&'a str),
    Icon(Symbol),
}

// These few runs are explicitly authored for this case, not parsed from rules text.
// Plain-English labels remain available even when the displayed text uses glyphs.
fn symbol_text(
    parent: &mut ChildSpawnerCommands,
    font: &Option<Handle<Font>>,
    runs: &[Run<'_>],
    label: &str,
    size: f32,
    color: Color,
) {
    let Some(font) = font else {
        parent.spawn((
            text(label, size, color),
            AccessibleLabel::new(label),
            Pickable::IGNORE,
        ));
        return;
    };
    parent
        .spawn((
            text("", size, color),
            AccessibleLabel::new(label),
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            for run in runs {
                let (value, text_font) = match run {
                    Run::Icon(symbol) => (
                        // Keep an adjacent quantity with its symbol across font runs.
                        // NBSP can still break on overflow in the pinned renderer.
                        format!("\u{2060}{}\u{2060}", symbol.glyph()),
                        TextFont {
                            font: font.clone().into(),
                            font_size: FontSize::Px(size.max(18.0) * 1.35),
                            ..default()
                        },
                    ),
                    Run::Words(words) => (
                        readable_text((*words).to_owned()),
                        TextFont {
                            font_size: FontSize::Px(size),
                            ..default()
                        },
                    ),
                };
                parent.spawn((TextSpan::new(value), text_font, TextColor(color)));
            }
        });
}

fn skill_summary(
    parent: &mut ChildSpawnerCommands,
    art: &Art,
    view: &Observation,
    size: f32,
    color: Color,
) {
    symbol_text(
        parent,
        &art.symbol_font,
        &[
            Run::Icon(Symbol::Willpower),
            Run::Words(&format!(" {} / TEST ", view.skill_value)),
            Run::Icon(Symbol::Willpower),
            Run::Words(&format!(" ({})", view.difficulty)),
        ],
        &format!(
            "Willpower {} / Test willpower ({})",
            view.skill_value, view.difficulty
        ),
        size,
        color,
    );
}

fn button(parent: &mut ChildSpawnerCommands, label: &str, action: Action, primary: bool) {
    let tone = if primary { TEAL } else { BUTTON };
    parent
        .spawn((
            Button,
            Hovered::default(),
            Node {
                width: percent(100),
                min_height: px(42),
                padding: UiRect::axes(px(12), px(10)),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(px(5)),
                ..default()
            },
            BackgroundColor(tone),
            ButtonTone(tone),
        ))
        .observe(move |_: On<Activate>, mut queue: ResMut<Actions>| queue.0.push(action.clone()))
        .with_child(text(label, 16.0, PAPER));
}

fn label(commands: &mut Commands, value: impl Into<String>, point: Vec3, color: Color) {
    commands.spawn((
        text(value, 16.0, color),
        Node {
            position_type: PositionType::Absolute,
            padding: UiRect::axes(px(8), px(4)),
            border_radius: BorderRadius::all(px(4)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.025, 0.045, 0.055, 0.90)),
        Pickable::IGNORE,
        WorldLabel(point),
        TransientScene,
    ));
}

// Keep the rendering resources explicit at the Bevy system boundary.
#[allow(clippy::too_many_arguments)]
fn rebuild_view(
    mut commands: Commands,
    mut table: ResMut<Tabletop>,
    orbit: Res<Orbit>,
    art: Res<Art>,
    panels: Query<Entity, With<Panel>>,
    transient: Query<Entity, With<TransientScene>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut reminder: Single<&mut ReminderPose>,
    mut rendered_view: Local<Option<Observation>>,
    windows: Query<&Window>,
) {
    if !table.dirty {
        return;
    }
    table.dirty = false;
    for entity in &panels {
        commands.entity(entity).despawn();
    }
    let view = table.view();
    // UI-only redraws retain the scene entities and any in-flight reveal motion.
    if rendered_view.as_ref() != Some(&view) {
        for entity in &transient {
            commands.entity(entity).despawn();
        }
        reminder.target = if view.jacqueline_used {
            std::f32::consts::PI
        } else {
            0.0
        };
        label(
            &mut commands,
            format!(
                "Reaction reminder\n{}",
                if view.jacqueline_used {
                    "USED THIS ROUND"
                } else {
                    "READY"
                }
            ),
            Vec3::new(4.55, 0.15, 2.65),
            if view.jacqueline_used { MUTED } else { GOLD },
        );
        let colors = [
            Color::srgb(0.66, 0.43, 0.20),
            Color::srgb(0.62, 0.23, 0.26),
            Color::srgb(0.37, 0.28, 0.64),
        ];
        for (i, investigator) in view.investigators.iter().enumerate() {
            let here = investigator.location == "Study";
            let (x, z) = if here {
                (-3.4 + i as f32 * 3.3, -2.3)
            } else {
                (-4.6, -0.35)
            };
            if !here {
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(2.2, 0.035, 1.6))),
                    MeshMaterial3d(materials.add(Color::srgb(0.12, 0.20, 0.22))),
                    Transform::from_xyz(x, -0.02, z),
                    Pickable::IGNORE,
                    TransientScene,
                ));
            }
            let image = art
                .investigators
                .iter()
                .find(|(name, _)| name == &investigator.name)
                .map(|(_, image)| image);
            standees::spawn(
                &mut commands,
                &mut meshes,
                &mut materials,
                image,
                Vec3::new(x, 0.0, z),
                colors[i % colors.len()],
            );
            label(
                &mut commands,
                format!(
                    "{}\n{} · Horror {}/{}",
                    investigator.name,
                    investigator.location,
                    investigator.horror,
                    investigator.sanity
                ),
                Vec3::new(x, 2.05, z),
                PAPER,
            );
        }
        label(
            &mut commands,
            format!(
                "Chaos bag · {}\n{} curse in bag",
                view.bag_count, view.curses_in_bag
            ),
            Vec3::new(-4.5, 1.2, 1.75),
            GOLD,
        );
        for (i, card) in CARDS.iter().enumerate() {
            let status = if i == 1 && view.promise_committed {
                "Committed · +4"
            } else {
                "Click to inspect"
            };
            label(
                &mut commands,
                format!("{}\n{status}", card.title),
                Vec3::new(
                    -2.0 + i as f32 * 2.4,
                    0.2,
                    if card.landscape { 2.96 } else { 3.36 },
                ),
                PAPER,
            );
        }
        if view.promise_committed {
            let frame = materials.add(StandardMaterial {
                base_color: GOLD,
                unlit: true,
                ..default()
            });
            for (size, at) in [
                (Vec3::new(2.0, 0.055, 0.09), Vec3::new(0.4, 0.15, 0.57)),
                (Vec3::new(2.0, 0.055, 0.09), Vec3::new(0.4, 0.15, 3.23)),
                (Vec3::new(0.09, 0.055, 2.7), Vec3::new(-0.57, 0.15, 1.9)),
                (Vec3::new(0.09, 0.055, 2.7), Vec3::new(1.37, 0.15, 1.9)),
            ] {
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::from_size(size))),
                    MeshMaterial3d(frame.clone()),
                    Transform::from_translation(at),
                    Pickable::IGNORE,
                    TransientScene,
                ));
            }
        }
        let mut canceled_index = 0;
        let mut active_index = 0;
        let has_canceled = view
            .tokens
            .iter()
            .any(|token| token.status == TokenStatus::Canceled);
        for (i, token) in view.tokens.iter().enumerate() {
            let canceled = token.status == TokenStatus::Canceled;
            let x = if canceled {
                let x = -4.5 + canceled_index as f32 * 1.3;
                canceled_index += 1;
                x
            } else if has_canceled {
                let x = -0.6 + active_index as f32 * 1.3;
                active_index += 1;
                x
            } else {
                -1.8 + i as f32 * 1.30
            };
            let color = match token.status {
                TokenStatus::Pending => GOLD,
                TokenStatus::Resolved => Color::srgb(0.28, 0.62, 0.51),
                TokenStatus::Canceled => Color::srgb(0.37, 0.25, 0.28),
            };
            let is_new = rendered_view.as_ref().is_none_or(|previous| {
                !previous
                    .tokens
                    .iter()
                    .any(|held| held.id == token.id && held.face == token.face)
            });
            let mut token_entity = commands.spawn((
                Mesh3d(meshes.add(Cylinder::new(0.46, 0.12))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    metallic: 0.32,
                    perceptual_roughness: 0.5,
                    ..default()
                })),
                Transform::from_xyz(x, 0.15, -0.20).with_scale(Vec3::splat(if is_new {
                    0.5
                } else {
                    1.0
                })),
                Pickable::IGNORE,
                TransientScene,
            ));
            if is_new {
                token_entity.insert(Appearing);
            }
            if let Some((_, image)) = art
                .token_handles
                .iter()
                .find(|(face, _)| *face == token.face)
            {
                // The plane's UV top points toward -Z, upright from the default camera.
                // Parenting carries the original face through the cylinder's reveal animation.
                token_entity.with_child((
                    Mesh3d(meshes.add(Plane3d::default().mesh().size(0.88, 0.88))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: if canceled {
                            Color::srgb(0.35, 0.35, 0.35)
                        } else {
                            Color::WHITE
                        },
                        base_color_texture: Some(image.clone()),
                        alpha_mode: AlphaMode::Mask(0.5),
                        unlit: true,
                        ..default()
                    })),
                    Transform::from_xyz(0.0, 0.065, 0.0),
                    Pickable::IGNORE,
                ));
            }
            if canceled {
                let crossed = materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 0.55, 0.48),
                    unlit: true,
                    ..default()
                });
                for angle in [-std::f32::consts::FRAC_PI_4, std::f32::consts::FRAC_PI_4] {
                    token_entity.with_child((
                        Mesh3d(meshes.add(Cuboid::new(0.75, 0.02, 0.065))),
                        MeshMaterial3d(crossed.clone()),
                        Transform::from_xyz(0.0, 0.085, 0.0)
                            .with_rotation(Quat::from_rotation_y(angle)),
                        Pickable::IGNORE,
                    ));
                }
            }
            let status = match token.status {
                TokenStatus::Pending => "Revealed",
                TokenStatus::Resolved => "Resolved",
                TokenStatus::Canceled => "Canceled",
            };
            label(
                &mut commands,
                format!("#{}  {}\n{status}", token.id + 1, token.face.label()),
                Vec3::new(x, 0.1, -1.2),
                if token.status == TokenStatus::Canceled {
                    MUTED
                } else {
                    PAPER
                },
            );
        }
        *rendered_view = Some(view.clone());
    }
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(25),
                top: px(20),
                flex_direction: FlexDirection::Column,
                row_gap: px(5),
                ..default()
            },
            Pickable::IGNORE,
            Panel,
        ))
        .with_children(|parent| {
            parent.spawn(text("ARKHAM  /  UNFOLDED", 28.0, GOLD));
            parent.spawn(text(
                "Scripted rules experiment · The Gathering",
                16.0,
                PAPER,
            ));
            parent.spawn(text(
                "Prepared token order · one skill-test rules case",
                13.0,
                MUTED,
            ));
        });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(25),
                bottom: px(16),
                ..default()
            },
            Pickable::IGNORE,
            Panel,
        ))
        .with_child(text(
            "Right drag / arrows: orbit    Wheel / + −: zoom    Home: reset    F12: capture",
            13.0,
            MUTED,
        ));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: px(0),
                top: px(0),
                bottom: px(0),
                width: px(PANEL),
                padding: UiRect::all(px(24)),
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(Color::srgb(0.055, 0.085, 0.10)),
            ScrollPosition::default(),
            Panel,
        ))
        .observe(scroll_panel)
        .with_children(|parent| {
            let mode = table.replay.as_ref().map_or("LIVE DECISION".into(), |r| {
                format!("REPLAY  ·  MOMENT {} / {}", r.index() + 1, r.len())
            });
            parent.spawn(text(mode, 13.0, GOLD));
            parent.spawn(text("Rotting Remains", 27.0, PAPER));
            skill_summary(parent, &art, &view, 17.0, GOLD);
            parent.spawn(text(&view.prompt, 19.0, PAPER));
            if table.replay.is_none() {
                for choice in &view.choices {
                    button(
                        parent,
                        &choice.label,
                        Action::Choose(view.revision, choice.id.clone()),
                        choice.id != "end",
                    );
                }
            } else if let Some(replay) = &table.replay {
                if replay.index() + 1 < replay.len() {
                    button(parent, "Next moment  →", Action::Next, true);
                }
                if replay.index() > 0 {
                    button(parent, "←  Previous moment", Action::Back, false);
                }
                button(parent, "Return to live session", Action::Live, false);
            }
            if matches!(view.stage, Stage::Finished) {
                parent.spawn(text(
                    format!("{} horror from this test", view.horror_dealt),
                    20.0,
                    if view.horror_dealt > 0 {
                        Color::srgb(0.95, 0.59, 0.49)
                    } else {
                        Color::srgb(0.50, 0.80, 0.67)
                    },
                ));
                button(parent, "Show test result", Action::ShowResult, true);
            }
            parent.spawn(text(
                format!(
                    "Curse pool: {} / 10\nJacqueline's reaction: {}",
                    view.curses_in_pool,
                    if view.jacqueline_used {
                        "used this round"
                    } else {
                        "available"
                    }
                ),
                14.0,
                MUTED,
            ));
            parent.spawn(text(&table.notice, 14.0, PAPER));
            parent.spawn((Node {
                flex_grow: 1.0,
                ..default()
            },));
            button(parent, "Save live session", Action::Save, false);
            button(parent, "Reopen saved session", Action::Reopen, false);
            if table.replay.is_none() {
                button(parent, "Open recorded replay", Action::Replay, false);
            }
            if matches!(view.stage, Stage::Finished | Stage::Ended) {
                button(parent, "Start another example", Action::New, false);
            }
            button(
                parent,
                if table.history {
                    "Hide action history"
                } else {
                    "Explain / action history"
                },
                Action::History,
                false,
            );
            if table.replay.is_some() {
                button(
                    parent,
                    if orbit.follow {
                        "Camera: following · switch to manual"
                    } else {
                        "Camera: manual · follow replay"
                    },
                    Action::Follow,
                    false,
                );
            }
            parent.spawn(text(
                if art.available == 3 {
                    "Original card faces available. Click any card to inspect."
                } else {
                    "Readable card substitutes are active. Click any card to inspect."
                },
                12.0,
                MUTED,
            ));
        });
    if let Some(index) = table.inspect {
        let viewport = windows
            .iter()
            .next()
            .map_or(Vec2::new(1600.0, 1000.0), |window| {
                Vec2::new(window.width(), window.height())
            });
        inspection::show(&mut commands, &art, index, viewport);
    }
    if table.history {
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(24),
                    bottom: px(55),
                    width: px(700),
                    max_height: px(420),
                    padding: UiRect::all(px(20)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.035, 0.055, 0.065, 0.97)),
                ScrollPosition::default(),
                Panel,
            ))
            .observe(scroll_panel)
            .with_children(|parent| {
                parent.spawn(text("What happened — and why", 22.0, GOLD));
                for item in view
                    .history
                    .iter()
                    .rev()
                    .take(7)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                {
                    parent.spawn(text(item, 15.0, PAPER));
                }
                if view.history.len() > 7 {
                    parent.spawn(text(
                        "Earlier entries are available by stepping back in the recorded replay.",
                        12.0,
                        MUTED,
                    ));
                }
                button(parent, "Close history", Action::History, false);
            });
    }
    if table.result_open && view.stage == Stage::Finished {
        show_result(&mut commands, &view, &art);
    }
}

fn show_result(commands: &mut Commands, view: &Observation, art: &Art) {
    let failed = view.horror_dealt > 0;
    let accent = if failed {
        Color::srgb(0.95, 0.59, 0.49)
    } else {
        Color::srgb(0.50, 0.80, 0.67)
    };
    let resolved: Vec<_> = view
        .tokens
        .iter()
        .filter(|t| t.status == TokenStatus::Resolved)
        .collect();
    let auto_fail = resolved.iter().any(|t| t.face == TokenFace::AutoFail);
    let curse = resolved.iter().any(|t| t.face == TokenFace::Curse);
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: percent(100), height: percent(100),
            align_items: AlignItems::Center, justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.015, 0.025, 0.03, 0.78)),
        GlobalZIndex(100), ModalDialogBarrier, Panel,
    ))
    .observe(|_: On<RequestClose>, mut queue: ResMut<Actions>| queue.0.push(Action::DismissResult))
    .with_children(|parent| {
        parent.spawn((
            Node {
                width: px(640), max_width: percent(90), max_height: percent(88),
                padding: UiRect::all(px(30)), border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(10)),
                flex_direction: FlexDirection::Column, row_gap: px(16),
                overflow: Overflow::scroll_y(), ..default()
            },
            BackgroundColor(Color::srgb(0.055, 0.085, 0.10)), BorderColor::all(accent),
            ScrollPosition::default(), ModalDialog, TabGroup::modal(),
        ))
        .observe(scroll_panel)
        .with_children(|panel| {
            panel.spawn(Node {
                width: percent(100), flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center, row_gap: px(12), ..default()
            }).with_children(|header| {
                header.spawn((text("ROTTING REMAINS / SKILL TEST COMPLETE", 14.0, GOLD),
                    TextLayout::justify(Justify::Center)));
                header.spawn((text(if failed { "TEST FAILED" } else { "TEST SUCCEEDED" }, 36.0, accent),
                    TextLayout::justify(Justify::Center)));
                skill_summary(header, art, view, 24.0, PAPER);
                let amount = if failed { format!("+{} ", view.horror_dealt) } else { "No additional ".into() };
                symbol_text(header, &art.symbol_font,
                    &[Run::Words(&amount), Run::Icon(Symbol::Horror)],
                    &format!("{} additional horror", view.horror_dealt), 28.0, accent);
                if let Some(jacqueline) = view.investigators.iter().find(|i| i.name == "Jacqueline Fine") {
                    let total = if failed {
                        format!(" {} -> {} / {}", jacqueline.horror.saturating_sub(view.horror_dealt), jacqueline.horror, jacqueline.sanity)
                    } else {
                        format!(" {} / {} (unchanged)", jacqueline.horror, jacqueline.sanity)
                    };
                    let label = if failed {
                        format!("Jacqueline's horror: {} -> {} of {}.", jacqueline.horror.saturating_sub(view.horror_dealt), jacqueline.horror, jacqueline.sanity)
                    } else {
                        format!("Jacqueline's horror: {} of {} (unchanged).", jacqueline.horror, jacqueline.sanity)
                    };
                    symbol_text(header, &art.symbol_font,
                        &[Run::Words("Jacqueline: "), Run::Icon(Symbol::Horror), Run::Words(&total)],
                        &label, 18.0, PAPER);
                }
            });
            panel.spawn(text(format!("Resolved: {}", resolved.iter().map(|t| t.face.label()).collect::<Vec<_>>().join(", ")), 17.0, MUTED));
            if auto_fail {
                panel.spawn(text("The uncanceled auto-fail makes the test fail with a final skill value of 0. Numerical modifiers do not change that result.", 18.0, PAPER));
            }
            if curse {
                panel.spawn(text("Why another token?", 22.0, GOLD));
                panel.spawn(text("The Curse was not canceled: it applies -2 AND requires another reveal. The additional token is separate from that -2 modifier. This experiment uses a prepared token order.", 18.0, PAPER));
                // Show the recorded continuation, rather than re-running or reconstructing it.
                if let Some(index) = view.history.iter().position(|entry| entry.starts_with("The resolved curse")) {
                    for entry in view.history.iter().skip(index + 1).filter(|entry| entry.starts_with("Revealed ")) {
                            let explanation = entry.split(" [").next().unwrap_or(entry);
                            panel.spawn(text(explanation, 16.0, MUTED));
                    }
                }
            }
            panel.spawn((
                Button, Hovered::default(), TabIndex(0),
                Node { width: percent(100), min_height: px(48), flex_shrink: 0.0,
                    padding: UiRect::all(px(12)), align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(px(5)), ..default() },
                BackgroundColor(TEAL), ButtonTone(TEAL),
            ))
            .observe(|_: On<Activate>, mut queue: ResMut<Actions>| queue.0.push(Action::DismissResult))
            .with_child(text("Return to table", 18.0, PAPER));
        });
    });
}

fn label_positions(
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut labels: Query<(&WorldLabel, &mut Node, &mut Visibility)>,
) {
    for (anchor, mut node, mut visibility) in &mut labels {
        if let Ok(point) = camera.0.world_to_viewport(camera.1, anchor.0) {
            node.left = px(point.x - 65.0);
            node.top = px(point.y - 16.0);
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

fn animate_tokens(
    mut commands: Commands,
    time: Res<Time>,
    mut tokens: Query<(Entity, &mut Transform), With<Appearing>>,
) {
    for (entity, mut transform) in &mut tokens {
        transform.scale = transform
            .scale
            .lerp(Vec3::ONE, (time.delta_secs() * 10.0).min(1.0));
        if transform.scale.x > 0.995 {
            transform.scale = Vec3::ONE;
            commands.entity(entity).remove::<Appearing>();
        }
    }
}

fn animate_reminder(time: Res<Time>, mut reminder: Single<(&mut Transform, &mut ReminderPose)>) {
    let (transform, pose) = &mut *reminder;
    pose.angle += (pose.target - pose.angle) * (time.delta_secs() * 8.0).min(1.0);
    transform.rotation = Quat::from_rotation_z(pose.angle);
}

fn button_colors(
    mut buttons: Query<(&Hovered, Has<Pressed>, &ButtonTone, &mut BackgroundColor), With<Button>>,
) {
    for (hovered, pressed, tone, mut color) in &mut buttons {
        if pressed {
            color.0 = Color::srgb(0.25, 0.48, 0.43);
        } else if hovered.get() {
            color.0 = Color::srgb(0.20, 0.33, 0.34);
        } else {
            color.0 = tone.0;
        }
    }
}

fn scroll_panel(event: On<PointerScroll>, mut panels: Query<(&mut ScrollPosition, &ComputedNode)>) {
    if let Ok((mut position, node)) = panels.get_mut(event.entity) {
        let delta = match event.unit {
            MouseScrollUnit::Line => event.y * 28.0,
            MouseScrollUnit::Pixel => event.y,
        };
        let range = (node.content_size.y - node.size.y).max(0.0) * node.inverse_scale_factor;
        position.y = (position.y - delta).clamp(0.0, range);
    }
}

fn capture_window(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut capture: ResMut<Capture>,
    capturing: Query<(), With<Capturing>>,
    mut exit: MessageWriter<AppExit>,
) {
    capture.frame += 1;
    if keys.just_pressed(KeyCode::F12) {
        let path = "local/experiment/tabletop.png";
        if let Some(parent) = Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
    if !capture.requested
        && capture.frame >= 90
        && let Some(path) = &capture.path
    {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.clone()));
        capture.requested = true;
    }
    if capture.quit
        && capture.requested
        && capture.frame > 95
        && capturing.is_empty()
        && capture.path.as_ref().is_some_and(|path| path.is_file())
    {
        exit.write(AppExit::Success);
    }
}

#[cfg(test)]
mod result_modal_tests {
    use super::*;

    fn pending_app(path: PathBuf) -> App {
        let mut session = Session::new(Fixture::curse());
        session.choose(0, "commit-promise").unwrap();
        session.choose(1, "use-jacqueline").unwrap();
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
        .add_systems(Update, handle_actions);
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
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<Time>()
            .insert_resource(Art {
                handles: [None, None, None],
                available: 0,
                token_handles: vec![],
                reminder: None,
                investigators: Vec::new(),
                symbol_font: None,
            })
            .add_systems(
                Update,
                (rebuild_view, animate_tokens).chain().after(handle_actions),
            );
        app.world_mut().spawn(ReminderPose {
            angle: 0.0,
            target: 0.0,
        });
        act(&mut app, Action::Choose(2, "cancel-1-2".into()));
        let appearing = |app: &mut App| {
            let world = app.world_mut();
            world
                .query_filtered::<Entity, With<Appearing>>()
                .iter(world)
                .count()
        };
        assert_eq!(appearing(&mut app), 4, "newly revealed tokens animate");
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
        assert_eq!(appearing(&mut app), 0, "tokens have reached full size");
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
                appearing(&mut app),
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
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .insert_resource(Art {
                handles: [None, None, None],
                available: 0,
                token_handles: vec![],
                reminder: None,
                investigators: Vec::new(),
                symbol_font: None,
            })
            .add_systems(Update, rebuild_view.after(handle_actions));
        let reminder = app
            .world_mut()
            .spawn(ReminderPose {
                angle: 0.7,
                target: 0.0,
            })
            .id();
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
        act(&mut app, Action::History); // An unrelated rebuild must not restart the physical flip.
        assert_eq!(
            app.world().get::<ReminderPose>(reminder).unwrap().angle,
            0.7
        );
        act(&mut app, Action::Next);
        assert!(!has_text(&mut app, "Next moment"));
        assert!(has_text(&mut app, "Previous moment"));
    }
}
