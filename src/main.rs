use bevy::{math::Vec3Swizzles, prelude::*, window::PrimaryWindow};
use bevy_inspector_egui::{
    prelude::ReflectInspectorOptions, quick::WorldInspectorPlugin, InspectorOptions,
};
use planet::{Drag, Planet, PlanetsPlugin};

mod planet;

const BACKGROUND_COLOR: Color = Color::rgb(0.1, 0.01, 0.02);

fn main() {
    App::new()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(IsDebugMode(false))
        .register_type::<Drag>()
        .add_plugins((DefaultPlugins, WorldInspectorPlugin::new(), PlanetsPlugin))
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
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 1000.0).looking_at(Vec3::splat(0.0), Vec3::Y),
            ..Default::default()
        },
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
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .and_then(|ray| Some(ray.get_point(ray.intersect_plane(Vec3::X, Vec3::Z)?)))
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
