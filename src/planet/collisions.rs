use bevy::{
    prelude::{shape::UVSphere, *},
    utils::HashMap,
};

use crate::{
    components::{Mass, Moment, Momentum, Radius, Velocity},
    planet::radius_from_mass,
};

use super::Planet;

pub struct CollisionResolutionPlugin;

impl Plugin for CollisionResolutionPlugin {
    fn build(&self, app: &mut App) {
        app // <autoformat ignore>
            .init_resource::<CollisionGroups>()
            .add_systems(PostUpdate, collision_resolution_system);
    }
}

#[derive(Resource, Default)]
pub struct CollisionGroups {
    pub map: HashMap<Entity, CollisionGroup>,
}

pub struct CollisionGroup {
    pub largest: PlanetInfo,
    pub members: Vec<PlanetInfo>,
}

impl CollisionGroup {
    fn iter_all_planets(&self) -> impl Iterator<Item = &PlanetInfo> {
        std::iter::once(&self.largest).chain(self.members.iter())
    }
}

pub struct PlanetInfo {
    pub entity: Entity,
    pub mass: Mass,
    pub vel: Velocity,
    pub pos: Vec3,
}

type CollisionResolutionPlanetsData<'a, 'b, 'c, 'd, 'e> = (
    Entity,
    &'a mut Handle<Mesh>,
    &'b mut Radius,
    &'c mut Velocity,
    &'d mut Mass,
    &'e mut Transform,
);

fn collision_resolution_system(
    mut commands: Commands,
    mut collision_groups: ResMut<CollisionGroups>,
    mut q_planets: Query<CollisionResolutionPlanetsData, With<Planet>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut new_phys_state = HashMap::new();

    for group in collision_groups.map.values() {
        let total_mass = group.iter_all_planets().map(|p| p.mass).sum::<Mass>();

        let total_momentum = group
            .iter_all_planets()
            .map(|p| p.mass * p.vel)
            .sum::<Momentum>();

        let center_of_mass = group
            .iter_all_planets()
            .map(|g| g.pos * g.mass)
            .sum::<Moment>()
            / total_mass;

        let new_v = total_momentum / total_mass;
        new_phys_state.insert(group.largest.entity, (total_mass, new_v, center_of_mass));

        // Despawn all the group members (excluding `largest`).
        for planet in &group.members {
            commands.entity(planet.entity).despawn_recursive();
        }
    }

    for (e, mut mesh, mut rad, mut vel, mut mass, mut tsf) in q_planets.iter_mut() {
        if let Some((new_m, new_v, center_of_mass)) = new_phys_state.get(&e) {
            *vel = *new_v;
            *mass = *new_m;
            *rad = radius_from_mass(*mass);
            tsf.translation = *center_of_mass;

            *mesh = meshes.set(
                mesh.as_ref(),
                UVSphere {
                    radius: rad.0,
                    ..default()
                }
                .try_into()
                .unwrap(),
            );
        }
    }

    collision_groups.map.clear();
}
