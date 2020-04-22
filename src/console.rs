use std::ffi::c_void;
use std::marker::{PhantomData, PhantomPinned};
use std::pin::Pin;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};

static INITIALIZED: AtomicBool = AtomicBool::new(false);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(C)]
pub struct ConsoleFont<'a> {
    gfx: *const c_void,
    ascii_offset: u16,
    num_chars: u16,
    tile_width: u16,
    tile_height: u16,
    _phantom: PhantomData<&'a [u8]>,
}

impl Default for ConsoleFont<'_> {
    fn default() -> Self {
        Self {
            gfx: ptr::null(),
            ascii_offset: 0,
            num_chars: 0,
            tile_width: 0,
            tile_height: 0,
            _phantom: PhantomData,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ConsoleRenderer {
    pub init: unsafe extern "C" fn(con: *mut PrintConsole) -> bool,
    pub deinit: unsafe extern "C" fn(con: *mut PrintConsole),
    pub draw_char: unsafe extern "C" fn(con: *mut PrintConsole, x: i32, y: i32, c: i32),
    pub scroll_window: unsafe extern "C" fn(con: *mut PrintConsole),
    pub flush_and_swap: unsafe extern "C" fn(con: *mut PrintConsole),
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PrintConsole<'a> {
    pub font: ConsoleFont<'a>,
    pub renderer: Option<&'a ConsoleRenderer>,
    pub cursor_x: i32,
    pub cursor_y: i32,
    prev_cursor_x: i32,
    prev_cursor_y: i32,
    pub console_width: i32,
    pub console_height: i32,
    pub window_x: i32,
    pub window_y: i32,
    pub window_width: i32,
    pub window_height: i32,
    pub tab_size: i32,
    pub fg: i32,
    pub bg: i32,
    pub flags: i32,
    pub initialized: bool,
}

#[derive(Copy, Clone)]
#[repr(transparent)]
struct PinnedConsole<'a>(pub PrintConsole<'a>, pub PhantomPinned);

extern "C" {
    fn consoleGetDefault() -> *mut PrintConsole<'static>;
    fn consoleSelect<'a>(console: *mut PrintConsole<'a>) -> *mut PrintConsole<'a>;
    fn consoleInit<'a>(console: *mut PrintConsole<'a>) -> *mut PrintConsole<'a>;
    fn consoleExit<'a>(console: *mut PrintConsole<'a>);
    fn consoleUpdate<'a>(console: *mut PrintConsole<'a>);
}

pub struct Console<'a>(Option<Pin<Box<PinnedConsole<'a>>>>);

impl Console<'static> {
    pub fn new() -> Self {
        if !INITIALIZED.swap(true, Ordering::SeqCst) {
            extern "C" fn exit() {
                unsafe {
                    consoleExit(ptr::null_mut());
                }
            }

            unsafe {
                libc::atexit(exit);
            }
        }

        let mut console = Self(None);
        console.init();
        console
    }
}

impl<'a> Console<'a> {
    pub fn new_with(options: PrintConsole<'a>) -> Self {
        Self(Some(Box::pin(PinnedConsole(options, PhantomPinned))))
    }

    pub fn as_raw(&mut self) -> *mut PrintConsole<'a> {
        if let Some(ref mut pin) = &mut self.0 {
            unsafe { Pin::get_unchecked_mut(Pin::as_mut(pin)) as *mut _ as *mut _ }
        } else {
            unsafe { consoleGetDefault() as *mut c_void as *mut PrintConsole<'a> }
        }
    }

    pub fn init(&mut self) {
        unsafe {
            consoleInit(self.as_raw());
        }
    }

    pub fn update(&mut self) {
        unsafe {
            consoleUpdate(self.as_raw());
        }
    }

    pub fn select(&mut self) {
        unsafe {
            consoleSelect(self.as_raw());
        }
    }
}

impl Drop for Console<'_> {
    fn drop(&mut self) {
        unsafe {
            if self.0.is_some() {
                consoleExit(Console::new().as_raw()); // switch to a 'static console and get rid of it (to prevent current console being freed)
                consoleExit(self.as_raw());
            }
        }
    }
}

impl Default for PrintConsole<'static> {
    fn default() -> Self {
        unsafe { ptr::read(consoleGetDefault()) }
    }
}
