use crate::{SECURITY_ATTRIBUTES, KernelObject, get_state};
use win32_system::{Mutex, System};
use win32_winapi::HANDLE;

#[win32_derive::dllexport]
pub fn CreateMutexA(
    sys: &dyn System,
    lpMutexAttributes: Option<&mut SECURITY_ATTRIBUTES>,
    bInitialOwner: bool,
    lpName: Option<&str>,
) -> HANDLE<()> {
    let name = if let Some(name) = lpName {
        let state = get_state(sys);
        if let Some(mx) = state.objects.iter().find(|(_, mx)| {
            if let KernelObject::Mutex(mx) = mx {
                if let Some(n) = &mx.name {
                    if n == name {
                        return true;
                    }
                }
            }
            false
        }) {
            // Return existing named mutex
            return HANDLE::from_raw(mx.0.to_raw());
        }
        Some(name.to_string())
    } else {
        None
    };

    // Get current thread ID for ownership tracking
    let thread_id = sys.get_thread_id();

    HANDLE::from_raw(
        get_state(sys)
            .objects
            .add(KernelObject::Mutex(Mutex::new(
                name,
                bInitialOwner,
                thread_id,
            )))
            .to_raw(),
    )
}

#[win32_derive::dllexport]
pub fn ReleaseMutex(sys: &dyn System, hMutex: HANDLE<()>) -> bool {
    let thread_id = sys.get_thread_id();
    let state = get_state(sys);

    if let Some(KernelObject::Mutex(mutex)) = state.objects.get(hMutex) {
        mutex.release(thread_id)
    } else {
        false
    }
}

#[win32_derive::dllexport]
pub fn OpenMutexA(
    sys: &dyn System,
    dwDesiredAccess: u32,
    bInheritHandle: bool,
    lpName: Option<&str>,
) -> HANDLE<()> {
    HANDLE::null() // fail
}
