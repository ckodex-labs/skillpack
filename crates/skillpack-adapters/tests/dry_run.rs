use skillpack_adapters::cli::dry_run::DryRun;

#[test]
fn dry_run_true_skips_side_effect() {
    let mut counter = 0i32;
    let dr = DryRun(true);
    dr.perform("increment counter", || counter += 1);
    assert_eq!(counter, 0, "DryRun(true) should skip the side effect");
}

#[test]
fn dry_run_false_executes_side_effect() {
    let mut counter = 0i32;
    let dr = DryRun(false);
    dr.perform("increment counter", || counter += 1);
    assert_eq!(counter, 1, "DryRun(false) should execute the side effect");
}

#[test]
fn dry_run_false_returns_value() {
    let dr = DryRun(false);
    let result: i32 = dr.perform("compute", || 42);
    assert_eq!(result, 42);
}

#[test]
fn dry_run_true_returns_default() {
    let dr = DryRun(true);
    let result: i32 = dr.perform("compute", || 42);
    assert_eq!(result, i32::default()); // 0
}
