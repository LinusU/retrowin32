use win32_system::System;

pub fn clear(ptr: Option<&mut [u8]>) {
    ptr.unwrap().fill(0);
}

pub fn malloc(sys: &mut dyn System, size: u32) -> u32 {
    let memory = sys.memory_mut();
    memory.process_heap.alloc(memory.imp.mem(), size)
}

pub fn free(sys: &mut dyn System, ptr: u32) {
    if ptr != 0 {
        let memory = sys.memory_mut();
        memory.process_heap.free(memory.mem(), ptr)
    }
}

pub fn realloc(sys: &mut dyn System, ptr: u32, size: u32) -> u32 {
    let memory = sys.memory_mut();
    let new_ptr = memory.process_heap.alloc(memory.mem(), size);

    if new_ptr != 0 {
        let old_size = memory.process_heap.size(memory.mem(), ptr);
        let copy_size = old_size.min(size);
        memory.mem().copy(ptr, new_ptr, copy_size);
        memory.process_heap.free(memory.mem(), ptr);
    }

    new_ptr
}
