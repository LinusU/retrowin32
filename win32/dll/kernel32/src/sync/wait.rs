//! WaitFor* functions that can block on various types of kernel objects.

use crate::{KernelObject, get_state};
use memory::Extensions;
use win32_system::{ArcEvent, System, Wait, WaitResult};
use win32_winapi::HANDLE;

impl KernelObject {
    pub fn get_event(&self) -> &ArcEvent {
        match self {
            KernelObject::Event(event) => event,
            KernelObject::Thread(thread) => &thread.terminated,
            KernelObject::Mutex(_) => panic!("Cannot get event from mutex"),
        }
    }

    /// Check if this kernel object is signaled (available).
    pub fn is_signaled(&self, thread_id: u32) -> bool {
        match self {
            KernelObject::Event(event) => *event.signaled.lock().unwrap(),
            KernelObject::Thread(thread) => *thread.terminated.signaled.lock().unwrap(),
            KernelObject::Mutex(mutex) => mutex.is_signaled() || {
                // Mutexes are recursive - if we already own it, it's "signaled"
                let owner = mutex.owner.lock().unwrap();
                owner.map(|id| id == thread_id).unwrap_or(false)
            },
        }
    }

    /// Acquire this kernel object (for mutexes, this means taking ownership).
    pub fn acquire(&self, thread_id: u32) {
        match self {
            KernelObject::Event(event) => {
                if !event.manual_reset {
                    // Auto-reset events get reset when acquired
                    *event.signaled.lock().unwrap() = false;
                }
            }
            KernelObject::Thread(_) => {
                // Threads don't need acquisition logic
            }
            KernelObject::Mutex(mutex) => {
                mutex.acquire(thread_id);
            }
        }
    }
}

/// The primitive beneath WaitForMultipleObjects etc.
pub async fn wait_for_objects(
    sys: &mut dyn System,
    handles: Box<[HANDLE<()>]>,
    wait_all: bool,
    wait: Wait,
) -> WaitResult {
    if wait_all {
        todo!("WaitForMultipleObjects: bWaitAll");
    }

    let thread_id = sys.get_thread_id();

    loop {
        for (i, &handle) in handles.iter().enumerate() {
            let state = get_state(sys);
            if let Some(obj) = state.objects.get(handle) {
                if obj.is_signaled(thread_id) {
                    obj.acquire(thread_id);
                    return WaitResult::Object(i as u32);
                }
            }
        }

        let until = match wait {
            Wait::None => return WaitResult::Timeout,
            Wait::Millis(until) => {
                if sys.host().ticks() >= until {
                    return WaitResult::Timeout;
                }
                Some(until)
            }
            Wait::Forever => None,
        };

        // log::info!(
        //     "{:?}: waiting for {:?}",
        //     crate::winapi::kernel32::current_thread(sys),
        //     handles
        // );

        sys.block(until).await;
    }
}

/// The primitive beneath WaitForMultipleObjects etc. (legacy event-only version).
pub async fn wait_for_events(
    sys: &mut dyn System,
    events: Box<[ArcEvent]>,
    wait_all: bool,
    wait: Wait,
) -> WaitResult {
    if wait_all {
        todo!("WaitForMultipleObjects: bWaitAll");
    }

    loop {
        for (i, event) in events.iter().enumerate() {
            let mut signaled = event.signaled.lock().unwrap();
            if *signaled {
                if !event.manual_reset {
                    // TODO: this should wake up exactly one waiting thread
                    *signaled = false;
                }
                return WaitResult::Object(i as u32);
            }
        }

        let until = match wait {
            Wait::None => return WaitResult::Timeout,
            Wait::Millis(until) => {
                if sys.host().ticks() >= until {
                    return WaitResult::Timeout;
                }
                Some(until)
            }
            Wait::Forever => None,
        };

        // log::info!(
        //     "{:?}: waiting for {:?}",
        //     crate::winapi::kernel32::current_thread(sys),
        //     handles
        // );

        sys.block(until).await;
    }
}

#[win32_derive::dllexport]
pub async fn WaitForSingleObject(
    sys: &mut dyn System,
    handle: HANDLE<()>,
    dwMilliseconds: u32,
) -> u32 {
    let wait = Wait::from_millis(sys.host(), dwMilliseconds);
    wait_for_objects(sys, [handle].into(), false, wait)
        .await
        .to_code()
}

#[win32_derive::dllexport]
pub async fn WaitForMultipleObjects(
    sys: &mut dyn System,
    nCount: u32,
    lpHandles: u32,
    bWaitAll: bool,
    dwMilliseconds: u32,
) -> u32 /* WAIT_EVENT */ {
    let handles: Vec<HANDLE<()>> = sys
        .mem()
        .iter_pod::<HANDLE<()>>(lpHandles, nCount)
        .collect();
    let wait = Wait::from_millis(sys.host(), dwMilliseconds);
    wait_for_objects(sys, handles.into(), bWaitAll, wait)
        .await
        .to_code()
}
