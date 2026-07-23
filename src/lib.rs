#![no_std]
//! This crate provides an easy, fast, cross-platform way to retrieve the
//! memory page size of the current system.
//!
//! Modern hardware and software tend to load data into RAM (and transfer data
//! from RAM to disk) in discrete chunk called pages. This crate provides a
//! helper method to retrieve the size in bytes of these pages. Since the page
//! size *should not* change during execution, this crate will cache the result
//! after it has been called once.
//!
//! To make this crate useful for writing memory allocators, it does not require
//! (but can use) the Rust standard library.
//!
//! Since Windows addresses sometimes have to correspond with an allocation
//! granularity that does not always match the size of the page, I have included
//! a method to retrieve that as well.
//!
//! # Example
//!
//! ```rust
//! extern crate page_size;
//! println!("{}", page_size::get());
//! ```

#[cfg(feature = "no_std")]
extern crate spin;
#[cfg(feature = "no_std")]
use spin::LazyLock;

#[cfg(not(feature = "no_std"))]
extern crate std;
#[cfg(not(feature = "no_std"))]
use std::sync::LazyLock;

#[cfg(unix)]
extern crate libc;

#[cfg(windows)]
extern crate winapi;

/// This function retrieves the system's memory page size.
///
/// # Example
///
/// ```rust
/// extern crate page_size;
/// println!("{}", page_size::get());
/// ```
pub fn get() -> usize {
    static PAGE_SIZE: LazyLock<usize> = LazyLock::new(os::get);
    *PAGE_SIZE
}

pub use os::get_granularity;

// Unix Section
#[cfg(unix)]
mod os {
    use libc::{sysconf, _SC_PAGESIZE};

    #[inline]
    pub fn get() -> usize {
        unsafe { sysconf(_SC_PAGESIZE) as usize }
    }

    // Unix does not have a specific allocation granularity.
    // The page size works well.
    #[inline]
    pub fn get_granularity() -> usize {
        get()
    }
}

// WebAssembly section
#[cfg(all(not(target_os = "emscripten"), any(target_arch = "wasm32", target_arch = "wasm64")))]
mod os {
    #[inline]
    pub fn get() -> usize {
        4096
    }
    // WebAssembly does not have a specific allocation granularity.
    // The "page size" works well.
    #[inline]
    pub fn get_granularity() -> usize {
        65536 // <https://webassembly.github.io/spec/core/exec/runtime.html#page-size>
    }
}

// Windows Section
#[cfg(windows)]
mod os {
    use core::mem;
    use super::LazyLock;

    use winapi::um::sysinfoapi::GetSystemInfo;
    use winapi::um::sysinfoapi::{LPSYSTEM_INFO, SYSTEM_INFO};

    #[inline]
    pub fn get() -> usize {
        unsafe {
            let mut info: SYSTEM_INFO = mem::zeroed();
            GetSystemInfo(&mut info as LPSYSTEM_INFO);

            info.dwPageSize as usize
        }
    }

    #[inline]
    pub fn get_granularity() -> usize {
        static GRANULARITY: LazyLock<usize> = LazyLock::new(|| unsafe {
            let mut info: SYSTEM_INFO = mem::zeroed();
            GetSystemInfo(&mut info as LPSYSTEM_INFO);

            info.dwAllocationGranularity as usize
        });
        *GRANULARITY
    }
}

// Stub Section

#[cfg(not(any(unix, windows, target_arch = "wasm32", target_arch = "wasm64")))]
mod os {
    #[inline]
    pub fn get() -> usize {
        4096 // 4k is the default on many systems
    }
    #[inline]
    pub fn get_granularity() -> usize {
        get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {
        let _page_size = get();
    }

    #[test]
    fn test_get_granularity() {
        let _granularity = get_granularity();
    }
}
