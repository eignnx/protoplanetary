use std::f32::consts::{SQRT_2, TAU};

use bevy::prelude::*;

use crate::{
    planet::{SpawnPlanetEvent, Sun},
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
    HeightSelect { chosen_ecliptic_pos: Vec3 },
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

    if state.is_nothing() {
        return;
    }

    let sun_tsl = q_sun.single().translation;

    if matches!(*state, Mode::EclipticPosSelect) {
        let Some(mouse_tsl) = mouse_ray.intersect_plane(Vec3::ZERO, Vec3::Y) else {
            return;
        };
        let line_len = (sun_tsl - mouse_tsl).length();
        gizmos.line(sun_tsl, mouse_tsl, Color::GOLD);
        gizmos.rect(
            sun_tsl,
            Quat::from_rotation_arc(
                Vec3::new(1.0, 0.0, 1.0).normalize(),
                mouse_tsl.normalize_or_zero(),
            )
            .mul_quat(Quat::from_axis_angle(Vec3::X, TAU / 4.0)),
            Vec2::splat(SQRT_2 * line_len),
            Color::GOLD,
        );

        if input.just_released(MouseButton::Left) {
            *state = Mode::HeightSelect {
                chosen_ecliptic_pos: mouse_tsl,
            };
        }

        return;
    }

    if let &Mode::HeightSelect {
        chosen_ecliptic_pos,
    } = state.as_ref()
    {
        let cam = q_cam.single();
        let Some(mouse_tsl) = mouse_ray.intersect_plane(chosen_ecliptic_pos, cam.forward()) else {
            return;
        };

        let chosen_pos = chosen_ecliptic_pos + mouse_tsl.project_onto(Vec3::Y);
        gizmos.line(sun_tsl, chosen_ecliptic_pos, Color::GOLD);
        gizmos.line(sun_tsl, chosen_pos, Color::GOLD);
        gizmos.line(chosen_ecliptic_pos, chosen_pos, Color::GOLD);

        // gizmos.rect(
        //     sun_tsl,
        //     Quat::IDENTITY
        //         .mul_quat(Quat::from_axis_angle(
        //             Vec3::Y,
        //             chosen_pos.reject_from(Vec3::Y).angle_between(Vec3::X),
        //         ))
        //         .mul_quat(Quat::from_axis_angle(Vec3::X, TAU / 4.0)),
        //     Vec2::splat(SQRT_2 * (sun_tsl - chosen_pos).length()),
        //     Color::GOLD,
        // );

        if input.just_released(MouseButton::Left) {
            spawn_planet.send(SpawnPlanetEvent {
                pos: Some(chosen_pos),
                ..default()
            });
            *state = Mode::Nothing;
        }
    }
}
