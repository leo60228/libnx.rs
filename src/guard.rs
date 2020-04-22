pub trait IntoLibnx<T> {
    fn into_libnx(self) -> T;
}

#[macro_export]
macro_rules! service_guard {
    ($name:ident) => {
        service_guard!($name, ())
    };
    ($name:ident, ($($arg_names:ident: $arg_types:ty as $ffi_types:ty),*)) => {
        paste::item! {
            pub struct $name(());

            impl $name {
                pub fn new($($arg_names: $arg_types),*) -> $crate::Result<Self> {
                    use $crate::IntoResult;

                    extern "C" {
                        fn [<$name:lower Initialize>]($($arg_names: $ffi_types),*) -> u32;
                    }

                    unsafe {
                        [<$name:lower Initialize>]($(IntoLibnx::into_libnx($arg_names)),*).into_result()?;
                    }

                    Ok(Self(()))
                }
            }

            impl Drop for $name {
                fn drop(&mut self) {
                    extern "C" {
                        fn [<$name:lower Exit>]();
                    }

                    unsafe {
                        [<$name:lower Exit>]();
                    }
                }
            }
        }
    };
}
