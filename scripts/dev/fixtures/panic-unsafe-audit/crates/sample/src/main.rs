pub fn prod_panic_path() {
    panic!("prod panic marker");
}

pub fn prod_unsafe_path(buffer: &mut [u8]) {
    if buffer.is_empty() {
        return;
    }
    // Fixture-only unsafe marker for audit classification coverage.
    unsafe {
        let ptr = buffer.as_mut_ptr();
        *ptr = 42;
    }
}
