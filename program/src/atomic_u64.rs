pub(crate) use implementation::AtomicU64;

#[cfg(target_pointer_width = "64")]
mod implementation {
    use std::sync::atomic;

    pub(crate) struct AtomicU64(atomic::AtomicU64);

    impl AtomicU64 {
        pub(crate) const fn new(initial: u64) -> Self {
            Self(atomic::AtomicU64::new(initial))
        }

        pub(crate) fn fetch_add(&self, v: u64) -> u64 {
            self.0.fetch_add(v, atomic::Ordering::Relaxed)
        }
    }
}

#[cfg(not(target_pointer_width = "64"))]
mod implementation {
    use parking_lot::{const_mutex, Mutex};

    pub(crate) struct AtomicU64(Mutex<u64>);

    impl AtomicU64 {
        pub(crate) const fn new(initial: u64) -> Self {
            Self(const_mutex(initial))
        }

        pub(crate) fn fetch_add(&self, v: u64) -> u64 {
            let mut lock = self.0.lock();
            let i = *lock;
            *lock = i + v;
            i
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AtomicU64;

    #[test]
    fn test_atomic_u64_initialization() {
        let atomic = AtomicU64::new(10);
        assert_eq!(atomic.fetch_add(0), 10);
    }

    #[test]
    fn test_atomic_u64_fetch_add() {
        let atomic = AtomicU64::new(5);
        assert_eq!(atomic.fetch_add(3), 5);
        assert_eq!(atomic.fetch_add(2), 8);
    }

    #[test]
    fn test_atomic_u64_concurrent_add() {
        let atomic = std::sync::Arc::new(AtomicU64::new(0));

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let atomic_clone = atomic.clone();
                std::thread::spawn(move || {
                    for _ in 0..100 {
                        atomic_clone.fetch_add(1);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(atomic.fetch_add(0), 1000);
    }
}
