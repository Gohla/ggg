/// Starts the profiler.
pub fn start() {
  #[cfg(feature = "profile-with-tracy")]
  tracy_client::Client::start();
}


/// Type for a global allocator that can profile memory allocations.
///
/// # Example
///
/// ```
/// #[global_allocator]
/// static GLOBAL: os::profile::Allocator = os::profile::create_allocator();
/// ```
pub type Allocator = allocator_internal::Allocator;
/// Create a global allocator that can profile memory allocations.
pub const fn create_allocator() -> Allocator {
  allocator_internal::create_allocator()
}

#[cfg(not(feature = "profile-with-tracy"))]
mod allocator_internal {
  pub type Allocator = std::alloc::System;
  pub const fn create_allocator() -> Allocator {
    std::alloc::System
  }
}

#[cfg(feature = "profile-with-tracy")]
mod allocator_internal {
  pub type Allocator = tracy_client::ProfiledAllocator<std::alloc::System>;
  pub const fn create_allocator() -> Allocator {
    // Specifying a non-zero `callstack_depth` will enable collection of callstacks for memory allocations. The number
    // provided will limit the number of call frames collected. Note that enabling callstack collection introduces a
    // non-trivial amount of overhead to each allocation and deallocation.
    Allocator::new(std::alloc::System, 0)
  }
}
