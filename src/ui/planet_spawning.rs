use std::f32::consts::{SQRT_2, TAU};

use bevy::prelude::*;

use crate::{
    planet::{mass_from_radius, SpawnPlanetEvent, Sun},
    MainCamera,
};

use super::MouseRay;

pub struct PlanetSpawningPlugin;

impl Plugin for PlanetSpawningPlugin {
    fn build(&self, app: &mut App) {
        app // <noformat>
            .insert_resource(PlanetSpawnMode::Nothing)
            .add_systems(Update, (planet_spawn_interaction_system,));
    }
}

#[derive(Resource, Clone, Copy)]
pub enum PlanetSpawnMode {
    Nothing,
    EclipticPosSelect,
    HeightSelect {
        chosen_ecliptic_pos: Vec3,
    },
    RadiusSelect {
        chosen_ecliptic_pos: Vec3,
        chosen_pos: Vec3,
    },
}

impl PlanetSpawnMode {
    pub fn is_nothing(&self) -> bool {
        matches!(self, Self::Nothing)
    }

    pub fn go_back(&mut self) {
        *self = match *self {
            Self::Nothing => Self::Nothing,
            Self::EclipticPosSelect => Self::Nothing,
            Self::HeightSelect { .. } => Self::EclipticPosSelect,
            Self::RadiusSelect {
                chosen_ecliptic_pos,
                ..
            } => Self::HeightSelect {
                chosen_ecliptic_pos,
            },
        };
    }
}

fn planet_spawn_interaction_system(
    q_sun: Query<&Transform, With<Sun>>,
    mouse_ray: Res<MouseRay>,
    mut state: ResMut<PlanetSpawnMode>,
    mut gizmos: Gizmos,
    input: Res<Input<MouseButton>>,
    q_cam: Query<&Transform, With<MainCamera>>,
    mut spawn_planet: EventWriter<SpawnPlanetEvent>,
) {
    use PlanetSpawnMode as Mode;

    match state.as_ref() {
        Mode::Nothing => (),

        Mode::EclipticPosSelect => {
            let sun_tsl = q_sun.single().translation;
            let Some(mouse_tsl) = mouse_ray.intersect_plane(Vec3::ZERO, Vec3::Y) else {
            return;
        };
            let line_len = (sun_tsl - mouse_tsl).length();
            gizmos.line(sun_tsl, mouse_tsl, Color::CYAN);
            gizmos.rect(
                sun_tsl,
                Quat::from_rotation_arc(
                    Vec3::new(1.0, 0.0, 1.0).normalize(),
                    mouse_tsl.normalize_or_zero(),
                )
                .mul_quat(Quat::from_axis_angle(Vec3::X, TAU / 4.0)),
                Vec2::splat(SQRT_2 * line_len),
                Color::CYAN,
            );

            if input.just_released(MouseButton::Left) {
                *state = Mode::HeightSelect {
                    chosen_ecliptic_pos: mouse_tsl,
                };
            }
        }

        &Mode::HeightSelect {
            chosen_ecliptic_pos,
        } => {
            let sun_tsl = q_sun.single().translation;
            let cam = q_cam.single();
            let Some(mouse_tsl) = mouse_ray.intersect_plane(chosen_ecliptic_pos, cam.forward()) else {
            return;
        };

            let chosen_pos = chosen_ecliptic_pos + mouse_tsl.project_onto(Vec3::Y);
            gizmos.line(sun_tsl, chosen_ecliptic_pos, Color::GOLD);
            gizmos.line(sun_tsl, chosen_pos, Color::CYAN);
            gizmos.line(chosen_ecliptic_pos, chosen_pos, Color::CYAN);

            let line_len = (sun_tsl - chosen_ecliptic_pos).length();
            gizmos.rect(
                sun_tsl,
                Quat::from_rotation_arc(
                    Vec3::new(1.0, 0.0, 1.0).normalize(),
                    chosen_ecliptic_pos.normalize_or_zero(),
                )
                .mul_quat(Quat::from_axis_angle(Vec3::X, TAU / 4.0)),
                Vec2::splat(SQRT_2 * line_len),
                Color::GOLD,
            );

            if input.just_released(MouseButton::Left) {
                *state = Mode::RadiusSelect {
                    chosen_ecliptic_pos,
                    chosen_pos,
                };
            }
        }

        &Mode::RadiusSelect {
            chosen_ecliptic_pos,
            chosen_pos,
        } => {
            let sun_tsl = q_sun.single().translation;
            let cam = q_cam.single();
            let Some(mouse_tsl) = mouse_ray.intersect_plane(chosen_pos, cam.forward()) else {
            return;
        };

            gizmos.line(sun_tsl, chosen_ecliptic_pos, Color::GOLD);
            gizmos.line(sun_tsl, chosen_pos, Color::GOLD);
            gizmos.line(chosen_ecliptic_pos, chosen_pos, Color::GOLD);

            let line_len = (sun_tsl - chosen_ecliptic_pos).length();
            gizmos.rect(
                sun_tsl,
                Quat::from_rotation_arc(
                    Vec3::new(1.0, 0.0, 1.0).normalize(),
                    chosen_ecliptic_pos.normalize_or_zero(),
                )
                .mul_quat(Quat::from_axis_angle(Vec3::X, TAU / 4.0)),
                Vec2::splat(SQRT_2 * line_len),
                Color::GOLD,
            );

            let radius = 2.5 * (mouse_tsl - chosen_pos).length().sqrt();

            gizmos.sphere(chosen_pos, Quat::IDENTITY, radius, Color::CYAN);

            if input.just_released(MouseButton::Left) {
                spawn_planet.send(SpawnPlanetEvent {
                    pos: Some(chosen_pos),
                    mass: Some(mass_from_radius(radius)),
                    ..default()
                });
                *state = Mode::Nothing;
            }
        }
    }
}
