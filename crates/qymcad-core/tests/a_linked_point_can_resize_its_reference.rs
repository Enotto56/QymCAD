//! Positional links release a reference line for resizing, while an axis dimension keeps its length free.
use qymcad_core::model::{Constraint, SketchPoint};
use qymcad_core::solver;

struct ReferenceSketch {
    points: Vec<SketchPoint>,
    constraints: Vec<Constraint>,
}

enum Handle {
    Coincident,
    Midpoint,
}

struct HandleLink {
    height: f64,
    constraint: Constraint,
}

fn linked_handle(handle: Handle) -> ReferenceSketch {
    let link = match handle {
        Handle::Coincident => HandleLink { height: 10.0, constraint: Constraint::Coincident { a: 2, b: 4 } },
        Handle::Midpoint => HandleLink { height: 5.0, constraint: Constraint::Midpoint { p: 4, a: 1, b: 2 } },
    };
    ReferenceSketch {
        points: vec![
            SketchPoint { id: 1, x: 0.0, y: 0.0 },
            SketchPoint { id: 2, x: 0.0, y: 10.0 },
            SketchPoint { id: 3, x: 0.0, y: 2.0 },
            SketchPoint { id: 4, x: 0.0, y: link.height },
        ],
        constraints: vec![Constraint::Fixed { p: 1 }, Constraint::Vertical { a: 1, b: 2 }, Constraint::PointOnLine { p: 3, a: 1, b: 2 }, link.constraint],
    }
}

fn vertical_line_on_a_midpoint() -> ReferenceSketch {
    ReferenceSketch {
        points: vec![
            SketchPoint { id: 5, x: 0.0, y: 0.0 },
            SketchPoint { id: 6, x: 0.0, y: -40.0 },
            SketchPoint { id: 7, x: 40.0, y: -40.0 },
            SketchPoint { id: 9, x: 0.0, y: 0.0 },
            SketchPoint { id: 110, x: 20.0, y: -40.0 },
            SketchPoint { id: 172, x: 20.0, y: 53.0 },
            SketchPoint { id: 17, x: 20.0, y: 6.15246 },
        ],
        constraints: vec![
            Constraint::Fixed { p: 5 },
            Constraint::Horizontal { a: 6, b: 7 },
            Constraint::Vertical { a: 6, b: 9 },
            Constraint::Midpoint { p: 110, a: 6, b: 7 },
            Constraint::Coincident { a: 9, b: 5 },
            Constraint::Distance { a: 6, b: 7, d: 40.0, off: 0.0, expr: String::new(), driven: false, axis: 0, at: None },
            Constraint::Vertical { a: 110, b: 172 },
            Constraint::PointOnLine { p: 17, a: 110, b: 172 },
        ],
    }
}

fn point(sketch: &ReferenceSketch, id: u64) -> SketchPoint {
    *sketch.points.iter().find(|point| point.id == id).expect("the reference point exists")
}

fn drag_and_release(sketch: &mut ReferenceSketch, id: u64, target: (f64, f64)) -> f64 {
    let from = point(sketch, id);
    for step in 1..=4 {
        let fraction = f64::from(step) / 4.0;
        solver::solve_full_iter(&mut sketch.points, &mut [], &sketch.constraints, Some((id, from.x + (target.0 - from.x) * fraction, from.y + (target.1 - from.y) * fraction)), 40);
    }
    solver::solve(&mut sketch.points, &sketch.constraints)
}

#[test]
fn a_coincident_handle_can_resize_the_reference_line() {
    let mut sketch = linked_handle(Handle::Coincident);

    let residual = drag_and_release(&mut sketch, 4, (0.0, 20.0));

    let handle = point(&sketch, 4);
    let end = point(&sketch, 2);
    assert!(residual < 1e-6, "the released sketch has residual {residual}");
    assert!(handle.x.hypot(handle.y - 20.0) < 1e-2, "the coincident handle did not reach (0, 20): {handle:?}");
    assert!(end.x.hypot(end.y - 20.0) < 1e-2, "the reference endpoint did not follow its coincident handle: {end:?}");
}

#[test]
fn a_midpoint_handle_can_resize_the_reference_line() {
    let mut sketch = linked_handle(Handle::Midpoint);

    let residual = drag_and_release(&mut sketch, 4, (0.0, 10.0));

    let handle = point(&sketch, 4);
    let end = point(&sketch, 2);
    assert!(residual < 1e-6, "the released sketch has residual {residual}");
    assert!(handle.x.hypot(handle.y - 10.0) < 1e-2, "the midpoint handle did not reach (0, 10): {handle:?}");
    assert!(end.x.hypot(end.y - 20.0) < 1e-2, "the reference endpoint did not follow its midpoint handle: {end:?}");
}

#[test]
fn a_transitively_coincident_handle_can_resize_the_reference_line() {
    let mut sketch = linked_handle(Handle::Coincident);
    sketch.points.extend([SketchPoint { id: 8, x: 0.0, y: 10.0 }, SketchPoint { id: 10, x: 0.0, y: 10.0 }]);
    sketch.constraints.extend([Constraint::Coincident { a: 4, b: 8 }, Constraint::Coincident { a: 8, b: 10 }]);

    let residual = drag_and_release(&mut sketch, 10, (0.0, 20.0));

    let handle = point(&sketch, 10);
    let end = point(&sketch, 2);
    assert!(residual < 1e-6, "the released sketch has residual {residual}");
    assert!(handle.x.hypot(handle.y - 20.0) < 1e-2, "the transitive handle did not reach (0, 20): {handle:?}");
    assert!(end.x.hypot(end.y - 20.0) < 1e-2, "the reference endpoint did not follow its transitive handle: {end:?}");
}

#[test]
fn a_coincident_alias_of_the_midpoint_can_resize_the_reference_line() {
    let mut sketch = linked_handle(Handle::Midpoint);
    sketch.points.push(SketchPoint { id: 8, x: 0.0, y: 5.0 });
    sketch.constraints.push(Constraint::Coincident { a: 4, b: 8 });

    let residual = drag_and_release(&mut sketch, 8, (0.0, 10.0));

    let handle = point(&sketch, 8);
    let midpoint = point(&sketch, 4);
    let end = point(&sketch, 2);
    assert!(residual < 1e-6, "the released sketch has residual {residual}");
    assert!(handle.x.hypot(handle.y - 10.0) < 1e-2, "the midpoint alias did not reach (0, 10): {handle:?}");
    assert!(midpoint.x.hypot(midpoint.y - 10.0) < 1e-2, "the midpoint did not follow its coincident alias: {midpoint:?}");
    assert!(end.x.hypot(end.y - 20.0) < 1e-2, "the reference endpoint did not follow its midpoint alias: {end:?}");
}

#[test]
fn a_horizontal_dimension_does_not_let_a_vertical_reference_collapse() {
    let mut sketch = vertical_line_on_a_midpoint();
    sketch.constraints.push(Constraint::Distance { a: 110, b: 172, d: 0.0, off: 0.0, expr: String::new(), driven: false, axis: 1, at: None });

    let residual = drag_and_release(&mut sketch, 17, (18.0272, 6.08297));

    let start = point(&sketch, 110);
    let end = point(&sketch, 172);
    let length = (end.x - start.x).hypot(end.y - start.y);
    let dragged = point(&sketch, 17);
    assert!(length > 83.7, "the 93 mm reference collapsed to {length} mm despite its horizontal dimension");
    assert!(length < 102.3, "the 93 mm reference grew to {length} mm despite its horizontal dimension");
    assert!(residual < 1e-6, "the released sketch has residual {residual}");
    assert!((dragged.x - 20.0).abs() < 1e-6, "the dragged point left the vertical reference: {dragged:?}");
}

#[test]
fn a_midpoint_of_another_line_does_not_release_the_vertical_reference() {
    let mut sketch = vertical_line_on_a_midpoint();
    let start = point(&sketch, 110);
    let dragged = point(&sketch, 17);
    sketch.points.push(SketchPoint { id: 200, x: 2.0 * dragged.x - start.x, y: 2.0 * dragged.y - start.y });
    sketch.constraints.push(Constraint::Midpoint { p: 17, a: 110, b: 200 });

    let residual = drag_and_release(&mut sketch, 17, (18.0272, 6.08297));

    let start = point(&sketch, 110);
    let end = point(&sketch, 172);
    let length = (end.x - start.x).hypot(end.y - start.y);
    let dragged = point(&sketch, 17);
    assert!(length > 83.7, "the 93 mm reference collapsed to {length} mm through an unrelated midpoint");
    assert!(length < 102.3, "the 93 mm reference grew to {length} mm through an unrelated midpoint");
    assert!(residual < 1e-6, "the released sketch has residual {residual}");
    assert!((dragged.x - 20.0).abs() < 1e-6, "the midpoint left the vertical reference: {dragged:?}");
}
