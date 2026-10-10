use super::conflict_rail_visible;

#[test]
fn narrow_window_releases_code_width_without_changing_preferences() {
    assert!(!conflict_rail_visible(860.0));
    assert!(!conflict_rail_visible(1099.0));
    assert!(conflict_rail_visible(1100.0));
    assert!(conflict_rail_visible(1600.0));
}
