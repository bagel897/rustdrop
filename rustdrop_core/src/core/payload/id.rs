use std::sync::atomic::AtomicI64;

static PAYLOAD_COUNTER: AtomicI64 = AtomicI64::new(0);
pub fn get_payload() -> i64 {
    PAYLOAD_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}
static UNIQUE_COUNTER: AtomicI64 = AtomicI64::new(0);
pub fn get_unique() -> i64 {
    UNIQUE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}
