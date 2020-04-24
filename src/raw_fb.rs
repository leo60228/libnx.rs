use crate::result::*;
use crate::types::{Event, RawMutex, Service};
use std::convert::TryInto;
use std::ffi::c_void;
use std::marker::{PhantomData, PhantomPinned};
use std::pin::Pin;
use std::ptr;
use std::slice;

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Transform {
    None = 0,
    FlipH = 0x01,
    FlipV = 0x02,
    Rot90 = 0x04,
    Rot180 = 0x03,
    Rot270 = 0x07,
}

impl Default for Transform {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Binder {
    pub created: bool,
    pub initialized: bool,
    pub id: i32,
    pub _dummy: usize,
    pub relay: *mut Service,
}

impl Default for Binder {
    fn default() -> Self {
        Self {
            created: false,
            initialized: false,
            id: 0,
            _dummy: 0,
            relay: ptr::null_mut(),
        }
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct BqRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct WindowData {
    pub magic: u32,
    pub bq: Binder,
    pub event: Event,
    pub mutex: RawMutex,
    pub slots_configured: u64,
    pub slots_requested: u64,
    pub cur_slot: i32,
    pub width: u32,
    pub height: u32,
    pub format: u32,
    pub usage: u32,
    pub crop: BqRect,
    pub scaling_mode: u32,
    pub transform: u32,
    pub sticky_transform: u32,
    pub default_width: u32,
    pub default_height: u32,
    pub swap_interval: u32,
    pub is_connected: bool,
    pub producer_controlled_by_app: bool,
    pub consumer_running_behind: bool,
    pub _pin: PhantomData<PhantomPinned>,
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NWindow<'a>(Option<Pin<&'a mut WindowData>>);

extern "C" {
    fn nwindowGetDefault() -> *mut WindowData;
}

impl<'a> NWindow<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// # Safety
    /// This must be a valid (i.e. have had nwindowCreate called on it) NWindow.
    pub unsafe fn from_data(data: Pin<&'a mut WindowData>) -> Self {
        Self(Some(data))
    }

    pub fn as_mut_ptr(&mut self) -> *mut WindowData {
        if let Some(data) = &mut self.0 {
            let pin = Pin::as_mut(data);

            unsafe { Pin::get_unchecked_mut(pin) as *mut _ }
        } else {
            unsafe { nwindowGetDefault() }
        }
    }

    pub fn as_ptr(&self) -> *const WindowData {
        if let Some(data) = &self.0 {
            &**data as *const _
        } else {
            unsafe { nwindowGetDefault() as *const _ }
        }
    }

    pub fn as_pin(&mut self) -> Option<Pin<&mut WindowData>> {
        self.0.as_mut().map(Pin::as_mut)
    }

    pub fn as_ref(&self) -> &WindowData {
        unsafe { &*self.as_ptr() }
    }

    pub fn get_dimensions(&mut self) -> Result<(u32, u32)> {
        extern "C" {
            fn nwindowGetDimensions(
                nw: *mut WindowData,
                out_width: *mut u32,
                out_height: *mut u32,
            ) -> u32;
        }

        let mut width = 0;
        let mut height = 0;

        unsafe {
            nwindowGetDimensions(
                self.as_mut_ptr(),
                &mut width as *mut _,
                &mut height as *mut _,
            )
            .into_result()?;
        }

        Ok((width, height))
    }

    pub fn set_dimensions(&mut self, w: u32, h: u32) -> Result<()> {
        extern "C" {
            fn nwindowSetDimensions(nw: *mut WindowData, width: u32, height: u32) -> u32;
        }

        unsafe { nwindowSetDimensions(self.as_mut_ptr(), w, h).into_result() }
    }

    pub fn set_crop(&mut self, left: i32, top: i32, right: i32, bottom: i32) -> Result<()> {
        extern "C" {
            fn nwindowSetCrop(
                nw: *mut WindowData,
                left: i32,
                top: i32,
                right: i32,
                bottom: i32,
            ) -> u32;
        }

        unsafe { nwindowSetCrop(self.as_mut_ptr(), left, top, right, bottom).into_result() }
    }

    pub fn set_transform(&mut self, transform: Transform) -> Result<()> {
        extern "C" {
            fn nwindowSetTransform(nw: *mut WindowData, transform: u32) -> u32;
        }

        unsafe { nwindowSetTransform(self.as_mut_ptr(), transform as u32).into_result() }
    }

    pub fn set_swap_interval(&mut self, swap_interval: u32) -> Result<()> {
        extern "C" {
            fn nwindowSetSwapInterval(nw: *mut WindowData, swap_interval: u32) -> u32;
        }

        unsafe { nwindowSetSwapInterval(self.as_mut_ptr(), swap_interval).into_result() }
    }

    pub fn is_consumer_running_behind(&mut self) -> bool {
        self.as_ref().consumer_running_behind
    }

    // TODO: buffers
}

impl Drop for NWindow<'_> {
    fn drop(&mut self) {
        if self.0.is_some() {
            extern "C" {
                fn nwindowClose(nw: *mut WindowData);
            }

            unsafe {
                nwindowClose(self.as_mut_ptr());
            }
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct NvKind(i32);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct NvMap {
    pub handle: u32,
    pub id: u32,
    pub size: u32,
    pub cpu_addr: *mut c_void,
    pub kind: NvKind,
    pub has_init: bool,
    pub is_cpu_cacheable: bool,
}

impl Default for NvMap {
    fn default() -> Self {
        Self {
            handle: 0,
            id: 0,
            size: 0,
            cpu_addr: ptr::null_mut(),
            kind: Default::default(),
            has_init: false,
            is_cpu_cacheable: false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct RawFramebuffer {
    win: *mut WindowData,
    map: NvMap,
    buf: *mut c_void,
    buf_linear: *mut c_void,
    stride: u32,
    width_aligned: u32,
    height_aligned: u32,
    num_fbs: u32,
    fb_size: u32,
    has_init: bool,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Framebuffer<'a> {
    inner: RawFramebuffer,
    width: u32,
    height: u32,
    format: PixelFormat,
    _phantom: PhantomData<&'a mut ()>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum PixelFormat {
    Rgba8888 = 1,
    Rgbx8888 = 2,
    Rgb565 = 4,
    Bgra8888 = 5,
    Rgba4444 = 7,
}

impl PixelFormat {
    pub fn bytes_per_pixel(self) -> u8 {
        match self {
            Self::Rgba8888 => 4,
            Self::Rgbx8888 => 4,
            Self::Rgb565 => 2,
            Self::Bgra8888 => 4,
            Self::Rgba4444 => 2,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum Buffering {
    Single = 1,
    Double = 2,
    Triple = 3,
}

impl<'a> Framebuffer<'a> {
    pub fn new(
        win: &'a mut NWindow<'_>,
        width: u32,
        height: u32,
        format: PixelFormat,
        buffering: Buffering,
    ) -> Result<Self> {
        let mut fb = RawFramebuffer {
            win: ptr::null_mut(),
            map: Default::default(),
            buf: ptr::null_mut(),
            buf_linear: ptr::null_mut(),
            stride: 0,
            width_aligned: 0,
            height_aligned: 0,
            num_fbs: 0,
            fb_size: 0,
            has_init: false,
        };

        extern "C" {
            fn framebufferCreate(
                fb: *mut RawFramebuffer,
                win: *mut WindowData,
                width: u32,
                height: u32,
                format: u32,
                num_fbs: u32,
            ) -> u32;
        }

        unsafe {
            framebufferCreate(
                &mut fb as *mut _,
                win.as_mut_ptr(),
                width,
                height,
                format as u32,
                buffering as u32,
            )
            .into_result()?;
        }

        let buf_len = (fb.fb_size * fb.num_fbs + 0xFFF) & !0xFFF;
        unsafe {
            ptr::write_bytes(fb.buf, 0, buf_len as usize);
        }

        Ok(Self {
            inner: fb,
            width,
            height,
            format,
            _phantom: PhantomData,
        })
    }

    pub fn make_linear(&mut self) -> Result<()> {
        extern "C" {
            fn framebufferMakeLinear(fb: *mut RawFramebuffer) -> u32;
        }

        unsafe { framebufferMakeLinear(&mut self.inner as *mut _).into_result() }
    }

    pub fn start_frame<'b>(&'b mut self) -> Frame<'b, 'a> {
        extern "C" {
            fn framebufferBegin(fb: *mut RawFramebuffer, out_stride: *mut u32) -> *mut c_void;
        }

        let mut stride = 0;

        let data = unsafe { framebufferBegin(&mut self.inner as *mut _, &mut stride as *mut _) }
            as *mut u8;

        assert!(!data.is_null());

        Frame {
            data,
            stride,
            fb: self,
        }
    }
}

impl Drop for Framebuffer<'_> {
    fn drop(&mut self) {
        extern "C" {
            fn framebufferClose(fb: *mut RawFramebuffer);
        }

        unsafe {
            framebufferClose(&mut self.inner as *mut _);
        }
    }
}

pub struct Frame<'a, 'b> {
    data: *mut u8,
    stride: u32,
    fb: &'a mut Framebuffer<'b>,
}

impl<'a> Frame<'a, '_> {
    pub fn as_raw(&self) -> (*mut u8, u32) {
        (self.data, self.stride)
    }

    pub fn row(&self, y: usize) -> &'a [u8] {
        assert!(y < self.fb.height.try_into().unwrap());
        let offset = y * (self.stride as usize);
        let ptr = self.data.wrapping_offset(offset.try_into().unwrap()) as *mut u8;
        let bpp: u32 = self.fb.format.bytes_per_pixel().into();
        let len = bpp * self.fb.width;
        unsafe { slice::from_raw_parts(ptr, len.try_into().unwrap()) }
    }

    pub fn row_mut(&mut self, y: usize) -> &'a mut [u8] {
        assert!(y < self.fb.height.try_into().unwrap());
        let offset = y * (self.stride as usize);
        let ptr = self.data.wrapping_offset(offset.try_into().unwrap()) as *mut u8;
        let bpp: u32 = self.fb.format.bytes_per_pixel().into();
        let len = bpp * self.fb.width;
        unsafe { slice::from_raw_parts_mut(ptr, len.try_into().unwrap()) }
    }

    pub fn pixel(&self, x: usize, y: usize) -> &'a [u8] {
        let bpp: usize = self.fb.format.bytes_per_pixel() as _;
        &self.row(y)[x * bpp..][..bpp]
    }

    pub fn pixel_mut(&mut self, x: usize, y: usize) -> &'a mut [u8] {
        let bpp: usize = self.fb.format.bytes_per_pixel() as _;
        &mut self.row_mut(y)[x * bpp..][..bpp]
    }

    pub fn slice(&self) -> &'a [u8] {
        let len = self.fb.height * self.stride;
        unsafe { slice::from_raw_parts(self.data, len.try_into().unwrap()) }
    }

    pub fn slice_mut(&mut self) -> &'a mut [u8] {
        let len = self.fb.height * self.stride;
        unsafe { slice::from_raw_parts_mut(self.data, len.try_into().unwrap()) }
    }

    pub fn stride(&self) -> u32 {
        self.stride
    }

    pub fn clear(&mut self) {
        for x in self.slice_mut() {
            *x = 0;
        }
    }
}

impl Drop for Frame<'_, '_> {
    fn drop(&mut self) {
        extern "C" {
            fn framebufferEnd(fb: *mut RawFramebuffer);
        }

        unsafe {
            framebufferEnd(&mut self.fb.inner as *mut _);
        }
    }
}
