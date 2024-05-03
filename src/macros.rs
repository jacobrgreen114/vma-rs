// Copyright (c) 2024 Jacob R. Green
// All rights reserved.

macro_rules! vma_handle {
    ($name:tt, $ty:tt) => {
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name {
            handle: $ty,
        }

        impl $name {
            pub const fn from_raw(handle: $ty) -> Self {
                Self { handle }
            }

            pub const fn as_raw(&self) -> $ty {
                self.handle
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($name))
                    .field("handle", &self.handle)
                    .finish()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }

        unsafe impl Sync for $name {}
        unsafe impl Send for $name {}

        assert_eq_size!($name, $ty);
    };
}

macro_rules! vma_struct {
    ($name:tt, $ty:tt) => {
        pub struct $name {
            inner: $ty,
        }

        impl $name {
            pub const fn new() -> Self {
                Self {
                    inner: unsafe { std::mem::zeroed() },
                }
            }

            pub const fn from_raw(inner: $ty) -> Self {
                Self { inner }
            }

            pub const fn as_raw(&self) -> &$ty {
                &self.inner
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.inner.fmt(f)
            }
        }

        assert_eq_size!($name, $ty);
    };
}

pub(crate) use vma_handle;
pub(crate) use vma_struct;
