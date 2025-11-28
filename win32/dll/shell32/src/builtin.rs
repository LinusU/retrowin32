#![doc = r" Generated code, do not edit.  See winapi/builtin.rs for an overview."]
#![allow(unused_imports)]
#![allow(unused_variables)]
use win32_system::dll::*;
mod wrappers {
    use crate as shell32;
    use crate::*;
    use ::memory::Extensions;
    use win32_system::{System, trace};
    use win32_winapi::{calling_convention::*, *};
    pub unsafe fn ShellExecuteA(sys: &mut dyn System, stack_args: u32) -> ABIReturn {
        use shell32::*;
        unsafe {
            let mem = sys.mem().detach();
            let hwnd = <u32>::from_stack(mem, stack_args + 0u32);
            let lpOperation = <Option<&str>>::from_stack(mem, stack_args + 4u32);
            let lpFile = <Option<&str>>::from_stack(mem, stack_args + 8u32);
            let lpParameters = <Option<&str>>::from_stack(mem, stack_args + 12u32);
            let lpDirectory = <Option<&str>>::from_stack(mem, stack_args + 16u32);
            let nShowCmd = <u32>::from_stack(mem, stack_args + 20u32);
            let __trace_record = if trace::enabled("shell32") {
                trace::Record::new(
                    shell32::ShellExecuteA_pos,
                    "shell32",
                    "ShellExecuteA",
                    &[
                        ("hwnd", &hwnd),
                        ("lpOperation", &lpOperation),
                        ("lpFile", &lpFile),
                        ("lpParameters", &lpParameters),
                        ("lpDirectory", &lpDirectory),
                        ("nShowCmd", &nShowCmd),
                    ],
                )
                .enter()
            } else {
                None
            };
            let result = shell32::ShellExecuteA(
                sys,
                hwnd,
                lpOperation,
                lpFile,
                lpParameters,
                lpDirectory,
                nShowCmd,
            );
            if let Some(mut __trace_record) = __trace_record {
                __trace_record.exit(&result);
            }
            result.into()
        }
    }
}
const SHIMS: [Shim; 1usize] = [Shim {
    name: "ShellExecuteA",
    func: Handler::Sync(wrappers::ShellExecuteA),
}];
pub const DLL: BuiltinDLL = BuiltinDLL {
    file_name: "shell32.dll",
    shims: &SHIMS,
    raw: std::include_bytes!("../shell32.dll"),
};
