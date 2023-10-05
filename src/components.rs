use std::ops::{Add, AddAssign, SubAssign};

use bevy::prelude::*;
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};

#[derive(Component, Resource, Default, Reflect, InspectorOptions, Debug, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct Mass(#[inspector(min = 0.0)] pub f32);

impl Add for Mass {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::iter::Sum for Mass {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|Self(m)| m).sum())
    }
}

#[derive(Component, Resource, Default, Reflect, InspectorOptions, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct Velocity(pub Vec3);

impl Velocity {
    pub const ZERO: Self = Self(Vec3::ZERO);
}

#[derive(Component, Resource, Default, Reflect, InspectorOptions, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct Force(pub Vec3);

impl Force {
    pub const ZERO: Self = Self(Vec3::ZERO);
}

impl AddAssign for Force {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign for Force {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

#[derive(Component, Resource, Default, Reflect, InspectorOptions, Debug, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct Radius(#[inspector(min = 0.0)] pub f32);
