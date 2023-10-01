use std::ops::{AddAssign, SubAssign};

use bevy::{math::Vec3Swizzles, prelude::*, window::PrimaryWindow};
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};

use crate::MouseDot;

#[derive(Component)]
pub struct Planet;

#[derive(Component, Clone, Copy, Default)]
pub struct Mass(pub f32);

#[derive(Component, Clone, Copy, Default)]
pub struct Velocity(pub Vec3);

#[derive(Component, Clone, Copy, Default)]
pub struct Acceleration(pub Vec3);

impl AddAssign for Acceleration {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign for Acceleration {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

#[derive(Component, Resource, Default, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct Drag(#[inspector(min = 0.0, max = 1.0)] pub f32);

pub struct PlanetsPlugin;

impl Plugin for PlanetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_planets)
            .add_systems(
                Update,
                (
                    drag_system,
                    nbody_system,
                    mouse_attraction_system,
                    player_bounds_system,
                ),
            )
            .add_systems(PostUpdate, (physics_system,));
    }
}

fn spawn_planets(
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
                transform: Transform::from_translation(jitter + Vec3 { z: i, ..default() })
                    .with_scale(3.0 * Vec3::splat(mass.cbrt())),
                ..default()
            },
        ));
    }
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

fn attraction_force(
    sat_mass: f32,
    sat_pos: Vec3,
    parent_mass: f32,
    parent_pos: Vec3,
    grav_const: f32,
) -> Vec3 {
    const MIN_DIST: f32 = 0.5;
    let sat_to_parent = parent_pos - sat_pos;
    let toward_parent = sat_to_parent.normalize_or_zero();
    grav_const * sat_mass * parent_mass * toward_parent
        / (sat_to_parent.length_squared() + MIN_DIST)
}

const GRAV_CONST: f32 = 0.2;

pub fn nbody_system(mut planets_mut: Query<(&Transform, &Mass, &mut Acceleration), With<Planet>>) {
    let mut it = planets_mut.iter_combinations_mut();
    while let Some([(p1_trans, m1, mut p1_acc), (p2_trans, m2, mut p2_acc)]) = it.fetch_next() {
        let force = attraction_force(
            m1.0,
            p1_trans.translation,
            m2.0,
            p2_trans.translation,
            GRAV_CONST,
        );
        *p1_acc += Acceleration(force / m1.0);
        *p2_acc -= Acceleration(force / m2.0);
    }
}

pub fn mouse_attraction_system(
    mouse_input: Res<Input<MouseButton>>,
    mut q_player: Query<(&Transform, &Mass, &mut Acceleration), With<Planet>>,
    q_mouse: Query<&Transform, With<MouseDot>>,
) {
    if !mouse_input.pressed(MouseButton::Left) {
        return;
    }

    const MOUSE_DOT_MASS: f32 = 2000.0;

    let mouse_pos = q_mouse.single().translation;

    for (player_pos, &Mass(mass), mut acc) in &mut q_player {
        let player_pos = player_pos.translation;
        *acc += Acceleration(
            attraction_force(mass, player_pos, MOUSE_DOT_MASS, mouse_pos, GRAV_CONST) / mass,
        );
    }
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
