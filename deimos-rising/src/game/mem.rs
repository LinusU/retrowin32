pub fn clear(ptr: Option<&mut [u8]>) {
    ptr.unwrap().fill(0);
}
