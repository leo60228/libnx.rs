pub mod error;

use backtrace::Backtrace;
use error::ErrorApplicationConfig;
use std::os::raw::c_char;
use std::panic::PanicInfo;
use std::ptr;
use std::thread;
use std::time::Duration;

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
        thread::sleep(Duration::from_millis(3000));
    }
    unsafe {
        consoleExit(ptr::null_mut());
    }
    std::process::exit(0);
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
    std::panic::set_hook(Box::new(panic_hook));
    panic!("test");
    unsafe { consoleUpdate(ptr::null_mut()) }
    thread::sleep(Duration::from_millis(3000));
    unsafe { consoleExit(ptr::null_mut()) }
}
