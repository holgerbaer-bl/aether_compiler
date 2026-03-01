use std::collections::HashMap;
use std::sync::Mutex;

// The types of resources we can manage
pub enum NativeHandle {
    Counter(StatefulCounter),
}

// Our dummy stateful Rust object
pub struct StatefulCounter {
    pub count: i64,
}

// Global thread-safe registry
// Instead of lazy_static we'll use a const Mutex with an Option since lazy_static might not be available
static COUNTER_REGISTRY: Mutex<Option<HashMap<usize, NativeHandle>>> = Mutex::new(None);
static COUNTER_NEXT_ID: Mutex<usize> = Mutex::new(1);

fn with_registry<F, R>(f: F) -> R
where
    F: FnOnce(&mut HashMap<usize, NativeHandle>) -> R,
{
    let mut option_guard = COUNTER_REGISTRY.lock().unwrap();
    if option_guard.is_none() {
        *option_guard = Some(HashMap::new());
    }
    f(option_guard.as_mut().unwrap())
}

// FFI Implementations
pub fn registry_create_counter() -> i64 {
    let mut id_guard = COUNTER_NEXT_ID.lock().unwrap();
    let id = *id_guard;
    *id_guard += 1;

    let counter = StatefulCounter { count: 0 };
    with_registry(|registry| {
        registry.insert(id, NativeHandle::Counter(counter));
    });

    id as i64
}

pub fn registry_increment(handle_id: i64) {
    let id = handle_id as usize;
    with_registry(|registry| {
        if let Some(NativeHandle::Counter(counter)) = registry.get_mut(&id) {
            counter.count += 1;
        } else {
            eprintln!(
                "[KnotenCore Registry] Error: Counter handle {} not found.",
                handle_id
            );
        }
    });
}

pub fn registry_get_value(handle_id: i64) -> i64 {
    let id = handle_id as usize;
    with_registry(|registry| {
        if let Some(NativeHandle::Counter(counter)) = registry.get(&id) {
            counter.count
        } else {
            eprintln!(
                "[KnotenCore Registry] Error: Counter handle {} not found.",
                handle_id
            );
            -1
        }
    })
}

pub fn registry_free(handle_id: i64) {
    let id = handle_id as usize;
    with_registry(|registry| {
        if registry.remove(&id).is_some() {
            // Memory freed natively
        } else {
            eprintln!(
                "[KnotenCore Registry] Warning: Double free or invalid handle {}.",
                handle_id
            );
        }
    });
}
