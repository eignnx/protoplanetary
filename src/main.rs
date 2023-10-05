use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
    },
    prelude::*,
    window::PrimaryWindow,
};
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use planet::{Planet, PlanetsPlugin};
use ui::MyUiPlugin;

mod components;
mod planet;
mod ui;

const BACKGROUND_COLOR: Color = Color::rgb(9. / 255., 1. / 255., 17. / 255.);

fn main() {
    App::new()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(IsDebugMode(false))
        .add_plugins((
            DefaultPlugins,
            PanOrbitCameraPlugin,
            PlanetsPlugin,
            MyUiPlugin,
        ))
        .add_systems(Startup, (init_camera, spawn))
        .add_systems(Update, (mouse_pos_update_system, toggle_debug_mode_system))
        .add_systems(PostUpdate, (spawn_debug_lines_system,))
        .run();
}

/// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

fn init_camera(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        PanOrbitCamera::default(),
        Camera3dBundle {
            camera: Camera {
                hdr: true, // 1. HDR is required for bloom
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
            transform: Transform::from_xyz(0.0, 0.0, 1000.0).looking_at(Vec3::splat(0.0), Vec3::Y),
            ..default()
        },
        BloomSettings {
            intensity: 0.01,
            low_frequency_boost: 2.0,
            low_frequency_boost_curvature: 1.0,
            high_pass_frequency: 100.0,
            composite_mode: BloomCompositeMode::Additive,
            ..default()
        }, // 3. Enable bloom for the camera
    ));
}

#[derive(Resource, Default, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct IsDebugMode(bool);

#[derive(Component)]
pub struct MouseDot;

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        MouseDot,
        Name::new("Mouse Dot"),
        PbrBundle {
            mesh: meshes.add(
                shape::Icosphere {
                    radius: 1.0,
                    ..default()
                }
                .try_into()
                .unwrap(),
            ),
            material: materials.add(StandardMaterial {
                emissive: Color::GREEN,
                ..default()
            }),
            ..default()
        },
    ));

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::rgba_u8(213, 211, 255, 255),
        brightness: 0.25,
    });
}

fn mouse_pos_update_system(
    mut q_mouse_dot: Query<&mut Transform, With<MouseDot>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let mouse_dot_pos = &mut q_mouse_dot.single_mut().translation;

    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = camera_q.single();

    // get the window that the camera is displaying to (or the primary window)
    let window = q_windows.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates.
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .and_then(|ray| Some(ray.get_point(ray.intersect_plane(Vec3::ZERO, Vec3::Y)?)))
    {
        *mouse_dot_pos = world_position;
    }
}

fn spawn_debug_lines_system(
    debug_mode: Res<IsDebugMode>,
    mut gizmos: Gizmos,
    q_mouse: Query<&Transform, With<MouseDot>>,
    q_balls: Query<&Transform, With<Planet>>,
) {
    if !debug_mode.0 {
        return;
    }

    let mouse_pos = q_mouse.single();
    for ball_pos in &q_balls {
        gizmos.line(
            ball_pos.translation,
            mouse_pos.translation,
            Color::Hsla {
                hue: 0.,
                saturation: 0.,
                lightness: 1.0,
                alpha: 0.1,
            },
        );
    }
}

fn toggle_debug_mode_system(mut debug_mode: ResMut<IsDebugMode>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Grave /* (tilde) */) {
        debug_mode.0 = !debug_mode.0;
    }
}
