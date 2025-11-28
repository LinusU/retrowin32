use std::sync::{Arc, Mutex as StdMutex};

pub type ArcMutex = Arc<Mutex>;

/// Mutex objects, used for synchronization between Windows threads.
///
/// Win32 mutexes are recursive and can be owned by a thread.
/// In the emulator there are no real threads, but we still track ownership
/// for proper signaling semantics.
pub struct Mutex {
    pub name: Option<String>,
    /// Thread ID that currently owns the mutex, or None if unowned.
    /// For the emulator, we use a simple u32 thread ID.
    pub owner: StdMutex<Option<u32>>,
}

impl Mutex {
    pub fn new(name: Option<String>, initially_owned: bool, owner_id: u32) -> Arc<Self> {
        Arc::new(Self {
            name,
            owner: StdMutex::new(if initially_owned { Some(owner_id) } else { None }),
        })
    }

    /// Check if the mutex is signaled (available to acquire).
    /// A mutex is signaled when it has no owner.
    pub fn is_signaled(&self) -> bool {
        self.owner.lock().unwrap().is_none()
    }

    /// Acquire the mutex for the given thread.
    pub fn acquire(&self, thread_id: u32) -> bool {
        let mut owner = self.owner.lock().unwrap();
        if owner.is_none() {
            *owner = Some(thread_id);
            true
        } else {
            // Win32 mutexes are recursive, so if the same thread already owns it,
            // this is allowed (though we don't track recursion count in this minimal impl)
            owner.unwrap() == thread_id
        }
    }

    /// Release the mutex.
    pub fn release(&self, thread_id: u32) -> bool {
        let mut owner = self.owner.lock().unwrap();
        if let Some(current_owner) = *owner {
            if current_owner == thread_id {
                *owner = None;
                return true;
            }
        }
        false
    }
}
