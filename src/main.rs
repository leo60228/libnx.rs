#[macro_use]
extern crate cpp;

pub mod console;
pub mod error;
mod guard;
pub mod raw_fb;
pub mod result;
pub mod rs_console;
pub mod types;

pub use result::*;

use backtrace::Backtrace;
use console::Console;
use error::ErrorApplicationConfig;
use raw_fb::*;
use std::os::raw::{c_char, c_int};
use std::panic::{self, PanicInfo};
use std::thread;

fn wait_for_button() {
    extern "C" {
        fn hidScanInput();
        fn hidKeysDown(controller: c_int) -> u64;
        fn appletMainLoop() -> bool;
    }
    unsafe {
        while appletMainLoop() {
            hidScanInput();
            let down = hidKeysDown(10);
            if down != 0 {
                break;
            }
        }
    }
}

fn panic_hook(info: &PanicInfo) {
    let thread = thread::current();
    let name = thread.name().unwrap_or("<unnamed>");
    let short = format!("thread '{}' {}", name, info);
    let long = format!("{}\nstack backtrace:\n{:?}", short, Backtrace::new());
    if let Some(error) = ErrorApplicationConfig::new(&short, Some(&long)) {
        error.show();
    } else {
        println!("{}", long);
        let mut console = Console::new();
        console.update();
        wait_for_button();
    }
}

extern "C" {
    fn romfsMountSelf(name: *const c_char) -> u32;
}

#[allow(unreachable_code)]
fn main() -> Result<()> {
    assert_eq!(unsafe { romfsMountSelf(b"romfs" as *const _) }, 0);
    panic::set_hook(Box::new(panic_hook));

    let mut nwindow = NWindow::default();
    let mut fb = Framebuffer::new(
        &mut nwindow,
        1280,
        720,
        PixelFormat::Rgba8888,
        Buffering::Single,
    )?;
    fb.make_linear()?;
    let mut frame = fb.start_frame();
    rs_console::draw_text(&mut frame, "Hello, world!", 10, 10, 24.0);
    drop(frame);
    wait_for_button();

    Ok(())
}
