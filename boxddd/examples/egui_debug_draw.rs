mod common;

use anyhow::{Context, Result};
use common::{BodySnapshot, VisualShape};
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};

fn main() -> Result<()> {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default().with_inner_size([980.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "boxddd egui debug draw",
        options,
        Box::new(|_| Ok(Box::new(DebugDrawApp::new()?))),
    )
    .map_err(|error| anyhow::anyhow!("{error}"))
}

struct DebugDrawApp {
    scene: common::DemoScene,
    snapshots: Vec<BodySnapshot>,
    paused: bool,
    show_labels: bool,
    sub_steps: i32,
}

impl DebugDrawApp {
    fn new() -> Result<Self> {
        let scene = common::falling_stack_scene().context("failed to create demo scene")?;
        let snapshots = scene.snapshots()?;
        Ok(Self {
            scene,
            snapshots,
            paused: false,
            show_labels: true,
            sub_steps: 4,
        })
    }

    fn step(&mut self) {
        if !self.paused {
            self.scene
                .step(1.0 / 60.0, self.sub_steps)
                .expect("demo scene should step");
            self.snapshots = self.scene.snapshots().expect("demo snapshots should read");
        }
    }
}

impl eframe::App for DebugDrawApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.step();
        ctx.request_repaint();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .button(if self.paused { "Resume" } else { "Pause" })
                    .clicked()
                {
                    self.paused = !self.paused;
                }
                ui.checkbox(&mut self.show_labels, "Labels");
                ui.add(egui::Slider::new(&mut self.sub_steps, 1..=12).text("sub-steps"));
                ui.label(format!("bodies: {}", self.snapshots.len()));
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            let rect = ui.available_rect_before_wrap();
            let painter = ui.painter_at(rect);
            draw_grid(&painter, rect);
            for snapshot in &self.snapshots {
                draw_body(&painter, rect, snapshot, self.show_labels);
            }
        });
    }
}

fn draw_grid(painter: &egui::Painter, rect: Rect) {
    painter.rect_filled(rect, 0.0, Color32::from_rgb(18, 22, 26));
    let stroke = Stroke::new(1.0, Color32::from_rgb(45, 52, 58));
    for i in -8..=8 {
        let a = project(rect, i as f32, -1.0);
        let b = project(rect, i as f32, 8.0);
        painter.line_segment([a, b], stroke);
    }
    for i in -1..=8 {
        let a = project(rect, -8.0, i as f32);
        let b = project(rect, 8.0, i as f32);
        painter.line_segment([a, b], stroke);
    }
    let ground_a = project(rect, -7.5, 0.0);
    let ground_b = project(rect, 7.5, 0.0);
    painter.line_segment([ground_a, ground_b], Stroke::new(3.0, Color32::LIGHT_GRAY));
}

fn draw_body(painter: &egui::Painter, rect: Rect, snapshot: &BodySnapshot, show_label: bool) {
    let x = snapshot.position.x as f32;
    let y = snapshot.position.y as f32;
    let center = project(rect, x, y);
    let scale = world_scale(rect);
    let radius = (snapshot.radius * scale).max(3.0);
    let color = match snapshot.shape {
        VisualShape::Cube => Color32::from_rgb(96, 174, 255),
        VisualShape::Sphere => Color32::from_rgb(255, 196, 87),
    };

    match snapshot.shape {
        VisualShape::Cube => {
            let half = Vec2::splat(radius);
            painter.rect_filled(Rect::from_min_max(center - half, center + half), 2.0, color);
        }
        VisualShape::Sphere => {
            painter.circle_filled(center, radius, color);
        }
    }

    if show_label {
        painter.text(
            center + Vec2::new(radius + 4.0, -radius),
            egui::Align2::LEFT_TOP,
            snapshot.label,
            egui::FontId::monospace(12.0),
            Color32::WHITE,
        );
    }
}

fn project(rect: Rect, x: f32, y: f32) -> Pos2 {
    let scale = world_scale(rect);
    Pos2::new(
        rect.center().x + x * scale,
        rect.bottom() - 64.0 - y * scale,
    )
}

fn world_scale(rect: Rect) -> f32 {
    (rect.width() / 18.0).min(rect.height() / 10.0)
}
