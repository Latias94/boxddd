use crate::control::{DebugDrawPreset, TestbedState};
use crate::lab::LabDiagnostics;
use crate::scenes::{SCENE_REGISTRY, TestbedEntity, TestbedScene};
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
    diagnostics: Res<LabDiagnostics>,
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
                                ui.label(
                                    egui::RichText::new(metadata.source_label())
                                        .small()
                                        .color(egui::Color32::LIGHT_GRAY),
                                );
                                if let Some(lesson) = metadata.showcase_lesson {
                                    ui.label(egui::RichText::new(lesson).small());
                                }
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
                ui.label(
                    egui::RichText::new(metadata.source_label())
                        .small()
                        .color(egui::Color32::LIGHT_GRAY),
                );
                if let Some(lesson) = metadata.showcase_lesson {
                    ui.label(egui::RichText::new(lesson).small());
                }
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

            draw_scene_lab_controls(ui, state.as_mut(), &diagnostics);
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

fn draw_scene_lab_controls(ui: &mut Ui, state: &mut TestbedState, diagnostics: &LabDiagnostics) {
    match crate::lab::current_scene(state) {
        TestbedScene::QueryLab => {
            ui.separator();
            ui.label(egui::RichText::new("Query Lab").strong());
            ui.add(
                egui::Slider::new(
                    &mut state.query_lab_ray_length,
                    crate::control::MIN_QUERY_RAY_LENGTH..=crate::control::MAX_QUERY_RAY_LENGTH,
                )
                .text("Ray length"),
            );
            ui.add(
                egui::Slider::new(
                    &mut state.query_lab_aabb_half_extent,
                    crate::control::MIN_QUERY_AABB_HALF_EXTENT
                        ..=crate::control::MAX_QUERY_AABB_HALF_EXTENT,
                )
                .text("AABB half extent"),
            );
            ui.add(
                egui::Slider::new(
                    &mut state.query_lab_shape_cast_length,
                    crate::control::MIN_QUERY_SHAPE_CAST_LENGTH
                        ..=crate::control::MAX_QUERY_SHAPE_CAST_LENGTH,
                )
                .text("Shape cast length"),
            );
            ui.add(
                egui::Slider::new(
                    &mut state.query_lab_shape_cast_radius,
                    crate::control::MIN_QUERY_SHAPE_CAST_RADIUS
                        ..=crate::control::MAX_QUERY_SHAPE_CAST_RADIUS,
                )
                .text("Shape radius"),
            );
            ui.add(
                egui::Slider::new(
                    &mut state.query_lab_mover_cast_length,
                    crate::control::MIN_QUERY_MOVER_CAST_LENGTH
                        ..=crate::control::MAX_QUERY_MOVER_CAST_LENGTH,
                )
                .text("Mover length"),
            );
            if diagnostics.query_ray_supported {
                ui.label(format!("Ray hits: {}", diagnostics.query_ray_hit_count));
            } else {
                ui.label("Ray hits: unavailable in browser provider mode");
            }
            if diagnostics.query_overlap_supported {
                ui.label(format!(
                    "Overlap hits: {}",
                    diagnostics.query_overlap_hit_count
                ));
            } else {
                ui.label("Overlap hits: unavailable in browser provider mode");
            }
            ui.label(match diagnostics.query_closest_fraction {
                Some(fraction) => format!("Closest fraction: {fraction:.3}"),
                None => "Closest fraction: none".to_string(),
            });
            if diagnostics.query_shape_cast_supported {
                ui.label(format!(
                    "Shape hits: {}",
                    diagnostics.query_shape_cast_hit_count
                ));
                ui.label(match diagnostics.query_shape_cast_closest_fraction {
                    Some(fraction) => format!("Shape fraction: {fraction:.3}"),
                    None => "Shape fraction: none".to_string(),
                });
            } else {
                ui.label("Shape cast: unavailable in browser provider mode");
            }
            if diagnostics.query_mover_supported {
                ui.label(match diagnostics.query_mover_fraction {
                    Some(fraction) => format!("Mover fraction: {fraction:.3}"),
                    None => "Mover fraction: none".to_string(),
                });
            } else {
                ui.label("Mover fraction: unavailable in browser provider mode");
            }
            if diagnostics.query_mover_planes_supported {
                ui.label(format!(
                    "Mover planes: {}",
                    diagnostics.query_mover_plane_count
                ));
            } else {
                ui.label("Mover planes: unavailable in browser provider mode");
            }
        }
        TestbedScene::DebugDrawInspector => {
            ui.separator();
            ui.label(egui::RichText::new("Debug Draw Inspector").strong());
            ui.horizontal_wrapped(|ui| {
                if ui.button("Shapes").clicked() {
                    state.debug_preset = DebugDrawPreset::Shapes;
                }
                if ui.button("Joints").clicked() {
                    state.debug_preset = DebugDrawPreset::ShapesAndJoints;
                }
                if ui.button("Contacts").clicked() {
                    state.debug_preset = DebugDrawPreset::Contacts;
                }
                if ui.button("Bounds").clicked() {
                    state.debug_preset = DebugDrawPreset::Bounds;
                }
            });
            ui.label(format!("Commands: {}", diagnostics.debug_command_count));
            ui.label(format!("Events: {}", diagnostics.debug_event_count));
            ui.label(format!(
                "Diagnostics: {}",
                diagnostics.debug_diagnostic_count
            ));
        }
        TestbedScene::MaterialLab => {
            ui.separator();
            ui.label(egui::RichText::new("Material Lab").strong());
            ui.add(
                egui::Slider::new(
                    &mut state.material_lab_friction,
                    crate::control::MIN_MATERIAL_FRICTION..=crate::control::MAX_MATERIAL_FRICTION,
                )
                .text("Friction"),
            );
            ui.add(
                egui::Slider::new(
                    &mut state.material_lab_restitution,
                    crate::control::MIN_MATERIAL_RESTITUTION
                        ..=crate::control::MAX_MATERIAL_RESTITUTION,
                )
                .text("Restitution"),
            );
            ui.label(format!(
                "Native shapes: {}",
                diagnostics.material_shape_count
            ));
        }
        _ => {}
    }
}
