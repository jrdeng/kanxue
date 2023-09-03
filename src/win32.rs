use encoding_rs::GB18030;
use std::ffi::CStr;
use std::os::raw::c_char;

extern "C" {
    fn c_raise_privilege();
    fn c_window_at_cursor_point() -> i64;
    fn c_open_process(hwnd: i64) -> i64;
    fn c_read_memory(handle: i64, address: i64, data: *mut i8, len: u64) -> bool;
    fn c_read_memory_as_string(handle: i64, address: i64, len: u64) -> *const c_char;
    fn c_free_string(str: *const c_char);
    fn c_close_handle(handle: i64);
}

pub fn raise_privilege() {
    unsafe { c_raise_privilege() }
}

pub fn window_at_cursor_point() -> i64 {
    unsafe { c_window_at_cursor_point() }
}

pub fn open_process(hwnd: i64) -> i64 {
    unsafe { c_open_process(hwnd) }
}

pub fn read_memory_as_number(handle: i64, address: i64, len: u64) -> i64 {
    let mut data: i64 = 0;
    unsafe { c_read_memory(handle, address, &mut data as *mut i64 as *mut i8, len) };
    return data;
}

pub fn read_memory_as_string(handle: i64, address: i64, len: u64) -> String {
    let c_str = unsafe { c_read_memory_as_string(handle, address, len) };
    if c_str.is_null() {
        // println!("failed to read memory");
        return "".to_string();
    }

    let c_slice = unsafe { CStr::from_ptr(c_str) };

    let encoded_bytes = c_slice.to_bytes();
    let res = GB18030
        .decode_without_bom_handling(encoded_bytes)
        .0
        .into_owned();

    // must be free after decoding?
    unsafe {
        c_free_string(c_str);
    }

    res
}

pub fn close_handle(handle: i64) {
    unsafe { c_close_handle(handle) }
}
