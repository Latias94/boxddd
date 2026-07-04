use crate::control::{DebugDrawPreset, TestbedState};
use crate::scenes::{SCENE_REGISTRY, TestbedEntity};
use crate::{TestbedCamera, switch_scene};
use bevy::prelude::*;
use bevy_egui::egui::{LayerId, Ui, UiBuilder};
use bevy_egui::{EguiContexts, egui};

pub(crate) fn draw_testbed_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<TestbedState>,
    mut commands: Commands,
    entities: Query<Entity, With<TestbedEntity>>,
    mut camera: Query<&mut Transform, With<TestbedCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) -> Result {
    let mut requested_scene = None;

    let ctx = contexts.ctx_mut()?;
    let mut root_ui = Ui::new(
        ctx.clone(),
        "boxddd_testbed_root".into(),
        UiBuilder::new()
            .layer_id(LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    egui::Panel::left("boxddd_testbed_panel")
        .default_size(290.0)
        .min_size(250.0)
        .resizable(true)
        .show(&mut root_ui, |ui| {
            ui.heading(if state.scene_switching_enabled {
                "boxddd Testbed"
            } else {
                SCENE_REGISTRY[state.scene_index].name
            });
            ui.separator();

            ui.horizontal(|ui| {
                if ui
                    .button(if state.paused { "Resume" } else { "Pause" })
                    .clicked()
                {
                    state.paused = !state.paused;
                    if !state.paused {
                        state.cancel_single_step();
                    }
                }
                if ui.button("Reset").clicked() {
                    requested_scene = Some(state.scene_index);
                }
                if ui
                    .add_enabled(state.paused, egui::Button::new("Step"))
                    .clicked()
                {
                    state.request_single_step();
                }
            });

            ui.separator();
            if state.scene_switching_enabled {
                ui.label(egui::RichText::new("Scenes").strong());
                egui::ScrollArea::vertical()
                    .max_height(260.0)
                    .show(ui, |ui| {
                        let mut current_category = None;
                        for (index, metadata) in SCENE_REGISTRY.iter().enumerate() {
                            if current_category != Some(metadata.category) {
                                current_category = Some(metadata.category);
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new(metadata.category)
                                        .small()
                                        .color(egui::Color32::GRAY),
                                );
                            }

                            if ui
                                .selectable_label(state.scene_index == index, metadata.name)
                                .clicked()
                            {
                                requested_scene = Some(index);
                            }

                            if state.scene_index == index {
                                ui.label(egui::RichText::new(metadata.description).small());
                            }
                        }
                    });
            } else {
                let metadata = &SCENE_REGISTRY[state.scene_index];
                ui.label(
                    egui::RichText::new(metadata.category)
                        .small()
                        .color(egui::Color32::GRAY),
                );
                ui.label(metadata.description);
            }

            ui.separator();
            ui.label(egui::RichText::new("World").strong());
            ui.checkbox(&mut state.gravity_enabled, "Gravity");
            ui.checkbox(&mut state.sleeping_enabled, "Sleeping");
            ui.checkbox(&mut state.warm_starting_enabled, "Warm starting");
            ui.checkbox(&mut state.continuous_enabled, "Continuous collision");
            ui.add(
                egui::Slider::new(
                    &mut state.sub_step_count,
                    crate::control::MIN_SUB_STEPS..=crate::control::MAX_SUB_STEPS,
                )
                .text("Substeps"),
            );
            ui.add(
                egui::Slider::new(
                    &mut state.hertz,
                    crate::control::MIN_HERTZ..=crate::control::MAX_HERTZ,
                )
                .text("Hz"),
            );

            ui.separator();
            ui.label(egui::RichText::new("Debug").strong());
            egui::ComboBox::from_label("Draw")
                .selected_text(state.debug_preset.label())
                .show_ui(ui, |ui| {
                    for preset in DebugDrawPreset::ALL {
                        ui.selectable_value(&mut state.debug_preset, preset, preset.label());
                    }
                });
        });

    if let Some(scene_index) = requested_scene {
        switch_scene(
            scene_index,
            &mut state,
            &mut commands,
            &entities,
            &mut camera,
            &mut meshes,
            &mut materials,
        );
    }

    Ok(())
}
