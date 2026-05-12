use crate::sim::KuramotoModel;
use eframe::egui::{self, Color32, FontId, Pos2, Stroke, Vec2};

const BG: Color32 = Color32::from_rgb(24, 24, 28);
const RING: Color32 = Color32::from_rgb(60, 60, 65);
const GRID: Color32 = Color32::from_rgb(38, 38, 42);
const ACCENT: Color32 = Color32::from_rgb(90, 140, 200);
const PANEL_BG: Color32 = Color32::from_rgb(30, 30, 34);
const TEXT_DIM: Color32 = Color32::from_rgb(130, 130, 140);
const TEXT: Color32 = Color32::from_rgb(200, 200, 210);
const DOT: Color32 = Color32::from_rgb(210, 210, 220);

pub struct KuramotoApp {
    model: KuramotoModel,
    n: usize,
    k: f32,
    dt: f64,
    paused: bool,
    steps_per_frame: usize,
    time: f64,
    r_history: Vec<f32>,
}

impl KuramotoApp {
    pub fn new(n: usize, k: f32, dt: f64) -> Self {
        Self {
            model: KuramotoModel::new(n, k as f64, dt),
            n,
            k,
            dt,
            paused: false,
            steps_per_frame: 10,
            time: 0.0,
            r_history: Vec::with_capacity(400),
        }
    }

    fn draw_phase_circle(&self, ui: &mut egui::Ui, center: Pos2, radius: f32) {
        let painter = ui.painter();

        // guide rings
        for i in 1..=4 {
            painter.circle_stroke(center, radius * (i as f32 / 4.0), Stroke::new(0.5, GRID));
        }

        // main ring
        painter.circle_stroke(center, radius, Stroke::new(1.5, RING));

        // tick marks
        let labels = ["0", "π/2", "π", "3π/2"];
        for (idx, &label) in labels.iter().enumerate() {
            let angle = idx as f32 * std::f32::consts::FRAC_PI_2;
            let dir = Vec2::new(angle.cos(), -angle.sin());
            let inner = center + dir * radius;
            let outer = center + dir * (radius + 6.0);
            painter.line_segment([inner, outer], Stroke::new(1.0, RING));
            painter.text(
                center + dir * (radius + 20.0),
                egui::Align2::CENTER_CENTER,
                label,
                FontId::proportional(11.0),
                TEXT_DIM,
            );
        }

        // oscillator dots
        let phases = self.model.phases();
        for i in 0..self.n {
            let a = phases[i] as f32;
            let pos = center + Vec2::new(radius * a.cos(), -radius * a.sin());
            painter.circle_filled(pos, 2.0, DOT);
        }

        // order parameter vector
        let (r, psi) = self.model.mean_field();
        let end = center
            + Vec2::new(
                (r as f32 * radius) * (psi as f32).cos(),
                -(r as f32 * radius) * (psi as f32).sin(),
            );

        painter.line_segment([center, end], Stroke::new(2.0, ACCENT));
        painter.circle_filled(end, 3.5, ACCENT);
        painter.circle_filled(center, 2.0, RING);

        painter.text(
            center + Vec2::new(0.0, radius + 38.0),
            egui::Align2::CENTER_TOP,
            format!("r = {:.4}", r),
            FontId::proportional(16.0),
            ACCENT,
        );
    }

    fn draw_r_history(&self, ui: &mut egui::Ui) {
        let width = ui.available_width();
        let height = 90.0;
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::hover());
        let painter = ui.painter_at(rect);

        painter.rect_filled(rect, 4.0, Color32::from_rgb(20, 20, 24));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, GRID));

        for i in 1..4 {
            let y = rect.top() + rect.height() * (i as f32 / 4.0);
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(0.5, GRID),
            );
        }

        if self.r_history.len() > 1 {
            let len = self.r_history.len();
            let points: Vec<Pos2> = self
                .r_history
                .iter()
                .enumerate()
                .map(|(i, &r)| {
                    Pos2::new(
                        rect.left() + 4.0 + (i as f32 / (len - 1) as f32) * (rect.width() - 8.0),
                        rect.bottom() - 4.0 - r.clamp(0.0, 1.0) * (rect.height() - 8.0),
                    )
                })
                .collect();

            for w in points.windows(2) {
                painter.line_segment([w[0], w[1]], Stroke::new(1.2, ACCENT));
            }
        }

        painter.text(
            Pos2::new(rect.left() + 3.0, rect.top() + 2.0),
            egui::Align2::LEFT_TOP,
            "1",
            FontId::proportional(9.0),
            TEXT_DIM,
        );
        painter.text(
            Pos2::new(rect.left() + 3.0, rect.bottom() - 2.0),
            egui::Align2::LEFT_BOTTOM,
            "0",
            FontId::proportional(9.0),
            TEXT_DIM,
        );
    }
}

impl eframe::App for KuramotoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.paused {
            for _ in 0..self.steps_per_frame {
                self.model.step();
                self.time += self.dt;
            }
            let r = self.model.order_parameter() as f32;
            self.r_history.push(r);
            if self.r_history.len() > 360 {
                self.r_history.remove(0);
            }
        }

        ctx.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                self.paused = !self.paused;
            }
            if i.key_pressed(egui::Key::R) {
                self.model = KuramotoModel::new(self.n, self.k as f64, self.dt);
                self.time = 0.0;
                self.r_history.clear();
            }
            if i.key_pressed(egui::Key::ArrowUp) {
                self.k = (self.k + 0.1).min(10.0);
                self.model.set_coupling(self.k as f64);
            }
            if i.key_pressed(egui::Key::ArrowDown) {
                self.k = (self.k - 0.1).max(0.0);
                self.model.set_coupling(self.k as f64);
            }
        });

        egui::SidePanel::right("controls")
            .min_width(250.0)
            .max_width(280.0)
            .frame(egui::Frame::none().fill(PANEL_BG).inner_margin(16.0))
            .show(ctx, |ui| {
                ui.visuals_mut().override_text_color = Some(TEXT);

                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Kuramoto Model").size(18.0).strong());
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new("dθ/dt = ω + (K/N) Σ sin(θj − θi)")
                            .size(11.0)
                            .color(TEXT_DIM),
                    );
                });

                ui.add_space(14.0);
                ui.separator();
                ui.add_space(8.0);

                ui.label(egui::RichText::new("Coupling K").size(12.0).color(TEXT_DIM));
                let slider = egui::Slider::new(&mut self.k, 0.0..=10.0)
                    .step_by(0.05)
                    .custom_formatter(|v, _| format!("{:.2}", v));
                if ui.add(slider).changed() {
                    self.model.set_coupling(self.k as f64);
                }

                ui.add_space(10.0);

                let (r, psi) = self.model.mean_field();

                ui.label(
                    egui::RichText::new(format!("r = {:.4}", r))
                        .size(14.0)
                        .color(ACCENT),
                );
                let bar_width = ui.available_width();
                let (bar_rect, _) =
                    ui.allocate_exact_size(Vec2::new(bar_width, 6.0), egui::Sense::hover());
                ui.painter()
                    .rect_filled(bar_rect, 3.0, Color32::from_rgb(40, 40, 46));
                let filled =
                    egui::Rect::from_min_size(bar_rect.min, Vec2::new(bar_width * r as f32, 6.0));
                ui.painter().rect_filled(filled, 3.0, ACCENT);

                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(format!(
                        "ψ = {:.4}   N = {}   t = {:.1}",
                        psi, self.n, self.time
                    ))
                    .size(12.0)
                    .color(TEXT_DIM),
                );

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    let label = if self.paused { "Play" } else { "Pause" };
                    if ui.button(label).clicked() {
                        self.paused = !self.paused;
                    }
                    if ui.button("Reset").clicked() {
                        self.model = KuramotoModel::new(self.n, self.k as f64, self.dt);
                        self.time = 0.0;
                        self.r_history.clear();
                    }
                });

                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new("Steps / frame")
                        .size(11.0)
                        .color(TEXT_DIM),
                );
                ui.add(egui::Slider::new(&mut self.steps_per_frame, 1..=50));

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(6.0);

                ui.label(egui::RichText::new("r(t)").size(12.0).color(TEXT_DIM));
                ui.add_space(4.0);
                self.draw_r_history(ui);

                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new("Space: pause | Up/Down: K | R: reset")
                        .size(10.0)
                        .color(Color32::from_rgb(70, 70, 80)),
                );
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG))
            .show(ctx, |ui| {
                let available = ui.available_size();
                let center = ui.min_rect().center();
                let radius = (available.x.min(available.y) * 0.40).min(350.0);
                self.draw_phase_circle(ui, center, radius);
            });

        ctx.request_repaint();
    }
}

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let candidates = [
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/Library/Fonts/Arial Unicode.ttf",
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/System/Library/Fonts/Supplemental/Times New Roman.ttf",
    ];

    for path in &candidates {
        if let Ok(data) = std::fs::read(path) {
            fonts.font_data.insert(
                "unicode_fallback".to_owned(),
                egui::FontData::from_owned(data).into(),
            );

            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("unicode_fallback".to_owned());
            break;
        }
    }

    ctx.set_fonts(fonts);
}

pub fn run(n: usize, k: f32, dt: f64) {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 750.0])
            .with_title("Kuramoto Model"),
        ..Default::default()
    };

    let app = KuramotoApp::new(n, k, dt);

    eframe::run_native(
        "Kuramoto Model",
        options,
        Box::new(move |cc| {
            configure_fonts(&cc.egui_ctx);
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(app))
        }),
    )
    .expect("Failed to launch visualizer");
}
