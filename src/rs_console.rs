use crate::raw_fb::{Buffering, Frame, Framebuffer, NWindow, PixelFormat};
use crate::Result;
use once_cell::sync::Lazy;
use rusttype::{point, Font, Scale};
use std::collections::VecDeque;

const FONT_DATA: &[u8] = include_bytes!("../assets/Hack.ttf");
static FONT: Lazy<Font<'static>> = Lazy::new(|| Font::try_from_bytes(FONT_DATA).unwrap());
const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const FONT_SIZE: u32 = 20;
const LINES: usize = (HEIGHT / FONT_SIZE) as usize;
const SCALE: Scale = Scale {
    x: FONT_SIZE as f32,
    y: FONT_SIZE as f32,
};
static CHAR_WIDTH: Lazy<f32> = Lazy::new(|| {
    // this will definitely work always.
    FONT.glyph('a').scaled(SCALE).h_metrics().advance_width
});
static CHARS: Lazy<usize> = Lazy::new(|| ((WIDTH as f32) / *CHAR_WIDTH) as usize);

fn draw_text(frame: &mut Frame, text: &str, x: i32, y: i32) {
    let v_metrics = FONT.v_metrics(SCALE);
    let coords = point(x as f32, y as f32 + v_metrics.ascent);

    for glyph in FONT.layout(text, SCALE, coords) {
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

pub struct Console<'a> {
    fb: Framebuffer<'a>,
    lines: VecDeque<String>,
    changed: Vec<usize>,
    redraw: bool,
}

impl<'a> Console<'a> {
    pub fn new(win: &'a mut NWindow<'_>) -> Result<Self> {
        let mut fb =
            Framebuffer::new(win, WIDTH, HEIGHT, PixelFormat::Rgba8888, Buffering::Double)?;
        fb.make_linear()?;
        let lines = VecDeque::with_capacity(LINES);
        let changed = Vec::with_capacity(LINES);
        let redraw = false;
        Ok(Self {
            fb,
            lines,
            changed,
            redraw,
        })
    }

    pub fn draw(&mut self) {
        let mut frame = self.fb.start_frame();
        if self.redraw {
            frame.clear();
        }
        let v_metrics = FONT.v_metrics(SCALE);
        let line_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
        for (i, line) in self.lines.iter().enumerate() {
            if !self.redraw && !self.changed.contains(&i) {
                continue;
            }
            let y = (i as i32) * (line_height as i32);
            draw_text(&mut frame, &line, 0, y);
        }
        self.redraw = false;
        self.changed.clear();
    }

    fn push_wrapped_line(&mut self, line: &str) {
        debug_assert!(line.len() <= *CHARS, "{} > {}", line.len(), *CHARS);
        if self.lines.len() == LINES {
            self.lines.pop_front();
            self.redraw = true;
        } else if !self.redraw {
            self.changed.push(self.lines.len());
        }
        self.lines.push_back(line.to_string());
        debug_assert!(self.lines.len() <= LINES);
    }

    fn push_one_line(&mut self, line: &str) {
        debug_assert!(!line.contains('\n'));
        let mut chars = line.char_indices().peekable();
        let mut last_idx = 0;
        loop {
            let _ = chars.by_ref().nth(*CHARS - 1);
            let end_idx = chars.peek();
            if let Some(&(end_idx, _)) = end_idx {
                let slice = &line[last_idx..end_idx];
                self.push_wrapped_line(slice);
                last_idx = end_idx;
            } else {
                let slice = &line[last_idx..];
                if slice.len() > 0 {
                    self.push_wrapped_line(slice);
                }
                break;
            }
        }
    }

    pub fn append(&mut self, text: &str) {
        for line in text.split('\n') {
            self.push_one_line(line);
        }
        self.draw();
    }
}
