cpp! {{
    #include <switch.h>
}}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Handle(pub u32);

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Service {
    pub session: Handle,
    pub own_handle: u32,
    pub object_id: u32,
    pub pointer_buffer_size: u16,
}

impl Drop for Service {
    fn drop(&mut self) {
        let self_ptr = self as *mut Service;
        unsafe {
            cpp!([self_ptr as "Service *"] {
                serviceClose(self_ptr);
            });
        }
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Event {
    pub revent: Handle,
    pub wevent: Handle,
    pub autoclear: bool,
}

pub type RawMutex = libc::_LOCK_T;
