use super::*;

#[test]
fn moved_tab_window_cascades_with_source_size_on_its_display() {
    let source = RectF::new(vec2f(-1400., 100.), vec2f(1000., 700.));
    let display = RectF::new(vec2f(-1600., 0.), vec2f(1600., 1000.));
    assert_eq!(
        cascade_tab_window_bounds(source, display),
        RectF::new(vec2f(-1376., 124.), source.size()),
    );
}

#[test]
fn moved_tab_window_stays_within_display_edges() {
    let source = RectF::new(vec2f(900., 500.), vec2f(1000., 600.));
    let display = RectF::new(vec2f(0., 0.), vec2f(1920., 1080.));
    assert_eq!(
        cascade_tab_window_bounds(source, display),
        RectF::new(vec2f(920., 480.), source.size()),
    );
}

#[test]
fn moved_tab_window_fits_source_larger_than_display() {
    let source = RectF::new(vec2f(1800., 0.), vec2f(1800., 1200.));
    let display = RectF::new(vec2f(1920., 100.), vec2f(1280., 800.));
    assert_eq!(cascade_tab_window_bounds(source, display), display);
}
