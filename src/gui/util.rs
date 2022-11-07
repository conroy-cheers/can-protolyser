use crate::message::Speed;
use eframe::egui::Color32;

pub fn ack_color(ack: bool) -> Color32 {
    if ack {
        Color32::GREEN
    } else {
        Color32::RED
    }
}

pub fn speed_color(speed: &Speed) -> Color32 {
    match speed.as_str() {
        "1M" => Color32::GREEN,
        "667k" => Color32::YELLOW,
        "500k" => Color32::RED,
        _ => Color32::WHITE,
    }
}

fn color_is_light(color: &Color32) -> bool {
    let r = color.r() as f32;
    let g = color.g() as f32;
    let b = color.b() as f32;
    // Counting the perceptive luminance
    // human eye favors green color...
    let a = 1.0 - (0.299 * r + 0.587 * g + 0.114 * b) / 255.0;
    return a < 0.5;
}

pub fn contrasting_text(color: &Color32) -> Color32 {
    if color_is_light(color) {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}
