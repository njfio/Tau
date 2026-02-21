#[test]
fn integration_contains_panic_and_unsafe_markers() {
    panic!("integration panic marker");
}

pub fn integration_unsafe_marker(bytes: &mut [u8]) {
    if bytes.is_empty() {
        return;
    }
    // Fixture-only unsafe marker for audit classification coverage.
    unsafe {
        let ptr = bytes.as_mut_ptr();
        *ptr = 7;
    }
}
