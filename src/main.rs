use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
    },
    prelude::*,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use planet::PlanetsPlugin;
use ui::MyUiPlugin;

mod components;
mod planet;
mod ui;

const BACKGROUND_COLOR: Color = Color::rgb(9. / 255., 1. / 255., 17. / 255.);

fn main() {
    App::new()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_plugins((
            DefaultPlugins,
            PanOrbitCameraPlugin,
            PlanetsPlugin,
            MyUiPlugin,
        ))
        .add_systems(Startup, (init_camera, spawn))
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
            tonemapping: Tonemapping::AcesFitted, // 2. Using a tonemapper that desaturates to white is recommended
            transform: Transform::from_xyz(0.0, 200.0, 1000.0)
                .looking_at(Vec3::splat(0.0), Vec3::Y),
            ..default()
        },
        BloomSettings {
            intensity: 0.025,
            low_frequency_boost: 2.0,
            low_frequency_boost_curvature: 1.0,
            high_pass_frequency: 100.0,
            composite_mode: BloomCompositeMode::Additive,
            ..default()
        }, // 3. Enable bloom for the camera
    ));
}

fn spawn(mut commands: Commands) {
    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::rgba_u8(213, 211, 255, 255),
        brightness: 0.25,
    });
}
