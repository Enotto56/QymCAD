//! A point constrained to a vertical line cannot escape by collapsing that line.
use qymcad_core::model::{Constraint, SketchPoint};
use qymcad_core::solver;

struct ReferenceSketch {
    points: Vec<SketchPoint>,
    constraints: Vec<Constraint>,
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

fn at(sketch: &ReferenceSketch, id: u64) -> SketchPoint {
    *sketch.points.iter().find(|point| point.id == id).expect("the point exists")
}

fn line_length(sketch: &ReferenceSketch) -> f64 {
    let start = at(sketch, 110);
    let end = at(sketch, 172);
    (end.x - start.x).hypot(end.y - start.y)
}

fn drag_and_release(sketch: &mut ReferenceSketch, id: u64, target: (f64, f64)) -> f64 {
    let from = at(sketch, id);
    for step in 1..=4 {
        let fraction = f64::from(step) / 4.0;
        solver::solve_full_iter(&mut sketch.points, &mut [], &sketch.constraints, Some((id, from.x + (target.0 - from.x) * fraction, from.y + (target.1 - from.y) * fraction)), 40);
    }
    solver::solve(&mut sketch.points, &sketch.constraints)
}

#[test]
fn pulling_a_point_off_a_vertical_line_keeps_the_line_whole() {
    let mut sketch = vertical_line_on_a_midpoint();
    let before = line_length(&sketch);

    let residual = drag_and_release(&mut sketch, 17, (18.0272, 6.08297));

    let length = line_length(&sketch);
    assert!(length > before * 0.9, "the vertical line collapsed from {before} mm to {length} mm");
    assert!(length < before * 1.1, "the vertical line grew from {before} mm to {length} mm during a blocked sideways drag");
    assert!(residual < 1e-6, "the released sketch has residual {residual}");
    let point = at(&sketch, 17);
    assert!((point.x - 20.0).abs() < 1e-6, "the point left the vertical line: {point:?}");
}

#[test]
fn a_point_can_slide_along_its_vertical_line() {
    let mut sketch = vertical_line_on_a_midpoint();

    let residual = drag_and_release(&mut sketch, 17, (20.0, 16.0));

    let point = at(&sketch, 17);
    assert!(residual < 1e-6, "the released sketch has residual {residual}");
    assert!((point.x - 20.0).hypot(point.y - 16.0) < 1e-2, "the point did not slide to (20, 16): {point:?}");
    assert!(line_length(&sketch) > 80.0, "sliding the point collapsed its line");
}

#[test]
fn dragging_the_reference_endpoint_can_resize_the_line() {
    let mut sketch = vertical_line_on_a_midpoint();
    let before = line_length(&sketch);

    let residual = drag_and_release(&mut sketch, 172, (20.0, 73.0));

    let end = at(&sketch, 172);
    assert!(residual < 1e-6, "the released sketch has residual {residual}");
    assert!((end.x - 20.0).hypot(end.y - 73.0) < 1e-2, "the endpoint did not follow the pointer: {end:?}");
    assert!(line_length(&sketch) > before + 15.0, "the endpoint drag did not lengthen the line");
}

#[test]
fn a_distance_dimension_can_resize_the_reference_line() {
    let mut sketch = vertical_line_on_a_midpoint();
    sketch.constraints.push(Constraint::Distance { a: 110, b: 172, d: 50.0, off: 0.0, expr: String::new(), driven: false, axis: 0, at: None });

    let residual = solver::solve(&mut sketch.points, &sketch.constraints);

    let length = line_length(&sketch);
    assert!(residual < 1e-6, "the dimensioned sketch has residual {residual}");
    assert!((length - 50.0).abs() < 1e-6, "the 50 mm dimension left the line {length} mm long");
}

#[test]
fn a_dimension_on_a_connected_line_can_move_the_midpoint() {
    let mut sketch = vertical_line_on_a_midpoint();
    sketch.constraints.push(Constraint::Distance { a: 9, b: 6, d: 60.0, off: 0.0, expr: String::new(), driven: false, axis: 0, at: None });

    let residual = solver::solve(&mut sketch.points, &sketch.constraints);

    let start = at(&sketch, 110);
    assert!(residual < 1e-6, "the connected dimension has residual {residual}");
    assert!((start.x - 20.0).hypot(start.y + 60.0) < 1e-6, "the midpoint did not follow the 60 mm dimension: {start:?}");
    assert!(line_length(&sketch) > 80.0, "the connected dimension collapsed the reference line");
}
