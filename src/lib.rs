// Copyright (c) 2024 Jacob R. Green
// All rights reserved.

#[macro_use]
extern crate static_assertions;

mod macros;

mod enums;
pub use enums::*;

mod allocation;
pub use allocation::*;

mod allocator;
pub use allocator::*;

use vma_sys::*;
use vulkan as vk;

pub use vma_sys as sys;

use std::mem::transmute;
