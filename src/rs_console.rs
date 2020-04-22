use crate::raw_fb::Frame;
use once_cell::sync::Lazy;
use rusttype::{point, Font, Scale};

const FONT_DATA: &[u8] = include_bytes!("../assets/Hack.ttf");
static FONT: Lazy<Font<'static>> = Lazy::new(|| Font::try_from_bytes(FONT_DATA).unwrap());

pub(crate) fn draw_text(frame: &mut Frame, text: &str, x: i32, y: i32, size: f32) {
    let scale = Scale { x: size, y: size };
    let v_metrics = FONT.v_metrics(scale);
    let coords = point(x as f32, y as f32 + v_metrics.ascent);

    for glyph in FONT.layout(text, scale, coords) {
        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                let x = x as i32 + bb.min.x;
                let y = y as i32 + bb.min.y;
                let c = (v * 255.0) as u8;
                let pixel = frame.pixel_mut(x as _, y as _);
                pixel[0] = c;
                pixel[1] = c;
                pixel[2] = c;
                pixel[3] = 255;
            });
        }
    }
}
