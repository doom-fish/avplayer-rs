mod support;

use std::ffi::c_void;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{self, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use avplayer::prelude::*;

extern "C" {
    static kCFRunLoopDefaultMode: *const c_void;
    fn CFRunLoopRunInMode(
        mode: *const c_void,
        seconds: f64,
        return_after_source_handled: u8,
    ) -> i32;
}

type Scenario = fn() -> support::TestResult;

struct DropFlag(Arc<AtomicBool>);

impl Drop for DropFlag {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

fn wait_until(deadline: Duration, condition: impl Fn() -> bool) -> bool {
    let start = Instant::now();
    while start.elapsed() < deadline {
        if condition() {
            return true;
        }
        thread::sleep(Duration::from_millis(10));
    }
    condition()
}

fn in_flight_drop(label: Option<&str>, stem: &str) -> support::TestResult {
    let player = support::player(stem)?;
    let entered = Arc::new(AtomicBool::new(false));
    let finished = Arc::new(AtomicBool::new(false));
    let calls = Arc::new(AtomicUsize::new(0));
    let observer = {
        let entered = Arc::clone(&entered);
        let finished = Arc::clone(&finished);
        let calls = Arc::clone(&calls);
        player.add_periodic_time_observer(Time::new(1, 20), label, move |_| {
            if calls.fetch_add(1, Ordering::SeqCst) == 0 {
                entered.store(true, Ordering::SeqCst);
                thread::sleep(Duration::from_millis(300));
                finished.store(true, Ordering::SeqCst);
            }
        })?
    };
    player.play();
    if !wait_until(Duration::from_secs(5), || entered.load(Ordering::SeqCst)) {
        return Err("periodic observer never fired".into());
    }
    drop(observer);
    if !finished.load(Ordering::SeqCst) {
        return Err("drop returned while the callback was still running".into());
    }
    let calls_after_drop = calls.load(Ordering::SeqCst);
    thread::sleep(Duration::from_millis(200));
    if calls.load(Ordering::SeqCst) != calls_after_drop {
        return Err("callback ran after the observer was dropped".into());
    }
    player.pause();
    Ok(())
}

fn labelled_queue_drop_waits_for_in_flight_callback() -> support::TestResult {
    in_flight_drop(
        Some("tests.avplayer.lifecycle.in-flight"),
        "test-lifecycle-labelled-in-flight",
    )
}

fn main_queue_drop_waits_for_in_flight_callback() -> support::TestResult {
    in_flight_drop(None, "test-lifecycle-main-in-flight")
}

fn self_drop(label: Option<&str>, stem: &str) -> support::TestResult {
    let player = support::player(stem)?;
    let slot: Arc<Mutex<Option<PeriodicTimeObserver>>> = Arc::new(Mutex::new(None));
    let closure_dropped = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::channel();
    let observer = {
        let slot = Arc::clone(&slot);
        let guard = DropFlag(Arc::clone(&closure_dropped));
        player.add_periodic_time_observer(Time::new(1, 20), label, move |_| {
            let _ = &guard;
            let taken = slot.lock().unwrap().take();
            if taken.is_some() {
                drop(taken);
                let _ = tx.send(());
            }
        })?
    };
    *slot.lock().unwrap() = Some(observer);
    player.play();
    rx.recv_timeout(Duration::from_secs(5))?;
    if !wait_until(Duration::from_secs(2), || {
        closure_dropped.load(Ordering::SeqCst)
    }) {
        return Err("the callback was never released after dropping itself".into());
    }
    player.pause();
    Ok(())
}

fn labelled_queue_observer_can_drop_itself() -> support::TestResult {
    self_drop(
        Some("tests.avplayer.lifecycle.self-drop"),
        "test-lifecycle-labelled-self-drop",
    )
}

fn main_queue_observer_can_drop_itself() -> support::TestResult {
    self_drop(None, "test-lifecycle-main-self-drop")
}

fn boundary_observer_can_drop_itself() -> support::TestResult {
    let player = support::player("test-lifecycle-boundary-self-drop")?;
    let slot: Arc<Mutex<Option<BoundaryTimeObserver>>> = Arc::new(Mutex::new(None));
    let closure_dropped = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::channel();
    let observer = {
        let slot = Arc::clone(&slot);
        let guard = DropFlag(Arc::clone(&closure_dropped));
        player.add_boundary_time_observer(
            &[Time::new(1, 10)],
            Some("tests.avplayer.lifecycle.boundary"),
            move || {
                let _ = &guard;
                let taken = slot.lock().unwrap().take();
                if taken.is_some() {
                    drop(taken);
                    let _ = tx.send(());
                }
            },
        )?
    };
    *slot.lock().unwrap() = Some(observer);
    player.play();
    rx.recv_timeout(Duration::from_secs(5))?;
    if !wait_until(Duration::from_secs(2), || {
        closure_dropped.load(Ordering::SeqCst)
    }) {
        return Err("the boundary callback was never released".into());
    }
    player.pause();
    Ok(())
}

fn status_observer_reports_ready_to_play() -> support::TestResult {
    let player = support::player("test-lifecycle-status")?;
    let (tx, rx) = mpsc::channel();
    let _observer = player.observe_status(move |event| {
        let _ = tx.send(event);
    })?;
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if let Ok(PlayerStatusEvent::StatusChanged {
            status: PlayerStatus::ReadyToPlay,
            ..
        }) = rx.recv_timeout(Duration::from_millis(100))
        {
            return Ok(());
        }
    }
    Err("status observer never reported ReadyToPlay".into())
}

fn run_scenarios() -> Vec<(&'static str, Result<(), String>)> {
    let scenarios: [(&str, Scenario); 6] = [
        (
            "labelled_queue_drop_waits_for_in_flight_callback",
            labelled_queue_drop_waits_for_in_flight_callback,
        ),
        (
            "main_queue_drop_waits_for_in_flight_callback",
            main_queue_drop_waits_for_in_flight_callback,
        ),
        (
            "labelled_queue_observer_can_drop_itself",
            labelled_queue_observer_can_drop_itself,
        ),
        (
            "main_queue_observer_can_drop_itself",
            main_queue_observer_can_drop_itself,
        ),
        (
            "boundary_observer_can_drop_itself",
            boundary_observer_can_drop_itself,
        ),
        (
            "status_observer_reports_ready_to_play",
            status_observer_reports_ready_to_play,
        ),
    ];
    scenarios
        .into_iter()
        .map(|(name, scenario)| (name, scenario().map_err(|error| error.to_string())))
        .collect()
}

fn main() -> ExitCode {
    let (done_tx, done_rx) = mpsc::channel();
    let worker = thread::spawn(move || {
        let results = run_scenarios();
        let _ = done_tx.send(());
        results
    });
    while matches!(done_rx.try_recv(), Err(TryRecvError::Empty)) {
        unsafe { CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.02, 0) };
    }
    let Ok(results) = worker.join() else {
        eprintln!("time observer lifecycle scenarios panicked");
        return ExitCode::FAILURE;
    };
    let mut failed = 0;
    for (name, result) in &results {
        match result {
            Ok(()) => println!("test {name} ... ok"),
            Err(error) => {
                failed += 1;
                println!("test {name} ... FAILED: {error}");
            }
        }
    }
    println!(
        "test result: {}. {} passed; {failed} failed",
        if failed == 0 { "ok" } else { "FAILED" },
        results.len() - failed
    );
    if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
