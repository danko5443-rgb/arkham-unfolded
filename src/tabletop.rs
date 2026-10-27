//! The replaceable tabletop for the single, explicitly scripted rules experiment.
//! Every displayed decision and replay moment comes from the shared rules interface.

use crate::{
    Fixture, Observation, Session, Stage, TokenStatus,
    continuity::{self, Replay},
};
use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit},
    picking::hover::Hovered,
    prelude::*,
    render::view::screenshot::{Capturing, Screenshot, save_to_disk},
    ui::Pressed,
    ui_widgets::{Activate, Button},
};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

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
}

const CARDS: [Card; 3] = [
    Card {
        code: "01163",
        title: "Rotting Remains",
        kind: "Treachery · The Gathering",
        text: "Revelation — Test willpower (3). For each point you fail by, take 1 horror.",
        color: Color::srgb(0.40, 0.22, 0.18),
    },
    Card {
        code: "07032",
        title: "Promise of Power",
        kind: "Skill · Four wild icons · Level 0",
        text: "After you commit Promise of Power to a skill test, add 1 curse token to the chaos bag. If you cannot, take 2 horror instead.",
        color: Color::srgb(0.29, 0.20, 0.42),
    },
    Card {
        code: "60401",
        title: "Jacqueline Fine",
        kind: "Investigator · Willpower 5",
        text: "Reaction — When an investigator at your location would reveal any number of chaos tokens: Reveal 2 additional tokens. Of the revealed tokens, choose and cancel 2 non-auto-fail tokens, or 1 auto-fail token. Limit once per round.\n\nElder sign: +1. If this effect is canceled or ignored, draw 1 card.\n\nThis experiment has no elder sign reveal or other replacement assets.",
        color: Color::srgb(0.30, 0.24, 0.48),
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
                handle_actions,
                camera_controls,
                rebuild_view,
                label_positions,
                animate_tokens,
                button_colors,
                capture_window,
            )
                .chain(),
        );
    let handles = load_art(app.world().resource::<AssetServer>(), art_root.as_deref());
    let available = handles.iter().filter(|h| h.is_some()).count();
    app.insert_resource(Art { handles, available });
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
        let bytes = std::fs::read(root.join(path)).ok()?;
        // A changed or missing image is a missing presentation asset, never a rules block.
        if format!("{:x}", Sha256::digest(&bytes)) != face["sha256"].as_str()? {
            return None;
        }
        Some(server.load(path.to_owned()))
    })
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
            Transform::from_xyz(x, 0.08, 1.9),
            Pickable::IGNORE,
        ));
        commands
            .spawn((
                Mesh3d(meshes.add(Plane3d::default().mesh().size(1.79, 2.54))),
                MeshMaterial3d(material),
                Transform::from_xyz(x, 0.125, 1.9),
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
}

fn handle_actions(
    mut queue: ResMut<Actions>,
    mut table: ResMut<Tabletop>,
    mut orbit: ResMut<Orbit>,
) {
    for action in queue.0.drain(..) {
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
                        orbit.follow = true;
                        table.notice = "Replay reopened from its saved record. Next advances one moment and pauses.".into();
                    }
                    Err(e) => table.notice = format!("Could not reopen the replay: {e}"),
                }
            }
            Action::Back => {
                if let Some(replay) = &mut table.replay {
                    replay.back();
                }
            }
            Action::Next => {
                if let Some(replay) = &mut table.replay {
                    replay.next();
                }
            }
            Action::Live => {
                table.replay = None;
                orbit.follow = false;
                table.notice = "Returned to the live session; replay made no changes to it.".into();
            }
            Action::New => {
                let session = Session::new(table.fixture.clone());
                match continuity::restart_session(&table.save_path, &session) {
                    Ok(()) => {
                        table.session = session;
                        table.replay = None;
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
    let scene_pointer = table.inspect.is_none()
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
        if keys.pressed(key) {
            orbit.yaw += yaw * time.delta_secs();
            orbit.pitch += pitch * time.delta_secs();
            orbit.distance += zoom * time.delta_secs();
            changed = true;
        }
    }
    if keys.just_pressed(KeyCode::Home) {
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
    let value = value
        .into()
        .replace(['−', '–'], "-")
        .replace('§', "section ")
        .replace('·', " / ")
        .replace('—', " - ")
        .replace('→', ">")
        .replace('←', "<");
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    )
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
) {
    if !table.dirty {
        return;
    }
    table.dirty = false;
    for entity in &panels {
        commands.entity(entity).despawn();
    }
    for entity in &transient {
        commands.entity(entity).despawn();
    }
    let view = table.view();
    label(&mut commands, "STUDY", Vec3::new(0.0, 0.1, -3.55), GOLD);
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
        let material = materials.add(StandardMaterial {
            base_color: colors[i % colors.len()],
            metallic: 0.12,
            perceptual_roughness: 0.62,
            ..default()
        });
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.53, 0.14))),
            MeshMaterial3d(materials.add(GOLD)),
            Transform::from_xyz(x, 0.1, z),
            Pickable::IGNORE,
            TransientScene,
        ));
        commands.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.25, 0.72))),
            MeshMaterial3d(material),
            Transform::from_xyz(x, 0.8, z),
            Pickable::IGNORE,
            TransientScene,
        ));
        label(
            &mut commands,
            format!(
                "{}\n{} · Horror {}/{}",
                investigator.name, investigator.location, investigator.horror, investigator.sanity
            ),
            Vec3::new(x, 1.65, z),
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
            Vec3::new(-2.0 + i as f32 * 2.4, 0.2, 3.36),
            PAPER,
        );
    }
    for (i, token) in view.tokens.iter().enumerate() {
        let x = -1.8 + i as f32 * 1.30;
        let color = match token.status {
            TokenStatus::Pending => GOLD,
            TokenStatus::Resolved => Color::srgb(0.28, 0.62, 0.51),
            TokenStatus::Canceled => Color::srgb(0.37, 0.25, 0.28),
        };
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.46, 0.12))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                metallic: 0.32,
                perceptual_roughness: 0.5,
                ..default()
            })),
            Transform::from_xyz(x, 0.15, -0.20).with_scale(Vec3::splat(0.5)),
            Pickable::IGNORE,
            TransientScene,
            Appearing,
        ));
        let status = match token.status {
            TokenStatus::Pending => "Revealed",
            TokenStatus::Resolved => "Resolved",
            TokenStatus::Canceled => "Canceled",
        };
        label(
            &mut commands,
            format!("#{}  {}\n{status}", token.id + 1, token.face.label()),
            Vec3::new(x, 0.35, -0.20),
            if token.status == TokenStatus::Canceled {
                MUTED
            } else {
                PAPER
            },
        );
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
            parent.spawn(text("A moment of doubt", 27.0, PAPER));
            parent.spawn(text(
                format!(
                    "WILLPOWER {}   /   TEST {}",
                    view.skill_value, view.difficulty
                ),
                17.0,
                GOLD,
            ));
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
            } else {
                button(parent, "Next moment  →", Action::Next, true);
                button(parent, "←  Previous moment", Action::Back, false);
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
        let card = &CARDS[index];
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(24),
                    top: px(120),
                    width: px(350),
                    padding: UiRect::all(px(22)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(13),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(7)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.07, 0.10, 0.115)),
                BorderColor::all(GOLD),
                Panel,
            ))
            .with_children(|parent| {
                parent.spawn(text(card.title, 25.0, GOLD));
                parent.spawn(text(card.kind, 14.0, MUTED));
                if let Some(image) = &art.handles[index] {
                    parent.spawn((
                        ImageNode::new(image.clone()),
                        Node {
                            width: px(175),
                            height: px(245),
                            align_self: AlignSelf::Center,
                            ..default()
                        },
                    ));
                }
                parent.spawn(text(card.text, 16.0, PAPER));
                button(parent, "Close card", Action::CloseInspect, false);
            });
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
