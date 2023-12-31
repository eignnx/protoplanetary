use std::f32::consts::TAU;

use bevy::prelude::*;
use rand::prelude::*;

use crate::components::{self, Force, Mass, Radius, Velocity};

use self::collisions::{CollisionGroup, CollisionGroups, CollisionResolutionPlugin};

mod collisions;

#[derive(Resource)]
pub struct Constants {
    pub mouse_spring_strength: f32,
    pub grav_const: f32,
    pub min_attraction_dist: f32,
}

impl Default for Constants {
    fn default() -> Self {
        Self {
            mouse_spring_strength: 0.01,
            grav_const: 20.0,
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
            .add_systems(Update, (nbody_system,))
            .add_systems(PostUpdate, (physics_system, spawn_planet_system));
    }
}

#[derive(Component)]
pub struct Planet;

#[derive(Component)]
pub struct Sun;

const SUN_MASS: Mass = Mass(1000.0);

fn spawn_sun(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let radius = radius_from_mass(SUN_MASS);

    commands
        .spawn((
            PointLightBundle {
                transform: Transform::from_translation(Vec3::ZERO),
                point_light: PointLight {
                    intensity: 50_000_000.0,
                    range: 10_000.0,
                    radius: 3.0,
                    color: Color::ORANGE,
                    shadows_enabled: true,
                    ..default()
                },
                ..default()
            },
            Sun,
            Planet,
            Name::new("Sun"),
            radius,
            SUN_MASS,
            Velocity::ZERO,
            Force::ZERO,
        ))
        .with_children(|builder| {
            builder.spawn(PbrBundle {
                mesh: meshes.add(
                    shape::UVSphere {
                        radius: radius.0,
                        ..default()
                    }
                    .try_into()
                    .unwrap(),
                ),
                material: materials.add(StandardMaterial {
                    base_color: Color::lch(1.0, 0.15, 50.0),
                    emissive: Color::lch(1.5, 0.05, 74.0),
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
    pub vel: Option<Velocity>,
    pub mass: Option<Mass>,
}

pub fn radius_from_mass(mass: Mass) -> Radius {
    Radius(3.0 * mass.0.cbrt())
}

pub fn mass_from_radius(radius: Radius) -> Mass {
    Mass((radius.0 / 3.0).powi(3))
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
            .unwrap_or_else(|| Mass(50.0 * rng.gen_range(0.0..1.0) + 2.0));
        let radius = radius_from_mass(mass);

        let vel = event.vel.unwrap_or_else(|| {
            let orbit_speed = f32::sqrt(constants.grav_const * SUN_MASS.0 * pos.length_recip());
            Velocity(-orbit_speed * pos.normalize().cross(Vec3::Y))
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
            Name::new(format!("Planet (m={:.1})", mass.0)),
            radius,
            mass,
            vel,
            Force::ZERO,
            PbrBundle {
                mesh: meshes.add(
                    shape::UVSphere {
                        radius: radius.0,
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

fn physics_system(
    mut query: Query<(&mut Transform, &mut Velocity, &Mass, &mut Force)>,
    time: Res<Time>,
) {
    let dt = components::Time(time.delta_seconds());
    for (mut pos, mut vel, mass, mut net_force) in &mut query {
        let acc = *net_force / *mass;
        *vel += acc * dt;
        pos.translation += *vel * dt;
        *net_force = Force::ZERO;
    }
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
    while let Some([(e1, tsf1, &m1, &r1, &v1, mut f_net1), (e2, tsf2, &m2, &r2, &v2, mut f_net2)]) =
        it.fetch_next()
    {
        let (tsl1, tsl2) = (tsf1.translation, tsf2.translation);

        let sat_to_parent = tsl2 - tsl1;
        let radii_sum = r1 + r2;

        // Collision detection:
        if sat_to_parent.length_squared() < radii_sum.0 * radii_sum.0 {
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

            let (larger, smaller) = if m1 > m2 { (p1, p2) } else { (p2, p1) };

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
            let min_dist_sq = min_dist * min_dist;
            let toward_parent = sat_to_parent.normalize_or_zero();
            let r_sq = sat_to_parent.length_squared();

            grav_const * sat_mass * parent_mass * toward_parent / r_sq.max(min_dist_sq)
        };

        *f_net1 += Force(force);
        *f_net2 -= Force(force);
    }
}
