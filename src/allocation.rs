// Copyright (c) 2024 Jacob R. Green
// All rights reserved.

use crate::macros::*;
use crate::*;
use vma_sys::*;

vma_handle!(Allocation, VmaAllocation);

vma_struct!(AllocationInfo, VmaAllocationInfo);

vma_struct!(AllocationCreateInfo, VmaAllocationCreateInfo);

impl AllocationCreateInfo {
    pub fn with_usage(mut self, usage: MemoryUsage) -> Self {
        self.inner.usage = usage.as_raw();
        self
    }

    pub fn with_required_flags(mut self, flags: vk::MemoryPropertyFlags) -> Self {
        self.inner.requiredFlags = flags.bits();
        self
    }

    pub fn with_preferred_flags(mut self, flags: vk::MemoryPropertyFlags) -> Self {
        self.inner.preferredFlags = flags.bits();
        self
    }

    pub fn with_creation_flags(mut self, flags: AllocationCreateFlags) -> Self {
        self.inner.flags = flags.bits();
        self
    }

    pub fn with_memory_type_bits(mut self, bits: u32) -> Self {
        self.inner.memoryTypeBits = bits;
        self
    }

    //     pub fn with_pool(mut self, pool: Pool) -> Self {
    //         self.inner.pool = pool.as_raw();
    //         self
    //     }
    //
    //     pub fn with_user_data(mut self, user_data: *mut std::ffi::c_void) -> Self {
    //         self.inner.pUserData = user_data;
    //         self
    //     }
}
