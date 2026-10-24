use std::{
    collections::VecDeque,
    num::NonZeroUsize,
    sync::{Condvar, Mutex, MutexGuard, OnceLock},
    time::{Duration, Instant},
};

pub(super) struct BoundedSlotPool<T, R> {
    capacity: NonZeroUsize,
    state: OnceLock<PoolState<T, R>>,
}

struct PoolState<T, R> {
    slots: Box<[Mutex<Slot<T, R>>]>,
    coordinator: Mutex<Coordinator>,
    slot_available: Condvar,
}

struct Coordinator {
    available: VecDeque<usize>,
    waiters: VecDeque<u64>,
    next_ticket: u64,
}

enum Slot<T, R> {
    Empty,
    Reusable(T),
    Invalidated { value: T, reason: R },
    // A build can unwind before populating the slot. Retain that cause even
    // without a value so the next acquisition still reports an unwind rebuild.
    Unwound(Option<T>),
}

impl<T, R> Slot<T, R> {
    fn into_value(self) -> Option<T> {
        match self {
            Self::Empty => None,
            Self::Reusable(value) | Self::Invalidated { value, .. } => Some(value),
            Self::Unwound(value) => value,
        }
    }
}

pub(super) struct BoundedSlotLease<'a, T, R> {
    pool: &'a PoolState<T, R>,
    slot_index: usize,
    slot: Option<MutexGuard<'a, Slot<T, R>>>,
    wait: Duration,
}

struct WaitTicket<'a, T, R> {
    pool: &'a PoolState<T, R>,
    ticket: Option<u64>,
}

impl<T, R> BoundedSlotPool<T, R> {
    pub(super) const fn new(capacity: NonZeroUsize) -> Self {
        Self {
            capacity,
            state: OnceLock::new(),
        }
    }

    pub(super) fn acquire(&self) -> BoundedSlotLease<'_, T, R> {
        // First-use slot allocation is setup, not capacity-queue wait.
        let pool = self.state();
        let started = Instant::now();
        // Declare the ticket guard before taking the coordinator lock so an
        // unwind releases the lock before cancellation tries to acquire it.
        let mut ticket_guard = WaitTicket { pool, ticket: None };

        let slot_index = {
            let mut coordinator = pool
                .coordinator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let ticket = coordinator.next_ticket;
            coordinator.next_ticket = coordinator.next_ticket.wrapping_add(1);
            coordinator.waiters.push_back(ticket);
            ticket_guard.ticket = Some(ticket);
            loop {
                if coordinator.waiters.front() == Some(&ticket)
                    && let Some(slot_index) = coordinator.available.pop_front()
                {
                    let removed = coordinator.waiters.pop_front();
                    debug_assert_eq!(removed, Some(ticket));
                    ticket_guard.ticket = None;
                    pool.slot_available.notify_all();
                    break slot_index;
                }

                coordinator = pool
                    .slot_available
                    .wait(coordinator)
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
            }
        };

        let slot = pool.slots[slot_index]
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        BoundedSlotLease {
            pool,
            slot_index,
            slot: Some(slot),
            wait: started.elapsed(),
        }
    }

    pub(super) const fn capacity(&self) -> NonZeroUsize {
        self.capacity
    }

    fn state(&self) -> &PoolState<T, R> {
        self.state.get_or_init(|| PoolState {
            slots: (0..self.capacity.get())
                .map(|_| Mutex::new(Slot::Empty))
                .collect(),
            coordinator: Mutex::new(Coordinator {
                available: (0..self.capacity.get()).collect(),
                waiters: VecDeque::new(),
                next_ticket: 0,
            }),
            slot_available: Condvar::new(),
        })
    }

    #[cfg(test)]
    fn waiting_count(&self) -> usize {
        self.state()
            .coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .waiters
            .len()
    }
}

impl<T, R> BoundedSlotLease<'_, T, R> {
    pub(super) const fn slot_index(&self) -> usize {
        self.slot_index
    }

    pub(super) const fn wait(&self) -> Duration {
        self.wait
    }

    pub(super) fn is_reusable(&self) -> bool {
        matches!(self.slot(), Slot::Reusable(_))
    }

    pub(super) fn is_populated(&self) -> bool {
        self.get().is_some()
    }

    pub(super) fn invalidated_by_unwind(&self) -> bool {
        matches!(self.slot(), Slot::Unwound(_))
    }

    pub(super) fn get(&self) -> Option<&T> {
        match self.slot() {
            Slot::Empty => None,
            Slot::Reusable(value) | Slot::Invalidated { value, .. } => Some(value),
            Slot::Unwound(value) => value.as_ref(),
        }
    }

    pub(super) fn invalidation_reason(&self) -> Option<&R> {
        match self.slot() {
            Slot::Invalidated { reason, .. } => Some(reason),
            Slot::Empty | Slot::Reusable(_) | Slot::Unwound(_) => None,
        }
    }

    pub(super) fn replace(&mut self, value: T) -> Option<T> {
        std::mem::replace(self.slot_mut(), Slot::Reusable(value)).into_value()
    }

    pub(super) fn take(&mut self) -> Option<T> {
        std::mem::replace(self.slot_mut(), Slot::Empty).into_value()
    }

    pub(super) fn invalidate(&mut self, reason: R) {
        let slot = self.slot_mut();
        let state = std::mem::replace(slot, Slot::Empty);
        *slot = match state {
            Slot::Reusable(value) | Slot::Invalidated { value, .. } => {
                Slot::Invalidated { value, reason }
            }
            other => other,
        };
    }

    fn slot(&self) -> &Slot<T, R> {
        self.slot
            .as_deref()
            .expect("bounded slot lease must retain its slot")
    }

    fn slot_mut(&mut self) -> &mut Slot<T, R> {
        self.slot
            .as_deref_mut()
            .expect("bounded slot lease must retain its slot")
    }
}

impl<T, R> Drop for BoundedSlotLease<'_, T, R> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            let value = self.take();
            *self.slot_mut() = Slot::Unwound(value);
        }
        drop(self.slot.take());

        let mut coordinator = self
            .pool
            .coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        coordinator.available.push_back(self.slot_index);
        drop(coordinator);
        self.pool.slot_available.notify_all();
    }
}

impl<T, R> Drop for WaitTicket<'_, T, R> {
    fn drop(&mut self) {
        let Some(ticket) = self.ticket else {
            return;
        };
        let mut coordinator = self
            .pool
            .coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(position) = coordinator
            .waiters
            .iter()
            .position(|queued_ticket| *queued_ticket == ticket)
        {
            coordinator.waiters.remove(position);
            drop(coordinator);
            self.pool.slot_available.notify_all();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BoundedSlotPool, WaitTicket};
    use std::{
        num::NonZeroUsize,
        panic::{AssertUnwindSafe, catch_unwind},
        sync::{Arc, Barrier, mpsc},
        thread,
        time::{Duration, Instant},
    };

    const TIMEOUT: Duration = Duration::from_secs(5);

    #[test]
    #[expect(
        clippy::significant_drop_tightening,
        reason = "lease and coordinator scope deliberately cover the complete operation"
    )]
    fn concurrent_first_acquisitions_share_one_lazy_capacity() {
        let capacity = NonZeroUsize::new(4).unwrap();
        let pool = Arc::new(BoundedSlotPool::<usize, ()>::new(capacity));
        assert_eq!(pool.capacity(), capacity);
        assert!(pool.state.get().is_none());

        let barrier = Arc::new(Barrier::new(capacity.get()));
        let (acquired_tx, acquired_rx) = mpsc::channel();
        let workers = (0..capacity.get())
            .map(|_| {
                let pool = Arc::clone(&pool);
                let barrier = Arc::clone(&barrier);
                let acquired_tx = acquired_tx.clone();
                thread::spawn(move || {
                    barrier.wait();
                    let lease = pool.acquire();
                    acquired_tx.send(lease.slot_index()).unwrap();
                    barrier.wait();
                })
            })
            .collect::<Vec<_>>();
        drop(acquired_tx);
        let mut slots = (0..capacity.get())
            .map(|_| acquired_rx.recv_timeout(TIMEOUT).unwrap())
            .collect::<Vec<_>>();
        slots.sort_unstable();
        assert_eq!(slots, [0, 1, 2, 3]);
        for worker in workers {
            worker.join().expect("first acquisition should not panic");
        }
    }

    #[test]
    fn capacity_allows_independent_leases() {
        let pool = BoundedSlotPool::<usize, ()>::new(NonZeroUsize::new(2).unwrap());
        let first = pool.acquire();
        let second = pool.acquire();

        assert_ne!(first.slot_index(), second.slot_index());
    }

    #[test]
    fn exhausted_capacity_waits_until_release() {
        let pool = Arc::new(BoundedSlotPool::<usize, ()>::new(
            NonZeroUsize::new(1).unwrap(),
        ));
        let first = pool.acquire();
        let worker_pool = Arc::clone(&pool);
        let (acquired_tx, acquired_rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            let second = worker_pool.acquire();
            acquired_tx
                .send(second.slot_index())
                .expect("capacity test receiver should remain live");
        });

        wait_for_waiters(&pool, 1);
        assert!(acquired_rx.try_recv().is_err());
        drop(first);

        assert_eq!(
            acquired_rx
                .recv_timeout(TIMEOUT)
                .expect("waiting lease should acquire after release"),
            0,
        );
        worker.join().expect("capacity worker should not panic");
    }

    #[test]
    fn waiter_tickets_are_served_in_fifo_order() {
        let pool = Arc::new(BoundedSlotPool::<usize, ()>::new(
            NonZeroUsize::new(1).unwrap(),
        ));
        let held = pool.acquire();
        let (order_tx, order_rx) = mpsc::channel();

        let first_pool = Arc::clone(&pool);
        let first_tx = order_tx.clone();
        let first = thread::spawn(move || {
            let _lease = first_pool.acquire();
            first_tx.send(1).expect("order receiver should remain live");
            thread::sleep(Duration::from_millis(20));
        });
        wait_for_waiters(&pool, 1);

        let second_pool = Arc::clone(&pool);
        let second = thread::spawn(move || {
            let _lease = second_pool.acquire();
            order_tx.send(2).expect("order receiver should remain live");
        });
        wait_for_waiters(&pool, 2);
        drop(held);

        assert_eq!(order_rx.recv_timeout(TIMEOUT).unwrap(), 1);
        assert_eq!(order_rx.recv_timeout(TIMEOUT).unwrap(), 2);
        first.join().expect("first waiter should not panic");
        second.join().expect("second waiter should not panic");
    }

    #[test]
    #[expect(
        clippy::significant_drop_tightening,
        reason = "lease and coordinator scope deliberately cover the complete operation"
    )]
    fn cancelling_the_head_waiter_wakes_the_next_waiter() {
        let pool = Arc::new(BoundedSlotPool::<usize, ()>::new(
            NonZeroUsize::new(1).unwrap(),
        ));
        // Represent an interrupted acquisition at the head of the queue. A
        // slot is available, so cancellation must itself wake the next caller.
        let cancelled = {
            let mut coordinator = pool.state().coordinator.lock().unwrap();
            let ticket = coordinator.next_ticket;
            coordinator.next_ticket += 1;
            coordinator.waiters.push_back(ticket);
            WaitTicket {
                pool: pool.state(),
                ticket: Some(ticket),
            }
        };
        let worker_pool = Arc::clone(&pool);
        let (acquired_tx, acquired_rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            let lease = worker_pool.acquire();
            acquired_tx.send(lease.slot_index()).unwrap();
        });
        wait_for_waiters(&pool, 2);
        assert!(acquired_rx.try_recv().is_err());
        drop(cancelled);
        assert_eq!(acquired_rx.recv_timeout(TIMEOUT).unwrap(), 0);
        worker.join().expect("remaining waiter should acquire");
    }

    #[test]
    fn explicit_invalidation_retains_latest_reason_and_value_until_replacement() {
        let pool = BoundedSlotPool::new(NonZeroUsize::new(1).unwrap());
        let mut lease = pool.acquire();
        lease.replace(42);
        lease.invalidate("first");
        lease.invalidate("latest");
        drop(lease);

        let mut lease = pool.acquire();
        assert!(!lease.is_reusable());
        assert!(!lease.invalidated_by_unwind());
        assert_eq!(lease.get(), Some(&42));
        assert_eq!(lease.invalidation_reason(), Some(&"latest"));
        assert_eq!(lease.replace(7), Some(42));
        assert!(lease.is_reusable());
        assert_eq!(lease.invalidation_reason(), None);
    }

    #[test]
    #[expect(
        clippy::significant_drop_tightening,
        reason = "lease and coordinator scope deliberately cover the complete operation"
    )]
    fn unwind_invalidates_but_preserves_slot_value_for_safe_teardown() {
        for (value, invalidate) in [(None, false), (Some(42), false), (Some(42), true)] {
            let pool = BoundedSlotPool::<_, ()>::new(NonZeroUsize::new(1).unwrap());

            let panic = catch_unwind(AssertUnwindSafe(|| {
                let mut lease = pool.acquire();
                if let Some(value) = value {
                    lease.replace(value);
                    if invalidate {
                        lease.invalidate(());
                    }
                }
                panic!("synthetic leased-test panic");
            }));
            assert!(panic.is_err());

            let mut lease = pool.acquire();
            assert!(!lease.is_reusable());
            assert_eq!(lease.is_populated(), value.is_some());
            assert!(lease.invalidated_by_unwind());
            assert_eq!(lease.invalidation_reason(), None);
            assert_eq!(lease.get(), value.as_ref());
            assert_eq!(lease.replace(7), value);
            drop(lease);

            let lease = pool.acquire();
            assert!(lease.is_reusable());
            assert!(!lease.invalidated_by_unwind());
            assert_eq!(lease.get(), Some(&7));
        }
    }

    fn wait_for_waiters<T, R>(pool: &BoundedSlotPool<T, R>, expected: usize) {
        let started = Instant::now();
        while pool.waiting_count() < expected {
            assert!(started.elapsed() < TIMEOUT, "waiter did not enter queue");
            thread::yield_now();
        }
    }
}
