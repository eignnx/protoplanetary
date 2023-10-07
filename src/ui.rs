use bevy::{prelude::*, window::PrimaryWindow};
use bevy_inspector_egui::{
    bevy_egui::{egui, EguiContexts, EguiPlugin},
    egui::{CollapsingHeader, DragValue},
    quick::WorldInspectorPlugin,
};
use rand::Rng;

use crate::{
    planet::{Constants, SpawnPlanetEvent},
    MainCamera,
};

use self::planet_spawning::{PlanetSpawnMode, PlanetSpawningPlugin};

mod planet_spawning;

pub struct MyUiPlugin;

#[derive(Resource)]
pub struct UiState {
    right_panel_open: bool,
    world_inspector_open: bool,
    new_planet_pos: Vec3,
}

#[allow(clippy::derivable_impls)]
impl Default for UiState {
    fn default() -> Self {
        Self {
            right_panel_open: false,
            world_inspector_open: false,
            new_planet_pos: Vec3::ZERO,
        }
    }
}

impl Plugin for MyUiPlugin {
    fn build(&self, app: &mut App) {
        // Systems that create Egui widgets should be run during the `CoreSet::Update` set,
        // or after the `EguiSet::BeginFrame` system (which belongs to the `CoreSet::PreUpdate` set).
        app // <rustfmt ignore>
            .add_plugins((
                EguiPlugin,
                WorldInspectorPlugin::new().run_if(world_inspector_open),
                PlanetSpawningPlugin,
            ))
            .insert_resource(UiState::default())
            .insert_resource(MouseRay::default())
            .add_systems(Update, (mouse_ray_update_system,))
            .add_systems(Update, (root_ui_system,));
    }
}

fn world_inspector_open(state: Res<UiState>) -> bool {
    state.world_inspector_open
}

fn root_ui_system(
    mut contexts: EguiContexts,
    mut state: ResMut<UiState>,
    input: Res<Input<KeyCode>>,
    mut constants: ResMut<Constants>,
    mut spawn_events: EventWriter<SpawnPlanetEvent>,
    mut planet_spawn_mode: ResMut<PlanetSpawnMode>,
) {
    if input.just_pressed(KeyCode::W) {
        state.world_inspector_open = !state.world_inspector_open;
    }

    if input.just_pressed(KeyCode::P) {
        state.right_panel_open = !state.right_panel_open;
    }

    if input.just_pressed(KeyCode::R) {
        spawn_events.send(SpawnPlanetEvent::default());
    }

    if input.just_pressed(KeyCode::S) {
        *planet_spawn_mode = PlanetSpawnMode::EclipticPosSelect;
    }

    egui::containers::SidePanel::right("my_side_panel").show_animated(
        contexts.ctx_mut(),
        state.right_panel_open,
        |ui| {
            ui.heading("Dev Menu");
            ui.separator();

            if ui.button("Close This [P]anel").clicked() {
                state.right_panel_open = false;
            }

            ui.horizontal(|ui| {
                ui.label("[W]orld Inspector Window");
                let txt = if state.world_inspector_open {
                    "Close"
                } else {
                    "Open"
                };
                if ui.button(txt).clicked() {
                    state.world_inspector_open = !state.world_inspector_open;
                }
            });

            ui.separator();

            CollapsingHeader::new("Spawn Planet")
                .default_open(true)
                .show(ui, |ui| {
                    if ui.small_button("Spawn [R]andom").clicked()
                        || input.just_released(KeyCode::R)
                    {
                        let mut rng = rand::thread_rng();
                        state.new_planet_pos = rng.gen_range(50.0..600.0)
                            * Vec3::new(
                                rng.gen_range(-1.0..1.0),
                                rng.gen_range(-1.0..1.0),
                                rng.gen_range(-1.0..1.0),
                            )
                            .normalize_or_zero();
                        spawn_events.send(SpawnPlanetEvent {
                            pos: Some(state.new_planet_pos),
                            ..default()
                        });
                    }

                    if input.just_pressed(KeyCode::Escape) {
                        planet_spawn_mode.go_back();
                    }

                    if ui
                        .add_enabled(
                            planet_spawn_mode.is_nothing(),
                            egui::Button::new("[S]pawn At Mouse"),
                        )
                        .clicked()
                    {
                        *planet_spawn_mode = PlanetSpawnMode::EclipticPosSelect;
                    }
                });

            CollapsingHeader::new("Constants")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Gravitational Const.");
                        ui.add(
                            DragValue::new(&mut constants.grav_const)
                                .speed(0.1)
                                .clamp_range(0.0..=f32::MAX),
                        );
                        egui::reset_button_with(
                            ui,
                            &mut constants.grav_const,
                            Constants::default().grav_const,
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Min. Attraction Dist.");
                        ui.add(
                            DragValue::new(&mut constants.min_attraction_dist)
                                .speed(0.1)
                                .clamp_range(0.0..=f32::MAX),
                        );
                        egui::reset_button_with(
                            ui,
                            &mut constants.min_attraction_dist,
                            Constants::default().min_attraction_dist,
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Mouse Interaction Strength");
                        ui.add(
                            DragValue::new(&mut constants.mouse_spring_strength)
                                .speed(0.1)
                                .clamp_range(0.0..=f32::MAX),
                        );
                        egui::reset_button_with(
                            ui,
                            &mut constants.mouse_spring_strength,
                            Constants::default().mouse_spring_strength,
                        );
                    });
                });
        },
    );
}

#[derive(Resource, Default)]
pub struct MouseRay(pub Option<Ray>);

impl MouseRay {
    pub fn intersect_plane(&self, plane_origin: Vec3, plane_normal: Vec3) -> Option<Vec3> {
        let ray = self.0?;
        let dist_along_ray = ray.intersect_plane(plane_origin, plane_normal)?;
        Some(ray.get_point(dist_along_ray))
    }
}

fn mouse_ray_update_system(
    mut mouse_ray: ResMut<MouseRay>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = camera_q.single();

    // get the window that the camera is displaying to (or the primary window)
    let window = q_windows.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates.
    *mouse_ray = MouseRay(
        window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor)),
    );
}
