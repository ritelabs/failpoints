// Copyright 2019 TiKV Project Authors. Licensed under Apache-2.0.
// Copyright 2021 RiteLabs. Licensed under Apache-2.0 OR MIT.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::*;
use std::time::*;
use std::*;

use failpoints::failpoint;

#[test]
fn test_off() {
    let f = || {
        failpoint!("off", |_| 2);
        0
    };
    assert_eq!(f(), 0);

    failpoints::cfg("off", "off").unwrap();
    assert_eq!(f(), 0);
}

#[test]
#[cfg_attr(not(feature = "failpoints"), ignore)]
fn test_return() {
    let f = || {
        failpoint!("return", |s: Option<String>| s
            .map_or(2, |s| s.parse().unwrap()));
        0
    };
    assert_eq!(f(), 0);

    failpoints::cfg("return", "return(1000)").unwrap();
    assert_eq!(f(), 1000);

    failpoints::cfg("return", "return").unwrap();
    assert_eq!(f(), 2);
}

#[test]
#[cfg_attr(not(feature = "failpoints"), ignore)]
fn test_sleep() {
    let f = || {
        failpoint!("sleep");
    };
    let timer = Instant::now();
    f();
    assert!(timer.elapsed() < Duration::from_millis(1000));

    let timer = Instant::now();
    failpoints::cfg("sleep", "sleep(1000)").unwrap();
    f();
    assert!(timer.elapsed() > Duration::from_millis(1000));
}

#[test]
#[should_panic]
#[cfg_attr(not(feature = "failpoints"), ignore)]
fn test_panic() {
    let f = || {
        failpoint!("panic");
    };
    failpoints::cfg("panic", "panic(msg)").unwrap();
    f();
}

#[test]
#[cfg_attr(not(feature = "failpoints"), ignore)]
fn test_pause() {
    let f = || {
        failpoint!("pause");
    };
    f();

    failpoints::cfg("pause", "pause").unwrap();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        // pause
        f();
        tx.send(()).unwrap();
        // woken up by new order pause, and then pause again.
        f();
        tx.send(()).unwrap();
        // woken up by remove, and then quit immediately.
        f();
        tx.send(()).unwrap();
    });

    assert!(rx.recv_timeout(Duration::from_millis(500)).is_err());
    failpoints::cfg("pause", "pause").unwrap();
    rx.recv_timeout(Duration::from_millis(500)).unwrap();

    assert!(rx.recv_timeout(Duration::from_millis(500)).is_err());
    failpoints::remove("pause");
    rx.recv_timeout(Duration::from_millis(500)).unwrap();

    rx.recv_timeout(Duration::from_millis(500)).unwrap();
}

#[test]
fn test_yield() {
    let f = || {
        failpoint!("yield");
    };
    failpoints::cfg("test", "yield").unwrap();
    f();
}

#[test]
#[cfg_attr(not(feature = "failpoints"), ignore)]
fn test_callback() {
    let f1 = || {
        failpoint!("cb");
    };
    let f2 = || {
        failpoint!("cb");
    };

    let counter = Arc::new(AtomicUsize::new(0));
    let counter2 = counter.clone();
    failpoints::cfg_callback("cb", move || {
        counter2.fetch_add(1, Ordering::SeqCst);
    })
    .unwrap();
    f1();
    f2();
    assert_eq!(2, counter.load(Ordering::SeqCst));
}

#[test]
#[cfg_attr(not(feature = "failpoints"), ignore)]
fn test_delay() {
    let f = || failpoint!("delay");
    let timer = Instant::now();
    failpoints::cfg("delay", "delay(1000)").unwrap();
    f();
    assert!(timer.elapsed() > Duration::from_millis(1000));
}

#[test]
#[cfg_attr(not(feature = "failpoints"), ignore)]
fn test_freq_and_count() {
    let f = || {
        failpoint!("freq_and_count", |s: Option<String>| s
            .map_or(2, |s| s.parse().unwrap()));
        0
    };
    failpoints::cfg(
        "freq_and_count",
        "50%50*return(1)->50%50*return(-1)->50*return",
    )
    .unwrap();
    let mut sum = 0;
    for _ in 0..5000 {
        let res = f();
        sum += res;
    }
    assert_eq!(sum, 100);
}

#[test]
#[cfg_attr(not(feature = "failpoints"), ignore)]
fn test_condition() {
    let f = |_enabled| {
        failpoint!("condition", _enabled, |_| 2);
        0
    };
    assert_eq!(f(false), 0);

    failpoints::cfg("condition", "return").unwrap();
    assert_eq!(f(false), 0);

    assert_eq!(f(true), 2);
}

#[test]
fn test_list() {
    assert!(!failpoints::list().contains(&("list".to_string(), "off".to_string())));
    failpoints::cfg("list", "off").unwrap();
    assert!(failpoints::list().contains(&("list".to_string(), "off".to_string())));
    failpoints::cfg("list", "return").unwrap();
    assert!(failpoints::list().contains(&("list".to_string(), "return".to_string())));
}
