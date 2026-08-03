use nih_plug::prelude::Editor;
use nih_plug_egui::{EguiState, create_egui_editor, egui};
use std::sync::Arc;

use gog_common::{
    setup_industrial_style, bento_frame, section_header, bento_button, bento_slider, ACCENT_RED, TEXT_DARK
};

// Импортируем структуру параметров из lib.rs
use crate::{Flt01Params, FilterType};

pub fn create_ui(
    editor_state: Arc<EguiState>,
    params: Arc<Flt01Params>,
) -> Option<Box<dyn Editor>> {
    let params_clone = params.clone();

    create_egui_editor(
        editor_state,
        params,
        |_, _| {},
        move |egui_ctx, param_setter, _state| {

            setup_industrial_style(egui_ctx);

            egui::CentralPanel::default().show(egui_ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading(egui::RichText::new("GOG // FLT-01").strong().color(egui::Color32::WHITE));
                    ui.set_width(325.0);

                    // 01 // TELEMETRY
                    bento_frame().show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.set_width(ui.available_width());
                            section_header(ui, "01 // TELEMETRY");
                            ui.add_space(8.0);

                            let (response, painter) = ui.allocate_painter(
                                egui::vec2(ui.available_width(), 120.0), 
                                egui::Sense::hover(),
                            );
                            let rect = response.rect;

                            let freq_to_x = |freq: f32| -> f32 {
                                let min_f = 20.0f32.log10();
                                let max_f = 20000.0f32.log10();
                                let norm = (freq.log10() - min_f) / (max_f - min_f);
                                rect.left() + norm * rect.width()
                            };

                            let grid_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(60));
                            let sub_grid_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(40));

                            let font_id = egui::FontId::monospace(10.0);
                            let text_color = egui::Color32::from_gray(80);

                            let grid_freqs = [100.0, 1000.0, 10000.0];

                            for &freq in &grid_freqs {
                                let x = freq_to_x(freq);
                                painter.line_segment(
                                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                                    grid_stroke,
                                );
                            }

                            let sub_grid_freqs = [50.0, 200.0, 500.0, 2000.0, 5000.0];
                            for &freq in &sub_grid_freqs {
                                let x = freq_to_x(freq);
                                painter.line_segment(
                                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                                    sub_grid_stroke,
                                );
                            }

                            let db_to_y = |db: f32| -> f32 {
                                let y_val = 1.0 + (db / 12.0) * 1.0;
                                let baseline_y = rect.bottom() - 20.0;
                                let max_height = rect.height() - 30.0;
                                baseline_y - y_val * (max_height * 0.35)
                            };

                            let db_levels = [ -12.0, 0.0, 12.0];

                            for &db in &db_levels {
                                let y = db_to_y(db);

                                let stroke = if db == 0.0 { 
                                    egui::Stroke::new(1.0, egui::Color32::from_gray(60))
                                } else { 
                                    egui::Stroke::new(1.0, egui::Color32::from_gray(30))
                                };

                                painter.line_segment(
                                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)], 
                                    stroke,
                                );

                                let label = if db > 0.0 {
                                    format!("+{} dB", db)
                                } else {
                                    format!("{} dB", db)
                                };
                                
                                painter.text(
                                    egui::pos2(rect.left() + 4.0, y - 4.0),
                                    egui::Align2::LEFT_BOTTOM,
                                    label,
                                    font_id.clone(),
                                    text_color
                                );
                            }

                            let cutoff = params_clone.cutoff.value();
                            let res = params_clone.resonance.value() / 100.0;
                            let f_type = params_clone.filter_type.value();

                            let slope_24 = params_clone.slope.value();
                            let mix = params_clone.mix.value() / 100.0;
                            let out_level = params_clone.out_level.value();
                            let drive = params_clone.drive.value();

                            let steepness = if slope_24 { 30.0 } else { 15.0 };

                            let min_f = 20.0f32.log10();
                            let max_f = 20000.0f32.log10();
                            let cut_f = cutoff.log10();
                            let norm_x = ((cut_f - min_f) / (max_f - min_f)).clamp(0.0, 1.0);

                            let mut points = vec![];
                            let steps = 100;

                            for i in 0..=steps {
                                let x_t = i as f32 / steps as f32;

                                let filter_y = match f_type {
                                    FilterType::Lowpass => {
                                        let dist = (x_t - norm_x) * steepness; 
                                        let smooth_roll_off = 1.0 / (1.0 + dist.exp());
                                        let peak = res * (-((x_t - norm_x) / 0.05).powi(2)).exp();
                                        smooth_roll_off + peak
                                    },
                                    FilterType::Highpass => {
                                        let dist = (norm_x - x_t) * steepness;
                                        let smooth_roll_off = 1.0 / (1.0 + dist.exp());
                                        let peak = res * (-((x_t - norm_x) / 0.05).powi(2)).exp();
                                        smooth_roll_off + peak
                                    },
                                    FilterType::Bandpass => {
                                        (0.5 + res * 1.5) * (-((x_t - norm_x) / 0.05).powi(2)).exp()
                                    },
                                    FilterType::Notch => {
                                        1.0 - (-((x_t - norm_x) / 0.05).powi(2)).exp()
                                    },
                                };

                                let dry_y = 1.0;
                                let mut y_val = dry_y + (filter_y - dry_y) * mix;

                                let drive_lift = (drive - 1.0).max(0.0) * 0.03;
                                y_val += drive_lift;

                                let out_lift = (out_level / 12.0) * 0.2;
                                y_val += out_lift;

                                let px = rect.left() + x_t * rect.width();
                                let baseline_y = rect.bottom() - 20.0;
                                let max_height = rect.height() - 30.0;
                                let ceiling_y = rect.top() + 2.0;

                                let py = (baseline_y - y_val * (max_height * 0.35)).max(ceiling_y);

                                points.push(egui::pos2(px, py));
                            }

                            let stroke = egui::Stroke::new(2.0, ACCENT_RED);
                            painter.add(egui::Shape::line(points, stroke));

                            let y_bottom = rect.bottom() - 4.0;

                            painter.text(
                                egui::pos2(freq_to_x(20.0) + 4.0, y_bottom),
                                egui::Align2::LEFT_BOTTOM,
                                "20",
                                font_id.clone(),
                                text_color
                            );

                            painter.text(
                                egui::pos2(freq_to_x(100.0), y_bottom), 
                                egui::Align2::CENTER_BOTTOM, 
                                "100", 
                                font_id.clone(), 
                                text_color
                            );
                            painter.text(
                                egui::pos2(freq_to_x(1000.0), y_bottom), 
                                egui::Align2::CENTER_BOTTOM, 
                                "1k", 
                                font_id.clone(), 
                                text_color
                            );
                            painter.text(
                                egui::pos2(freq_to_x(10000.0), y_bottom), 
                                egui::Align2::CENTER_BOTTOM, 
                                "10k", 
                                font_id.clone(), 
                                text_color
                            );
                        });
                    });

                    // 02 // FILTER TYPE
                    bento_frame().show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.set_width(ui.available_width());
                            section_header(ui, "02 // FILTER TYPE");
                            
                            let current_type = params_clone.filter_type.value();

                            let filter_btn = |ui: &mut egui::Ui, label: &str, f_type: FilterType| {
                                if bento_button(ui, label, current_type == f_type, None) {
                                    param_setter.begin_set_parameter(&params_clone.filter_type);
                                    param_setter.set_parameter(&params_clone.filter_type, f_type);
                                    param_setter.end_set_parameter(&params_clone.filter_type);
                                }
                            };
                            
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    filter_btn(ui, "[ LPF ] LOWPASS ", FilterType::Lowpass);
                                    ui.add_space(4.0);
                                    filter_btn(ui, "[ BPF ] BANDPASS", FilterType::Bandpass);
                                });
                                ui.vertical(|ui| {
                                    filter_btn(ui, "[ HPF ] HIGHPASS", FilterType::Highpass);
                                    ui.add_space(4.0);
                                    filter_btn(ui, "[ NTC ]  NOTCH  ", FilterType::Notch);
                                });
                            });
                        });
                    });

                    // 03 // CORE CONTROLS
                    bento_frame().show(ui, |ui| {
                        ui.vertical(|ui| { 
                            ui.set_width(ui.available_width());
                            section_header(ui, "03 // CORE CONTROLS");
                            bento_slider(ui, "CUTOFF FREQ:", &params_clone.cutoff, param_setter, 200.0);
                            bento_slider(ui, "RESONANCE:", &params_clone.resonance, param_setter, 200.0);

                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("SLOPE:").size(10.0).color(TEXT_DARK));
                                let is_24 = params_clone.slope.value();

                                let slope_btn = |ui: &mut egui::Ui, text: &str, target_val: bool| {
                                    if bento_button(ui, text, is_24 == target_val, None) {
                                        param_setter.begin_set_parameter(&params_clone.slope);
                                        param_setter.set_parameter(&params_clone.slope, target_val);
                                        param_setter.end_set_parameter(&params_clone.slope);
                                    }
                                };

                                // 3. Выводим сами кнопки
                                slope_btn(ui, "[ 12 dB ]", false);
                                slope_btn(ui, "[ 24 dB ]", true);
                            });
                        });
                    });

                    // 04 // SATURATION & OUTPUT
                    bento_frame().show(ui, |ui| {
                        ui.vertical(|ui| { 
                            ui.set_width(ui.available_width());
                            ui.set_height(ui.available_height());
                            section_header(ui, "04 // SATURATION & OUTPUT");

                            bento_slider(ui, "DRIVE AMOUNT:", &params_clone.drive, param_setter, 200.0);
                            bento_slider(ui, "DRY / WET MIX:", &params_clone.mix, param_setter, 200.0);
                            bento_slider(ui, "OUTPUT LEVEL:", &params_clone.out_level, param_setter, 200.0);
                        });
                    });
            });
        });
    })
}