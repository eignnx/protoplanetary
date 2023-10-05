use bevy::{
    prelude::{shape::UVSphere, *},
    utils::HashMap,
};

use crate::{
    components::{Mass, Radius, Velocity},
    planet::radius_from_mass,
};

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

pub struct PlanetInfo {
    pub entity: Entity,
    pub mass: Mass,
    pub vel: Velocity,
}

fn collision_resolution_system(
    mut commands: Commands,
    mut collision_groups: ResMut<CollisionGroups>,
    mut q_planets: Query<(
        Entity,
        &mut Handle<Mesh>,
        &mut Radius,
        &mut Velocity,
        &mut Mass,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut new_vels = HashMap::new();

    for group in collision_groups.map.values() {
        let gid = group.largest.entity;

        info!(
            "GROUP LARGEST {gid:?}: mass = {largest_mass:?}, radius = {radius}",
            largest_mass = group.largest.mass,
            radius = radius_from_mass(group.largest.mass.0),
        );

        let total_mass = group.largest.mass + group.members.iter().map(|p| p.mass).sum();

        info!("{gid:?} > TOTAL MASS: total mass = {total_mass:?}");

        let total_momentum = group.largest.vel.0 * group.largest.mass.0
            + group
                .members
                .iter()
                .inspect(|p| {
                    info!(
                        "{gid:?} > GROUP MEMBER {e:?}: mass = {m:?}",
                        e = p.entity,
                        m = p.mass
                    )
                })
                .map(|p| p.mass.0 * p.vel.0)
                .sum::<Vec3>();

        let new_v = total_momentum / total_mass.0;
        new_vels.insert(group.largest.entity, (total_mass, new_v));

        // Despawn all the group members (excluding `largest`).
        for planet in &group.members {
            commands.entity(planet.entity).despawn_recursive();
        }
    }

    for (e, mut mesh, mut rad, mut vel, mut mass) in q_planets.iter_mut() {
        if let Some((new_m, new_v)) = new_vels.get(&e) {
            *vel = Velocity(*new_v);
            *mass = *new_m;
            *rad = Radius(radius_from_mass(mass.0));
            info!("{e:?} > RESULT: mass = {new_m:?}, radius = {rad:?}");

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
