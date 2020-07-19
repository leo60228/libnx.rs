use crate::raw_fb::{Buffering, Frame, Framebuffer, NWindow, PixelFormat};
use crate::Result;
use lru_time_cache::LruCache;
use once_cell::sync::Lazy;
use rusttype::{point, Font, GlyphId, PositionedGlyph, Scale};
use std::collections::VecDeque;

const FONT_DATA: &[u8] = include_bytes!("../assets/Hack.ttf");
static FONT: Lazy<Font<'static>> = Lazy::new(|| Font::try_from_bytes(FONT_DATA).unwrap());
const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const FONT_SIZE: u32 = 20;
const SCALE: Scale = Scale {
    x: FONT_SIZE as f32,
    y: FONT_SIZE as f32,
};
static CHAR_WIDTH: Lazy<f32> = Lazy::new(|| {
    // this will definitely work always.
    FONT.glyph('a').scaled(SCALE).h_metrics().advance_width
});
static CHAR_PX: Lazy<usize> = Lazy::new(|| CHAR_WIDTH.ceil() as usize);
static CHARS: Lazy<usize> = Lazy::new(|| ((WIDTH as f32) / *CHAR_WIDTH) as usize);
const GLYPH_CACHE_SIZE: usize = 64; // randomly picked

type GlyphCache = LruCache<GlyphId, Vec<u8>>;

fn draw_text(frame: &mut Frame, glyph_cache: &mut GlyphCache, text: &str, x: i32, y: i32) {
    let v_metrics = FONT.v_metrics(SCALE);
    let coords = point(x as f32, y as f32 + v_metrics.ascent);

    for glyph in FONT.layout(text, SCALE, coords) {
        if let Some(bb) = glyph.pixel_bounding_box() {
            let tex = glyph_cache
                .entry(glyph.id())
                .or_insert_with(|| draw_glyph(&glyph));
            for (i, byte) in tex.iter().enumerate() {
                let y = (i / *CHAR_PX) as i32 + bb.min.y;
                let x = (i % *CHAR_PX) as i32 + bb.min.x;
                if y >= (HEIGHT as i32) {
                    break;
                }
                let pixel = frame.pixel_mut(x as _, y as _);
                pixel[0] = *byte;
                pixel[1] = *byte;
                pixel[2] = *byte;
                pixel[3] = 255;
            }
        }
    }
}

fn draw_glyph(glyph: &PositionedGlyph) -> Vec<u8> {
    let mut vec = vec![0; *CHAR_PX * FONT_SIZE as usize];
    glyph.draw(|x, y, v| {
        let x = x as usize;
        let y = y as usize;
        let v = (v * 255.0) as u8;
        let i = y * *CHAR_PX + x;
        vec[i] = v;
    });
    vec
}

pub struct Console<'a> {
    fb: Framebuffer<'a>,
    lines: VecDeque<String>,
    changed: Vec<usize>,
    redraw: bool,
    glyph_cache: GlyphCache,
    line_count: usize,
}

impl<'a> Console<'a> {
    pub fn new(win: &'a mut NWindow<'_>) -> Result<Self> {
        let mut fb =
            Framebuffer::new(win, WIDTH, HEIGHT, PixelFormat::Rgba8888, Buffering::Double)?;
        fb.make_linear()?;
        let v_metrics = FONT.v_metrics(SCALE);
        let line_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
        let line_count = ((HEIGHT as f32) / line_height).floor() as usize;
        let lines = VecDeque::with_capacity(line_count);
        let changed = Vec::with_capacity(line_count);
        let redraw = false;
        let glyph_cache = GlyphCache::with_capacity(GLYPH_CACHE_SIZE);
        Ok(Self {
            fb,
            lines,
            changed,
            redraw,
            glyph_cache,
            line_count,
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
            draw_text(&mut frame, &mut self.glyph_cache, &line, 0, y);
        }
        self.redraw = false;
        self.changed.clear();
    }

    fn push_wrapped_line(&mut self, line: &str) {
        debug_assert!(line.len() <= *CHARS, "{} > {}", line.len(), *CHARS);
        if self.lines.len() == self.line_count {
            self.lines.pop_front();
            self.redraw = true;
        } else if !self.redraw {
            self.changed.push(self.lines.len());
        }
        self.lines.push_back(line.to_string());
        debug_assert!(self.lines.len() <= self.line_count);
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
