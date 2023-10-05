use std::f32::consts::TAU;

use bevy::prelude::*;
use rand::prelude::*;

use crate::{
    components::{Force, Mass, Radius, Velocity},
    MouseDot,
};

use self::collisions::{CollisionGroup, CollisionGroups, CollisionResolutionPlugin};

mod collisions;

#[derive(Resource)]
pub struct Constants {
    pub mouse_dot_mass: f32,
    pub grav_const: f32,
    pub min_attraction_dist: f32,
}

impl Default for Constants {
    fn default() -> Self {
        Self {
            mouse_dot_mass: 2000.0,
            grav_const: 0.2,
            min_attraction_dist: 0.001,
        }
    }
}

pub struct PlanetsPlugin;

impl Plugin for PlanetsPlugin {
    fn build(&self, app: &mut App) {
        app // <no autoformat>
            .register_type::<Mass>()
            .register_type::<Radius>()
            .register_type::<Velocity>()
            .register_type::<Force>()
            .add_event::<SpawnPlanetEvent>()
            .init_resource::<Constants>()
            .add_plugins(CollisionResolutionPlugin)
            .add_systems(Startup, (spawn_planets, spawn_sun))
            .add_systems(PreUpdate, (spawn_planet_system,))
            .add_systems(
                Update,
                (nbody_system, mouse_attraction_system, bounds_system),
            )
            .add_systems(PostUpdate, (physics_system,));
    }
}

#[derive(Component)]
pub struct Planet;

const SUN_MASS: f32 = 1000.0;

fn spawn_sun(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let radius: f32 = radius_from_mass(SUN_MASS);

    commands
        .spawn((
            PointLightBundle {
                transform: Transform::from_translation(Vec3::ZERO),
                point_light: PointLight {
                    intensity: 10_000_000.0,
                    range: 10_000.0,
                    radius: 3.0,
                    color: Color::ORANGE,
                    shadows_enabled: true,
                    ..default()
                },
                ..default()
            },
            Name::new("Sun"),
            Planet,
            Radius(radius),
            Mass(SUN_MASS),
            Velocity::ZERO,
            Force::ZERO,
        ))
        .with_children(|builder| {
            builder.spawn(PbrBundle {
                mesh: meshes.add(
                    shape::UVSphere {
                        radius,
                        ..default()
                    }
                    .try_into()
                    .unwrap(),
                ),
                material: materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    emissive: Color::ORANGE,
                    ..default()
                }),
                transform: Transform::from_translation(Vec3::ZERO),
                ..default()
            });
        });
}

#[derive(Event, Default, Clone, Copy)]
pub struct SpawnPlanetEvent {
    pub pos: Option<Vec3>,
    pub vel: Option<Vec3>,
    pub mass: Option<f32>,
}

fn radius_from_mass(mass: f32) -> f32 {
    3.0 * mass.cbrt()
}

fn spawn_planet_system(
    mut ereader: EventReader<SpawnPlanetEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    constants: Res<Constants>,
) {
    let mut rng = thread_rng();

    for event in ereader.iter() {
        let pos = event.pos.unwrap_or_else(|| {
            rng.gen_range(50.0..500.0)
                * (Quat::from_axis_angle(Vec3::Y, rng.gen_range(0.0..TAU)).mul_vec3(Vec3::X)
                    + rng.gen_range(-0.1..0.1) * Vec3::Y)
        });

        let mass = event
            .mass
            .unwrap_or_else(|| 50.0 * rng.gen_range(0.0..1.0) + 2.0);
        let radius: f32 = radius_from_mass(mass);

        let vel = event.vel.unwrap_or_else(|| {
            let orbit_speed =
                0.025 * f32::sqrt(constants.grav_const * SUN_MASS * mass * pos.length_recip());
            -orbit_speed * pos.normalize().cross(Vec3::Y)
        });

        let material = StandardMaterial {
            base_color: Color::Hsla {
                hue: 360.0 * rng.gen_range(0.0..1.0),
                saturation: 0.5,
                lightness: 0.5,
                alpha: 1.0,
            },
            perceptual_roughness: 0.9,
            metallic: 0.5,
            reflectance: 0.1,
            fog_enabled: true,
            ..default()
        };

        commands.spawn((
            Planet,
            Name::new(format!("Planet (m={mass:.1})")),
            Radius(radius),
            Mass(mass),
            Velocity(vel),
            Force::ZERO,
            PbrBundle {
                mesh: meshes.add(
                    shape::UVSphere {
                        radius,
                        ..default()
                    }
                    .try_into()
                    .unwrap(),
                ),
                material: materials.add(material),
                transform: Transform::from_translation(pos),
                ..default()
            },
        ));
    }
}

fn spawn_planets(mut ewriter: EventWriter<SpawnPlanetEvent>) {
    const N: usize = 25;
    ewriter.send_batch(std::iter::repeat(SpawnPlanetEvent::default()).take(N));
}

fn physics_system(mut query: Query<(&mut Transform, &mut Velocity, &Mass, &mut Force)>) {
    for (mut pos, mut vel, mass, mut net_force) in &mut query {
        vel.0 += net_force.0 / mass.0;
        pos.translation += vel.0;
        *net_force = Force::ZERO;
    }
}

fn attraction_force(
    sat_mass: f32,
    sat_pos: Vec3,
    parent_mass: f32,
    parent_pos: Vec3,
    grav_const: f32,
    min_dist: f32,
) -> Vec3 {
    let sat_to_parent = parent_pos - sat_pos;
    let toward_parent = sat_to_parent.normalize_or_zero();
    grav_const * sat_mass * parent_mass * toward_parent
        / (sat_to_parent.length_squared() + min_dist)
}

type NBodyPlanetsData<'a, 'b, 'c, 'd, 'e> = (
    Entity,
    &'a Transform,
    &'b Mass,
    &'c Radius,
    &'d Velocity,
    &'e mut Force,
);

fn nbody_system(
    mut planets_mut: Query<NBodyPlanetsData, With<Planet>>,
    constants: Res<Constants>,
    mut collision_groups: ResMut<CollisionGroups>,
) {
    let mut it = planets_mut.iter_combinations_mut();
    while let Some([(e1, tsf1, &m1, &r1, &v1, mut acc1), (e2, tsf2, &m2, &r2, &v2, mut acc2)]) =
        it.fetch_next()
    {
        let (tsl1, tsl2) = (tsf1.translation, tsf2.translation);

        let sat_to_parent = tsl2 - tsl1;
        let radii_sum = r1.0 + r2.0;

        // Collision detection:
        if sat_to_parent.length() < radii_sum {
            use collisions::PlanetInfo;

            let p1 = PlanetInfo {
                entity: e1,
                mass: m1,
                vel: v1,
                pos: tsl1,
            };

            let p2 = PlanetInfo {
                entity: e2,
                mass: m2,
                vel: v2,
                pos: tsl2,
            };

            let (larger, smaller) = if m1.0 > m2.0 { (p1, p2) } else { (p2, p1) };

            collision_groups
                .map
                .entry(larger.entity)
                .or_insert(CollisionGroup {
                    largest: larger,
                    members: vec![],
                })
                .members
                .push(smaller);

            // Skip rest of force computation.
            continue;
        }

        let force = {
            let sat_mass = m1.0;
            let parent_mass = m2.0;
            let grav_const = constants.grav_const;
            let min_dist = constants.min_attraction_dist;
            let toward_parent = sat_to_parent.normalize_or_zero();
            grav_const * sat_mass * parent_mass * toward_parent
                / (sat_to_parent.length_squared() + min_dist)
        };
        *acc1 += Force(force / m1.0);
        *acc2 -= Force(force / m2.0);
    }
}

pub fn mouse_attraction_system(
    mouse_input: Res<Input<MouseButton>>,
    mut q_player: Query<(&Transform, &Mass, &mut Force), With<Planet>>,
    q_mouse: Query<&Transform, With<MouseDot>>,
    constants: Res<Constants>,
) {
    if !mouse_input.pressed(MouseButton::Left) {
        return;
    }

    let mouse_pos = q_mouse.single().translation;

    for (player_pos, &Mass(mass), mut acc) in &mut q_player {
        let player_pos = player_pos.translation;
        *acc += Force(
            attraction_force(
                mass,
                player_pos,
                constants.mouse_dot_mass,
                mouse_pos,
                constants.grav_const,
                constants.min_attraction_dist,
            ) / mass,
        );
    }
}

fn bounds_system(//
    // mut q_player: Query<(&mut Transform, &mut Velocity), With<Planet>>,
    // q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    // let Ok(win) = q_windows.get_single() else { return };
    // let (w, h) = (win.width(), win.height());
    // let d = 0.5 * (w + h);

    // let bounds = bevy::prelude::shape::Box {
    //     min_x: -0.5 * w,
    //     max_x: 0.5 * w,
    //     min_y: -0.5 * h,
    //     max_y: 0.5 * h,
    //     min_z: -0.5 * d,
    //     max_z: 0.5 * d,
    // };

    // for (mut transform, mut vel) in &mut q_player {
    //     let mut pos = transform.translation;
    //     if bounds..contains(pos) {
    //         continue;
    //     }

    //     if !(win.min.x..win.max.x).contains(&pos.x) {
    //         pos.x = pos.x.clamp(win.min.x, win.max.x);
    //         vel.0.x *= -1.0;
    //     }

    //     if !(win.min.y..win.max.y).contains(&pos.y) {
    //         pos.y = pos.y.clamp(win.min.y, win.max.y);
    //         vel.0.y *= -1.0;
    //     }

    //     transform.translation = pos;
    // }
}
