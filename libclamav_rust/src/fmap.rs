use log::error;
use mbox::MBox;
use std::ptr;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
mod internal {
    include!(concat!(env!("OUT_DIR"), "/fmap_internal.rs"));
}

use internal::fmap_t;
use internal::size_t;

extern "C" {
    pub fn cli_getpagesize() -> libc::c_int;
    pub fn fmap_align_items(sz: u64, al: u64) -> u64;
    pub fn unmap_malloc(m: *mut fmap_t);
    pub fn mem_need(m: *mut fmap_t, at: u64, len_hint: u64, lock: i32) -> *const libc::c_void;
    pub fn mem_need_offstr(m: *mut fmap_t, at: size_t, len_hint: size_t) -> *const libc::c_void;
    pub fn mem_gets(
        m: *mut fmap_t,
        dst: *mut ::std::os::raw::c_char,
        at: *mut size_t,
        max_len: size_t,
    ) -> *const ::std::os::raw::c_void;
    pub fn mem_unneed_off(m: *mut fmap_t, at: size_t, len: size_t);
    pub fn cli_strdup(s: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_char;
}

/// Used only by test functions
#[no_mangle]
pub extern "C" fn fmap_zeroed() -> *mut internal::fmap_t {
    unsafe {
        let fmap = libc::calloc(1, std::mem::size_of::<internal::cl_fmap>());
        fmap as *mut internal::fmap_t
    }
}

#[no_mangle]
pub extern "C" fn funmap(map: *mut internal::fmap_t) {
    unsafe {
        assert!(!map.is_null());
        if let Some(unmap) = (*map).unmap {
            unmap(map)
        }
    };
}

#[cfg(unix)]
fn getpagesize() -> u64 {
    sysconf::page::pagesize() as u64
}

#[no_mangle]
pub extern "C" fn fmap_open_memory(
    start: *const ::std::os::raw::c_void,
    len: size_t,
    name: *const libc::c_char,
) -> *mut internal::fmap_t {
    let pgsz = getpagesize();

    let name_copy = if !name.is_null() {
        let name = unsafe { cli_strdup(name) };
        if name.is_null() {
            error!("fmap: failed to duplicate map name");
            return ptr::null_mut();
        }
        name
    } else {
        ptr::null_mut()
    };

    let map = MBox::new(fmap_t {
        nested_offset: Default::default(),
        pread_cb: Default::default(),
        paged: Default::default(),
        aging: Default::default(),
        bitmap: ptr::null_mut(),
        data: start as *const ::std::os::raw::c_void,
        dont_cache_flag: Default::default(),
        gets: Some(mem_gets),
        handle: ptr::null_mut(),
        handle_is_fd: Default::default(),
        have_maphash: false,
        len: len as u64,
        maphash: Default::default(),
        mtime: Default::default(),
        name: name_copy,
        need: Some(mem_need),
        need_offstr: Some(mem_need_offstr),
        offset: Default::default(),
        pages: unsafe { fmap_align_items(len as u64, pgsz as u64) },
        pgsz: getpagesize(),
        real_len: len,
        unmap: Some(unmap_malloc),
        unneed_off: Some(mem_unneed_off),
    });

    MBox::into_raw(map)
}
