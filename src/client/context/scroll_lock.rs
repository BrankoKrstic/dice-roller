use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct ScrollLockContext {
    locks: RwSignal<usize>,
}

impl ScrollLockContext {
    pub fn lock(&self) {
        self.locks.update(|locks| *locks += 1);
    }

    pub fn unlock(&self) {
        self.locks.update(|locks| *locks = locks.saturating_sub(1));
    }

    pub fn is_locked(&self) -> bool {
        self.locks.get() > 0
    }
}

pub fn provide_scroll_lock_context() {
    provide_context(ScrollLockContext {
        locks: RwSignal::new(0),
    });
}

pub fn use_scroll_lock_context() -> ScrollLockContext {
    use_context::<ScrollLockContext>().unwrap_or(ScrollLockContext {
        locks: RwSignal::new(0),
    })
}
