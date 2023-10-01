use std::ops::{AddAssign, SubAssign};

use bevy::prelude::*;
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
