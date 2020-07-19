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
use std::os::raw::c_int;
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

#[allow(unreachable_code)]
fn main() -> Result<()> {
    panic::set_hook(Box::new(panic_hook));

    let mut nwindow = NWindow::default();
    let mut console = rs_console::Console::new(&mut nwindow)?;
    console.append("Hello, world!");
    console.append("This\nhas a newline!");
    console.append(&("lots".to_string() + &" and lots".repeat(100) + " of text\nwith\nnewlines"));
    wait_for_button();
    console.append(&"line\n".repeat(100));
    console.append("Press any button to exit.");
    wait_for_button();

    Ok(())
}
