//! For some reason kernel32 exports functions that I would've expected to find in the libc...

use super::*;

const TRACE_CONTEXT: &'static str = "kernel32/libc";

#[win32_derive::dllexport]
pub fn lstrlenW(_machine: &mut Machine, lpString: Option<&Str16>) -> u32 {
    match lpString {
        None => 0,
        // The mapping to Str16 already computes the string length.
        Some(str) => str.len() as u32,
    }
}

#[win32_derive::dllexport]
pub fn lstrcpyA(machine: &mut Machine, lpString1: u32, lpString2: Option<&str>) -> u32 {
    let src = lpString2.unwrap();
    let dst = machine
        .mem()
        .sub(lpString1, (src.len() + 1) as u32)
        .as_mut_slice_todo();
    dst[..src.len()].copy_from_slice(src.as_bytes());
    dst[src.len()] = 0;
    lpString1
}

#[win32_derive::dllexport]
pub fn lstrcpyW(machine: &mut Machine, lpString1: u32, lpString2: Option<&Str16>) -> u32 {
    let lpString2 = lpString2.unwrap();
    // lpString1 is a buffer of unspecified size!
    let copy_len = (lpString2.len() + 1) * 2; // include nul
    let dst = machine.mem().sub(lpString1, copy_len as u32);
    let src =
        unsafe { std::slice::from_raw_parts(lpString2.buf().as_ptr() as *const u8, copy_len) };
    dst.as_mut_slice_todo().copy_from_slice(src);
    lpString1
}

#[win32_derive::dllexport]
pub fn lstrcmpiA(_machine: &mut Machine, lpString1: Option<&str>, lpString2: Option<&str>) -> i32 {
    let lpString1 = lpString1.unwrap();
    let lpString2 = lpString2.unwrap();

    for (a, b) in lpString1.bytes().zip(lpString2.bytes()) {
        // TODO: case insensitive
        if a < b {
            return -1;
        } else if a > b {
            return 1;
        }
    }

    match lpString1.len().cmp(&lpString2.len()) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Greater => 1,
        std::cmp::Ordering::Equal => 0,
    }
}
