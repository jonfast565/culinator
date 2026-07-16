#[test]
fn module_is_wired() {}

#[test]
fn schedule_options_have_default_duration() {
    assert_eq!(
        super::ScheduleOptions::default().default_duration_seconds,
        300
    );
}
