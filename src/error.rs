use std::mem::MaybeUninit;
use std::os::raw::c_char;

#[repr(C)]
pub struct ErrorCommonHeader {
    pub typ: u8,
    pub jump_flag: u8,
    pub unknown: [u8; 3],
    pub context_flag: u8,
    pub result_flag: u8,
    pub context_flag_2: u8,
}

#[repr(C, packed)]
pub struct ErrorApplicationArg {
    pub hdr: ErrorCommonHeader,
    pub error_number: u32,
    pub language_code: u64,
    pub dialog_message: [c_char; 0x800],
    pub fullscreen_message: [c_char; 0x800],
}

#[repr(C)]
pub struct ErrorApplicationConfig {
    pub arg: ErrorApplicationArg,
}

impl Default for ErrorApplicationConfig {
    fn default() -> Self {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

impl ErrorApplicationConfig {
    pub fn new(dialog_message: &str, fullscreen_message: Option<&str>) -> Option<Self> {
        let mut cfg = Self::default();
        cfg.arg.hdr.typ = 2;
        cfg.arg.hdr.jump_flag = 1;
        cfg.arg
            .dialog_message
            .get_mut(..dialog_message.len())?
            .copy_from_slice(dialog_message.as_bytes());
        if let Some(fullscreen_message) = fullscreen_message {
            cfg.arg
                .fullscreen_message
                .get_mut(..fullscreen_message.len())?
                .copy_from_slice(fullscreen_message.as_bytes());
        }
        // TODO: language code
        Some(cfg)
    }

    pub fn show(&self) -> u32 {
        extern "C" {
            fn errorApplicationShow(c: *const ErrorApplicationConfig) -> u32;
        }
        unsafe { errorApplicationShow(self as *const _) }
    }
}
