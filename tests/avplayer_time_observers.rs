mod support;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use avplayer::prelude::*;

struct DropFlag(Arc<AtomicBool>);

impl Drop for DropFlag {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

#[test]
fn main_queue_observers_drop_without_a_serviced_main_queue() -> support::TestResult {
    let player = support::player("test-time-observer-main-queue")?;
    let closure_dropped = Arc::new(AtomicBool::new(false));
    let guard = DropFlag(Arc::clone(&closure_dropped));
    let periodic = player.add_periodic_time_observer(Time::new(1, 10), None, move |_| {
        let _ = &guard;
    })?;
    let boundary_dropped = Arc::new(AtomicBool::new(false));
    let boundary_guard = DropFlag(Arc::clone(&boundary_dropped));
    let boundary = player.add_boundary_time_observer(&[Time::new(1, 4)], None, move || {
        let _ = &boundary_guard;
    })?;
    player.play();
    thread::sleep(Duration::from_millis(100));
    let started = Instant::now();
    drop(periodic);
    drop(boundary);
    assert!(started.elapsed() < Duration::from_secs(2));
    assert!(closure_dropped.load(Ordering::SeqCst));
    assert!(boundary_dropped.load(Ordering::SeqCst));
    player.pause();
    Ok(())
}

#[test]
fn failed_registrations_release_the_callback() -> support::TestResult {
    let player = support::player("test-time-observer-failure")?;
    for interval in [Time::new(0, 1), Time::invalid(), Time::new(-1, 10)] {
        let dropped = Arc::new(AtomicBool::new(false));
        let guard = DropFlag(Arc::clone(&dropped));
        let result = player.add_periodic_time_observer(interval, None, move |_| {
            let _ = &guard;
        });
        assert!(matches!(result, Err(AVPlayerError::ObserverFailed(_))));
        assert!(dropped.load(Ordering::SeqCst));
    }

    let dropped = Arc::new(AtomicBool::new(false));
    let guard = DropFlag(Arc::clone(&dropped));
    let result = player.add_boundary_time_observer(&[], None, move || {
        let _ = &guard;
    });
    assert!(matches!(result, Err(AVPlayerError::ObserverFailed(_))));
    assert!(dropped.load(Ordering::SeqCst));
    Ok(())
}
