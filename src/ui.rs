use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::{egui, EguiContexts, EguiPlugin},
    quick::WorldInspectorPlugin,
};

use crate::planet::SpawnPlanetEvent;

pub struct MyUiPlugin;

#[derive(Resource)]
pub struct UiState {
    right_panel_open: bool,
    world_inspector_open: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for UiState {
    fn default() -> Self {
        Self {
            right_panel_open: false,
            world_inspector_open: false,
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
    mut ewriter: EventWriter<SpawnPlanetEvent>,
) {
    egui::containers::SidePanel::right("my_side_panel").show_animated(
        contexts.ctx_mut(),
        state.right_panel_open,
        |ui| {
            ui.heading("Dev Menu");

            if ui.button("Close Panel").clicked() {
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

            if ui.button("Spawn Planet").clicked() {
                ewriter.send(SpawnPlanetEvent);
            }
        },
    );

    if input.just_pressed(KeyCode::P) {
        state.right_panel_open = !state.right_panel_open;
    }
}
