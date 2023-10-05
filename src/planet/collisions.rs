use bevy::{
    prelude::{shape::UVSphere, *},
    utils::HashMap,
};

use crate::{
    components::{Mass, Radius, Velocity},
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

fn collision_resolution_system(
    mut commands: Commands,
    mut collision_groups: ResMut<CollisionGroups>,
    mut q_planets: Query<
        (
            Entity,
            &mut Handle<Mesh>,
            &mut Radius,
            &mut Velocity,
            &mut Mass,
            &mut Transform,
        ),
        With<Planet>,
    >,
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

        let total_mass = group.iter_all_planets().map(|p| p.mass).sum::<Mass>();

        info!("{gid:?} > TOTAL MASS: total mass = {total_mass:?}");

        let total_momentum = group
            .iter_all_planets()
            .inspect(|p| {
                info!(
                    "{gid:?} > GROUP MEMBER {e:?}: mass = {m:?}",
                    e = p.entity,
                    m = p.mass
                )
            })
            .map(|p| p.mass.0 * p.vel.0)
            .sum::<Vec3>();

        let center_of_mass = group
            .iter_all_planets()
            .map(|g| g.pos * g.mass.0)
            .sum::<Vec3>()
            / total_mass.0;

        let new_v = total_momentum / total_mass.0;
        new_vels.insert(group.largest.entity, (total_mass, new_v, center_of_mass));

        // Despawn all the group members (excluding `largest`).
        for planet in &group.members {
            commands.entity(planet.entity).despawn_recursive();
        }
    }

    for (e, mut mesh, mut rad, mut vel, mut mass, mut tsf) in q_planets.iter_mut() {
        if let Some((new_m, new_v, center_of_mass)) = new_vels.get(&e) {
            *vel = Velocity(*new_v);
            *mass = *new_m;
            *rad = Radius(radius_from_mass(mass.0));
            tsf.translation = *center_of_mass;

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
