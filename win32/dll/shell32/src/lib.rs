#![allow(non_snake_case)]

mod builtin;

pub use builtin::DLL;

use win32_system::System;

#[win32_derive::dllexport]
pub fn ShellExecuteA(
    sys: &dyn System,
    hwnd: u32,
    lpOperation: Option<&str>,
    lpFile: Option<&str>,
    lpParameters: Option<&str>,
    lpDirectory: Option<&str>,
    nShowCmd: u32,
) -> u32 {
    // Stub implementation: return a value > 32 to indicate success
    // In the real ShellExecuteA, values <= 32 indicate errors
    33
}