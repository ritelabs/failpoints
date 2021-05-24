use failpoints::{failpoint, FailScenario};

fn do_fallible_work() {
    failpoint!("read-dir");
    let _dir: Vec<_> = std::fs::read_dir(".").unwrap().collect();
    // ... do some work on the directory ...
}

fn main() {
    let scenario = FailScenario::setup();
    do_fallible_work();
    scenario.teardown();
    println!("done");
}
