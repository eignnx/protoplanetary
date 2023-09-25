use std::ops::{AddAssign, SubAssign};

use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};

use crate::MouseDot;

#[derive(Component)]
pub struct Planet;

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

fn attraction_acceleration(
    sat_pos: Vec3,
    parent_mass: f32,
    parent_pos: Vec3,
    grav_const: f32,
) -> Acceleration {
    let sat_to_parent = parent_pos - sat_pos;
    let toward_parent = sat_to_parent.normalize_or_zero();
    Acceleration(grav_const * parent_mass * toward_parent / (sat_to_parent.length_squared() + 0.1))
}

const GRAV_CONST: f32 = 10.0;
pub fn nbody_system(mut planets_mut: Query<(&Transform, &mut Acceleration), With<Planet>>) {
    let mut bodies = Vec::<(&Transform, Mut<Acceleration>)>::new();

    for (p1_trans, mut p1_acc) in planets_mut.iter_mut() {
        p1_acc.0 = Vec3::ZERO;
        for (p2_trans, p2_acc) in bodies.iter_mut() {
            let acc = attraction_acceleration(
                p1_trans.translation,
                10.0,
                p2_trans.translation,
                GRAV_CONST,
            );
            *p1_acc += acc;
            **p2_acc -= acc;
        }
        bodies.push((p1_trans, p1_acc));
    }
}

pub fn mouse_attraction_system(
    mouse_input: Res<Input<MouseButton>>,
    mut q_player: Query<(&Transform, &mut Acceleration), With<Planet>>,
    q_mouse: Query<&Transform, With<MouseDot>>,
) {
    if !mouse_input.pressed(MouseButton::Left) {
        return;
    }

    let mouse_pos = q_mouse.single().translation;
    for (player_pos, mut acc) in &mut q_player {
        let player_pos = player_pos.translation;
        const MOUSE_DOT_MASS: f32 = 150.0;
        *acc = attraction_acceleration(player_pos, MOUSE_DOT_MASS, mouse_pos, GRAV_CONST);
    }
}
