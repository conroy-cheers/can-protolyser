use crate::egui;
use crate::gui::util::contrasting_text;

pub(crate) fn color_chip(ui: &mut egui::Ui, color: egui::Color32) {
    let size = ui.spacing().interact_size;
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::focusable_noninteractive());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let rect = rect.expand(visuals.expansion);

        egui::color_picker::show_color_at(ui.painter(), color, rect);

        let rounding = visuals.rounding.at_most(2.0);
        ui.painter()
            .rect_stroke(rect, rounding, (2.0, visuals.bg_fill)); // fill is intentional, because default style has no border
    }
}

pub(crate) fn colored_label(ui: &mut egui::Ui, color: egui::Color32, text: &String) {
    let font_id = egui::TextStyle::Body.resolve(ui.style());
    let size = egui::Vec2 {
        x: ui
            .painter()
            .layout(text.to_string(), font_id, egui::Color32::DEBUG_COLOR, 1e5)
            .size()
            .x
            * 1.1
            + 10.0,
        y: ui.text_style_height(&egui::TextStyle::Body) + 5.0,
    };
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::focusable_noninteractive());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let rect = rect.expand(visuals.expansion);

        egui::color_picker::show_color_at(ui.painter(), color, rect);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::default(),
            contrasting_text(&color),
        );

        let rounding = visuals.rounding.at_most(2.0);
        ui.painter()
            .rect_stroke(rect, rounding, (2.0, visuals.bg_fill)); // fill is intentional, because default style has no border
    }
}
