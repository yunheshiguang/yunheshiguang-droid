use std::collections::VecDeque;
use std::sync::{Mutex, PoisonError};
use std::thread;

pub fn run_bounded<S, R>(
    serials: Vec<String>,
    limit: usize,
    init: impl Fn() -> S + Sync,
    run: impl Fn(&S, String) -> R + Sync,
) -> Vec<R>
where
    R: Send,
{
    let workers = serials.len().min(limit);
    let queue = Mutex::new(VecDeque::from(serials));
    let results = Mutex::new(Vec::new());

    thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                let state = init();

                while let Some(serial) = next_serial(&queue) {
                    let result = run(&state, serial);
                    results
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .push(result);
                }
            });
        }
    });

    results.into_inner().unwrap_or_else(PoisonError::into_inner)
}

fn next_serial(queue: &Mutex<VecDeque<String>>) -> Option<String> {
    queue
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .pop_front()
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::Duration;

    use super::run_bounded;

    fn serials(count: usize) -> Vec<String> {
        (0..count)
            .map(|index| format!("device-{index:02}"))
            .collect()
    }

    #[test]
    fn runs_every_device_exactly_once() {
        let mut results = run_bounded(serials(10), 3, || (), |_, serial| serial);
        results.sort();

        assert_eq!(results, serials(10));
    }

    #[test]
    fn never_runs_more_devices_at_once_than_the_limit() {
        let running = AtomicUsize::new(0);
        let peak = AtomicUsize::new(0);

        run_bounded(
            serials(12),
            4,
            || (),
            |_, _| {
                let now = running.fetch_add(1, Ordering::SeqCst) + 1;
                peak.fetch_max(now, Ordering::SeqCst);
                thread::sleep(Duration::from_millis(10));
                running.fetch_sub(1, Ordering::SeqCst);
            },
        );

        let peak = peak.load(Ordering::SeqCst);
        assert!(
            peak <= 4,
            "{peak} devices ran at once, above the limit of 4"
        );
        assert!(peak > 1, "devices ran one at a time");
    }

    #[test]
    fn opens_one_client_per_worker_and_no_more_workers_than_devices() {
        let clients = AtomicUsize::new(0);

        run_bounded(
            serials(2),
            4,
            || {
                clients.fetch_add(1, Ordering::SeqCst);
            },
            |_, _| (),
        );

        assert_eq!(clients.load(Ordering::SeqCst), 2);
    }
}
