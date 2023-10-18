use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

use bevy::prelude::*;
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};

macro_rules! impl_zero_for {
    ($Type:ty = $z:expr) => {
        impl $Type {
            pub const ZERO: Self = Self($z);
        }
    };
}

macro_rules! impl_from_for {
    ($Inner:ty => $Type:ty) => {
        impl From<$Inner> for $Type {
            fn from(inner: $Inner) -> Self {
                Self(inner)
            }
        }
    };
}

macro_rules! impl_add_sub_for {
    ($Type:ty) => {
        impl Add for $Type {
            type Output = Self;
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl std::iter::Sum for $Type {
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(Self::ZERO, Add::add)
            }
        }

        impl AddAssign for $Type {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl Sub for $Type {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl SubAssign for $Type {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }
    };
}

macro_rules! impl_binop {
    ($Lhs:ty {$op:tt} $Rhs:ty = $Output:ty) => {
        impl_binop_with!($Lhs {$op} $Rhs = $Output {
            |a: $Lhs, b: $Rhs| (a.0 * b.0).into()
        });
    };
}

macro_rules! impl_binop_with {
    ($Lhs:ty {*} $Rhs:ty = $Output:ty { $body:expr }) => {
        impl Mul<$Rhs> for $Lhs {
            type Output = $Output;
            fn mul(self, rhs: $Rhs) -> $Output {
                $body(self, rhs)
            }
        }
    };
    ($Lhs:ty {/} $Rhs:ty = $Output:ty { $body:expr }) => {
        impl Div<$Rhs> for $Lhs {
            type Output = $Output;
            fn div(self, rhs: $Rhs) -> $Output {
                $body(self, rhs)
            }
        }
    };
}

macro_rules! impl_scalar {
    ($Type:ty) => {
        impl_from_for!(f32 => $Type);
        impl_zero_for!($Type = 0.0);
        impl_add_sub_for!($Type);
        impl_binop!($Type {*} $Type = $Type);
        impl_binop!($Type {/} $Type = $Type);
    };
}

macro_rules! impl_vector {
    ($Type:ty) => {
        impl_from_for!(Vec3 => $Type);
        impl_zero_for!($Type = Vec3::ZERO);
        impl_add_sub_for!($Type);
        impl_binop_with!(f32 {*} $Type = $Type { |a:f32, b: $Type| (a * b.0).into() });
        impl_binop_with!($Type {*} f32 = $Type { |a: $Type, b:f32| (a.0 * b).into() });
        impl_binop_with!($Type {/} f32 = $Type { |a: $Type, b:f32| (a.0 / b).into() });
    };
}

#[derive(Component, Resource, Default, Reflect, InspectorOptions, Debug, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct Mass(#[inspector(min = 0.0)] pub f32);

impl_scalar!(Mass);

#[derive(Component, Resource, Default, Reflect, InspectorOptions, Debug, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct Moment(pub Vec3);

impl_vector!(Moment);
impl_binop_with!(Mass {*} Vec3 = Moment { |a: Mass, b: Vec3| Moment(a.0 * b) });
impl_binop_with!(Moment {/} Mass = Vec3 { |a: Moment, b: Mass| a.0 / b.0 });

#[derive(Component, Resource, Default, Reflect, InspectorOptions, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct Velocity(pub Vec3);

impl_vector!(Velocity);

#[derive(Component, Resource, Default, Reflect, InspectorOptions, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct Force(pub Vec3);

impl_vector!(Force);
impl_binop!(Force {/} Mass = Acceleration);

#[derive(Component, Resource, Default, Reflect, InspectorOptions, Debug, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct Radius(#[inspector(min = 0.0)] pub f32);

impl_scalar!(Radius);

#[derive(Component, Resource, Default, Reflect, InspectorOptions, Debug, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct Momentum(pub Vec3);

impl_vector!(Momentum);
impl_binop!(Momentum {/} Mass = Velocity);
impl_binop!(Mass {*} Velocity = Momentum);
impl_binop!(Velocity {*} Mass = Momentum);

#[derive(Component, Resource, Default, Reflect, InspectorOptions, Debug, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct Acceleration(pub Vec3);

impl_vector!(Acceleration);
impl_binop!(Mass {*} Acceleration = Force);

#[derive(Component, Resource, Default, Reflect, InspectorOptions, Debug, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct Time(pub f32);

impl_scalar!(Time);
impl_binop!(Acceleration {*} Time = Velocity);
impl_binop!(Time {*} Acceleration = Velocity);
impl_binop!(Force {*} Time = Momentum);
impl_binop!(Time {*} Force = Momentum);
impl_binop_with!(Velocity {*} Time = Vec3 { |a: Velocity, b: Time| a.0 * b.0 });
