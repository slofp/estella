use serenity::model::colour::Color;

const SUCCESS: u32 = 0x00B06B;
const FAILED: u32 = 0xFF4B00;
const WARNING: u32 = 0xF2E700;
const NORMAL: u32 = 0x1971FF;
const CRITICAL: u32 = 0x990099;
//const DANGER:   u32 = 0xF6AA00;

pub fn success_color() -> Color {
	Color::new(SUCCESS)
}

pub fn failed_color() -> Color {
	Color::new(FAILED)
}

pub fn warning_color() -> Color {
	Color::new(WARNING)
}

pub fn normal_color() -> Color {
	Color::new(NORMAL)
}

pub fn critical_color() -> Color {
	Color::new(CRITICAL)
}

//pub fn danger_color() -> Color { Color::new(DANGER) }
