pub mod error;

use backtrace::Backtrace;
use error::ErrorApplicationConfig;
use std::os::raw::{c_char, c_int};
use std::panic::{self, PanicInfo};
use std::ptr;
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
        unsafe {
            consoleUpdate(ptr::null_mut());
        }
        wait_for_button();
    }
}

enum PrintConsole {}

extern "C" {
    fn consoleInit(console: *mut PrintConsole) -> *mut PrintConsole;
    fn consoleUpdate(console: *mut PrintConsole);
    fn consoleExit(console: *mut PrintConsole);
    fn romfsMountSelf(name: *const c_char) -> u32;
}

#[allow(unreachable_code)]
fn main() {
    unsafe { consoleInit(ptr::null_mut()) };
    assert_eq!(unsafe { romfsMountSelf(b"romfs" as *const _) }, 0);
    panic::set_hook(Box::new(panic_hook));

    let result = panic::catch_unwind(|| {
        println!("hello!");
    });
    assert!(dbg!(result).is_ok());

    let result = panic::catch_unwind(|| {
        panic!("oh no!");
    });
    assert!(dbg!(result).is_err());

    unsafe { consoleUpdate(ptr::null_mut()) }
    wait_for_button();
    unsafe { consoleExit(ptr::null_mut()) }
}
