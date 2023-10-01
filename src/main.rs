use bevy::{math::Vec3Swizzles, prelude::*, window::PrimaryWindow};
use bevy_inspector_egui::{
    prelude::ReflectInspectorOptions, quick::WorldInspectorPlugin, InspectorOptions,
};
use planet::{mouse_attraction_system, nbody_system, Drag};

use crate::planet::{Acceleration, Mass, Planet, Velocity};

mod planet;

const BACKGROUND_COLOR: Color = Color::rgb(0.1, 0.01, 0.02);
const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, 0.0, 0.0);

fn main() {
    App::new()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(IsDebugMode(false))
        .register_type::<Drag>()
        .add_plugins((DefaultPlugins, WorldInspectorPlugin::new()))
        .add_systems(Startup, (init_camera, spawn_player))
        .add_systems(PreUpdate, (drag_system,))
        .add_systems(
            Update,
            (
                physics_system,
                mouse_attraction_system,
                mouse_pos_update_system,
                player_bounds_system,
                toggle_debug_mode_system,
                nbody_system,
            ),
        )
        .add_systems(PostUpdate, (spawn_debug_lines_system,))
        .run();
}

fn init_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 1000.0).looking_at(Vec3::splat(0.0), Vec3::Y),
        ..Default::default()
    });
}

#[derive(Resource, Default, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct IsDebugMode(bool);

#[derive(Component)]
pub struct MouseDot;

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    const N: usize = 100;
    for i in 0..N {
        let i = i as f32 / N as f32;
        let jitter = Vec3::new(
            100.0 * (i * 19251.352 - 5.32).sin(),
            100.0 * (i * 13526.221).cos(),
            0.0,
        );

        const DRAG: f32 = 0.01;

        let mass = 50.0 * (1.0 - (i * 16236.0).sin().abs()) + 2.0;

        commands.spawn((
            Planet,
            Name::new("Planet"),
            Mass(mass),
            Velocity(0.025 * jitter),
            Acceleration(Vec3::splat(0.0)),
            Drag(0.01 + DRAG + (DRAG * (15000.0 * i).sin())),
            PbrBundle {
                mesh: meshes.add(shape::Icosphere::default().try_into().unwrap()),
                material: materials.add(StandardMaterial::from(Color::Hsla {
                    hue: 360.0 * i,
                    saturation: 0.5,
                    lightness: 0.5,
                    alpha: 1.0,
                })),
                transform: Transform::from_translation(
                    BALL_STARTING_POSITION + jitter + Vec3 { z: i, ..default() },
                )
                .with_scale(3.0 * Vec3::splat(mass.cbrt())),
                ..default()
            },
        ));
    }

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
        color: Color::WHITE,
        brightness: 0.4,
    });

    // point source light
    commands
        .spawn(PointLightBundle {
            transform: Transform::from_xyz(20.0, 50.0, -10.0),
            point_light: PointLight {
                intensity: 16000000.0, // lumens - roughly a 100W non-halogen incandescent bulb
                color: Color::RED,
                shadows_enabled: true,
                range: 11000.0,
                radius: 250.0,
                ..default()
            },
            ..default()
        })
        .with_children(|builder| {
            builder.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere {
                    radius: 0.1,
                    ..default()
                })),
                material: materials.add(StandardMaterial {
                    base_color: Color::RED,
                    emissive: Color::rgba_linear(7.13, 0.0, 0.0, 0.0),
                    ..default()
                }),
                ..default()
            });
        });
}

fn physics_system(mut query: Query<(&mut Transform, &mut Velocity, &mut Acceleration)>) {
    for (mut pos, mut vel, mut acc) in &mut query {
        vel.0 += acc.0;
        pos.translation += vel.0;
        *acc = Acceleration(Vec3::ZERO)
    }
}

fn drag_system(mut query: Query<(&mut Velocity, &Drag)>) {
    for (mut vel, Drag(drag)) in &mut query {
        vel.0 *= 1.0 - *drag;
    }
}

fn mouse_pos_update_system(
    mut q_mouse_dot: Query<&mut Transform, With<MouseDot>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    let mouse_dot_pos = &mut q_mouse_dot.single_mut().translation;
    let Ok(win) = q_windows.get_single() else { return };
    let Some(cursor_pos) = win.cursor_position() else { return };
    *mouse_dot_pos = cursor_pos.extend(0.0);
    mouse_dot_pos.y *= -1.0;
    mouse_dot_pos.x -= win.width() / 2.0;
    mouse_dot_pos.y += win.height() / 2.0;
}

fn player_bounds_system(
    mut q_player: Query<(&mut Transform, &mut Velocity), With<Planet>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(win) = q_windows.get_single() else { return };
    let win = Rect::new(
        -win.width() / 2.0,
        -win.height() / 2.0,
        win.width() / 2.0,
        win.height() / 2.0,
    );
    for (mut transform, mut vel) in &mut q_player {
        let mut pos = transform.translation;
        if win.contains(pos.xy()) {
            continue;
        }

        if !(win.min.x..win.max.x).contains(&pos.x) {
            pos.x = pos.x.clamp(win.min.x, win.max.x);
            vel.0.x *= -1.0;
        }

        if !(win.min.y..win.max.y).contains(&pos.y) {
            pos.y = pos.y.clamp(win.min.y, win.max.y);
            vel.0.y *= -1.0;
        }

        transform.translation = pos;
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
        gizmos.line_2d(
            ball_pos.translation.xy(),
            mouse_pos.translation.xy(),
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
