//! The eighth point can be dragged without losing the vertical line it is constrained to.
#[cfg(test)]
mod tests {
    use super::super::hand::Hand;
    use super::super::App;
    use qymcad_core::model::{EntityKind, SketchPoint};

    fn point(app: &App, id: u64) -> SketchPoint {
        *app.project.sketches[0].points.iter().find(|point| point.id == id).expect("the sketch point exists")
    }

    fn line_length(app: &App) -> f64 {
        let start = point(app, 110);
        let end = point(app, 172);
        (end.x - start.x).hypot(end.y - start.y)
    }

    fn snapshot(hand: &mut Hand<'_>, name: &str) {
        hand.frame(Vec::new());
        let png = super::super::color_image_to_png(&hand.snapshot()).expect("the sketch picture encodes as PNG");
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/point-move-investigation");
        std::fs::create_dir_all(&dir).expect("the picture directory exists");
        std::fs::write(dir.join(name), png).expect("the sketch picture is saved");
    }

    #[test]
    fn dragging_the_eighth_point_keeps_the_reported_vertical_line() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../project_before_moving.qcad");
        if !path.exists() {
            eprintln!("PASSED OVER: the private point-move document is absent");
            return;
        }
        let mut app = App::default();
        let mut hand = Hand::new(&mut app);
        hand.open_project(&path);
        let owner = hand.app.project.timeline.iter().find(|node| node.id == 4).and_then(|node| node.parent).expect("the reported sketch belongs to a part");
        let part = hand.app.project.components.iter().find(|part| part.id == owner).expect("the reported part exists");
        let part_name = qymcad_i18n::name(&part.name);
        assert!(hand.double_press_word(&part_name, egui::pos2(100.0, 300.0)), "the reported part is visible in the tree");
        let name = hand.app.project.sketches[0].name.clone();
        assert!(hand.double_press_word(&name, egui::pos2(100.0, 300.0)), "the reported sketch is visible in the tree");
        assert_eq!(hand.app.sketch_ses.editing, Some(4), "the tree double-click opened the reported sketch");
        hand.look2d((20.0, 6.0));
        let eighth = hand.app.project.sketches[0].points[7];
        assert_eq!(eighth.id, 17, "the eighth displayed point is the constrained corner");
        let before = line_length(hand.app);
        snapshot(&mut hand, "point-drag-before.png");

        hand.drag2d((eighth.x, eighth.y), (18.0272, 6.08297));

        snapshot(&mut hand, "point-drag-after.png");
        assert!(
            hand.app.project.sketches[0].entities.iter().any(|entity| entity.id == 173 && matches!(entity.kind, EntityKind::Line { a: 110, b: 172 })),
            "the vertical line entity remains in the sketch"
        );
        let length = line_length(hand.app);
        assert!(length > before * 0.9, "the vertical line collapsed from {before} mm to {length} mm");
        assert!(length < before * 1.1, "the vertical line grew from {before} mm to {length} mm while dragging its incident point");
        let moved = point(hand.app, 17);
        assert!((moved.x - 20.0).abs() < 1e-6, "the eighth point left its vertical line: {moved:?}");
        assert!((moved.y - eighth.y).abs() > 0.01, "the pointer drag did not move the eighth point: {moved:?}");
    }
}
