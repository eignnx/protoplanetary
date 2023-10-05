use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::{egui, EguiContexts, EguiPlugin},
    egui::{CollapsingHeader, DragValue},
    quick::WorldInspectorPlugin,
};
use rand::Rng;

use crate::planet::{Constants, SpawnPlanetEvent};

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
            ))
            .insert_resource(UiState::default())
            .add_systems(Update, root_ui_system);
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
    mut ewriter: EventWriter<SpawnPlanetEvent>,
) {
    egui::containers::SidePanel::right("my_side_panel").show_animated(
        contexts.ctx_mut(),
        state.right_panel_open,
        |ui| {
            ui.heading("Dev Menu");
            ui.separator();

            if ui.button("Close Thi Panel").clicked() {
                state.right_panel_open = false;
            }

            ui.horizontal(|ui| {
                ui.label("World Inspector Window");
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
                    ui.label("Position");

                    ui.add(
                        DragValue::new(&mut state.new_planet_pos.x)
                            .speed(0.1)
                            .prefix("x: "),
                    );
                    ui.add(
                        DragValue::new(&mut state.new_planet_pos.y)
                            .speed(0.1)
                            .prefix("y: "),
                    );
                    ui.add(
                        DragValue::new(&mut state.new_planet_pos.z)
                            .speed(0.1)
                            .prefix("z: "),
                    );

                    if ui.small_button("Randomize").clicked() {
                        let mut rng = rand::thread_rng();
                        state.new_planet_pos = rng.gen_range(50.0..600.0)
                            * Vec3::new(
                                rng.gen_range(-1.0..1.0),
                                rng.gen_range(-1.0..1.0),
                                rng.gen_range(-1.0..1.0),
                            )
                            .normalize_or_zero();
                    }

                    if ui.button("Spawn Planet").clicked() {
                        ewriter.send(SpawnPlanetEvent {
                            pos: Some(state.new_planet_pos),
                            ..default()
                        });
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

    if input.just_pressed(KeyCode::P) {
        state.right_panel_open = !state.right_panel_open;
    }
}
