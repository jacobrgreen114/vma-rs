// Copyright (c) 2024 Jacob R. Green
// All rights reserved.

use crate::macros::*;
use crate::*;
use std::ffi::c_void;
use std::ptr::NonNull;
use vma_sys::*;

vma_handle!(Allocator, VmaAllocator);

impl crate::allocator::Allocator {
    pub fn create(create_info: &AllocatorCreateInfo) -> Result<Self, ()> {
        let mut allocator = std::ptr::null_mut();

        let result = unsafe { vmaCreateAllocator(create_info.as_raw(), &mut allocator) };
        if result != vk::sys::VK_SUCCESS {
            return Err(());
        }

        Ok(Self::from_raw(allocator))
    }

    pub fn destroy(self) {
        unsafe { vmaDestroyAllocator(self.as_raw()) };
    }

    pub fn create_buffer(
        &self,
        buffer_create_info: &vk::BufferCreateInfo,
        allocation_create_info: &AllocationCreateInfo,
        allocation_info: Option<&mut AllocationInfo>,
    ) -> Result<(vk::Buffer, Allocation), ()> {
        let mut buffer = std::ptr::null_mut();
        let mut allocation = std::ptr::null_mut();

        let result = unsafe {
            vmaCreateBuffer(
                self.as_raw(),
                buffer_create_info.as_raw(),
                allocation_create_info.as_raw(),
                &mut buffer,
                &mut allocation,
                transmute(allocation_info),
            )
        };

        if result != vk::sys::VK_SUCCESS {
            return Err(());
        }

        Ok((
            vk::Buffer::from_raw(buffer),
            Allocation::from_raw(allocation),
        ))
    }

    pub fn destroy_buffer(&self, buffer: vk::Buffer, allocation: Allocation) {
        unsafe { vmaDestroyBuffer(self.as_raw(), buffer.as_raw(), allocation.as_raw()) };
    }

    pub fn create_image(
        &self,
        image_create_info: &vk::ImageCreateInfo,
        allocation_create_info: &AllocationCreateInfo,
        allocation_info: Option<&mut AllocationInfo>,
    ) -> Result<(vk::Image, Allocation), ()> {
        let mut image = std::ptr::null_mut();
        let mut allocation = std::ptr::null_mut();

        let result = unsafe {
            vmaCreateImage(
                self.as_raw(),
                image_create_info.as_raw(),
                allocation_create_info.as_raw(),
                &mut image,
                &mut allocation,
                transmute(allocation_info),
            )
        };

        if result != vk::sys::VK_SUCCESS {
            return Err(());
        }

        Ok((vk::Image::from_raw(image), Allocation::from_raw(allocation)))
    }

    pub fn destroy_image(&self, image: vk::Image, allocation: Allocation) {
        unsafe { vmaDestroyImage(self.as_raw(), image.as_raw(), allocation.as_raw()) };
    }

    pub fn map_memory<'a>(&self, allocation: Allocation) -> Result<NonNull<c_void>, ()> {
        let mut data = std::ptr::null_mut();
        let result = unsafe { vmaMapMemory(self.as_raw(), allocation.as_raw(), &mut data) };
        if result != vk::sys::VK_SUCCESS {
            return Err(());
        }
        Ok(NonNull::new(data).unwrap())
    }

    pub fn unmap_memory(&self, allocation: Allocation) {
        unsafe { vmaUnmapMemory(self.as_raw(), allocation.as_raw()) };
    }
}

vma_struct!(AllocatorCreateInfo, VmaAllocatorCreateInfo);

impl AllocatorCreateInfo {
    pub fn flags(&mut self, flags: AllocatorCreateFlags) -> &mut Self {
        self.inner.flags = flags.bits();
        self
    }

    pub fn with_physical_device(mut self, physical_device: vk::PhysicalDevice) -> Self {
        self.inner.physicalDevice = physical_device.as_raw();
        self
    }

    pub fn with_device(mut self, device: vk::Device) -> Self {
        self.inner.device = device.as_raw();
        self
    }

    pub fn preferred_large_heap_block_size(&mut self, size: u64) -> &mut Self {
        self.inner.preferredLargeHeapBlockSize = size;
        self
    }

    // pub fn allocation_callbacks(&mut self, callbacks: *const VmaAllocationCallbacks) -> &mut Self {
    //     self.inner.pAllocationCallbacks = callbacks;
    //     self
    // }
    //
    // pub fn device_memory_callbacks(
    //     &mut self,
    //     callbacks: *const VmaDeviceMemoryCallbacks,
    // ) -> &mut Self {
    //     self.inner.pDeviceMemoryCallbacks = callbacks;
    //     self
    // }
    //
    // pub fn heap_size_limit(&mut self, limit: *const VmaPoolSizeLimit) -> &mut Self {
    //     self.inner.pHeapSizeLimit = limit;
    //     self
    // }
    //
    // pub fn vulkan_functions(&mut self, functions: *const VmaVulkanFunctions) -> &mut Self {
    //     self.inner.pVulkanFunctions = functions;
    //     self
    // }

    pub fn with_instance(mut self, instance: vk::Instance) -> Self {
        self.inner.instance = instance.as_raw();
        self
    }

    pub fn vulkan_api_version(mut self, version: vk::ApiVersion) -> Self {
        self.inner.vulkanApiVersion = version.0;
        self
    }

    // pub fn type_external_memory_handle_types(
    //     &mut self,
    //     types: *const vk::ExternalMemoryHandleTypeFlags,
    // ) -> &mut Self {
    //     self.inner.pTypeExternalMemoryHandleTypes = types;
    //     self
    // }
}
