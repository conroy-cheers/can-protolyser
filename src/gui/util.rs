use eframe::egui::Color32;

pub fn ack_color(ack: bool) -> Color32 {
    if ack {
        Color32::GREEN
    } else {
        Color32::RED
    }
}

pub fn speed_color(speed: &String) -> Color32 {
    match speed.as_str() {
        "1M" => Color32::GREEN,
        "667k" => Color32::YELLOW,
        "500k" => Color32::RED,
        _ => Color32::WHITE,
    }
}
