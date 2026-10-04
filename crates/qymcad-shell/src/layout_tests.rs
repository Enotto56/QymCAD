use super::{Fills, Place, Shell, Slot};
use std::collections::BTreeMap;

#[derive(Default)]
struct Bounds {
    panels: BTreeMap<&'static str, egui::Rect>,
}

impl Fills for Bounds {
    fn live(&self, _key: &'static str) -> bool {
        true
    }

    fn fill(&mut self, key: &'static str, ui: &mut egui::Ui) {
        self.panels.insert(key, ui.max_rect());
        ui.set_min_size(ui.available_size());
        ui.allocate_space(egui::vec2(32.0, 32.0));
    }

    fn bar_frame(&self) -> egui::Frame {
        egui::Frame::default()
    }
}

fn registered() -> Shell {
    let mut shell = Shell::default();
    shell.put(Place::new("alpha", Slot::Left).sized(240.0, true));
    shell.put(Place::new("omega", Slot::Centre));
    shell
}

fn frame(ctx: &egui::Context, shell: &Shell) -> Bounds {
    let mut bounds = Bounds::default();
    let input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 800.0))), ..Default::default() };
    let _ = ctx.run_ui(input, |ui| {
        for slot in Slot::ORDER {
            shell.run_slot(slot, ui, &mut bounds);
        }
    });
    bounds
}

#[test]
fn resetting_after_a_horizontal_move_restores_the_side_width() {
    for horizontal in [Slot::Top, Slot::Bottom] {
        let ctx = egui::Context::default();
        let mut shell = registered();
        let initial = frame(&ctx, &shell);
        shell.move_to("alpha", horizontal);
        let _ = frame(&ctx, &shell);
        shell.reset();
        let restored = frame(&ctx, &shell);
        assert!(
            (restored.panels["alpha"].width() - initial.panels["alpha"].width()).abs() < 1.0,
            "reset after {horizontal:?} must restore the side width: initial {:?}, restored {:?}",
            initial.panels["alpha"],
            restored.panels["alpha"]
        );
        assert!(restored.panels["omega"].width() > 900.0, "the centre must remain usable after reset");
    }
}

#[test]
fn moving_to_another_axis_uses_that_slots_size() {
    for slot in [Slot::Top, Slot::Bottom, Slot::Left, Slot::Right] {
        let expected_ctx = egui::Context::default();
        let mut expected_shell = registered();
        expected_shell.move_to("alpha", slot);
        let expected = frame(&expected_ctx, &expected_shell);

        let ctx = egui::Context::default();
        let mut shell = registered();
        shell.move_to("alpha", if matches!(slot, Slot::Top | Slot::Bottom) { Slot::Left } else { Slot::Top });
        let _ = frame(&ctx, &shell);
        shell.move_to("alpha", slot);
        let moved = frame(&ctx, &shell);
        assert!(
            (moved.panels["alpha"].size() - expected.panels["alpha"].size()).length() < 1.0,
            "moving to {slot:?} must use its own dimensions: expected {:?}, moved {:?}",
            expected.panels["alpha"],
            moved.panels["alpha"]
        );
        assert!(moved.panels["omega"].width() > 900.0 && moved.panels["omega"].height() > 500.0, "moving to {slot:?} must leave room in the centre");
    }
}

#[test]
fn a_saved_full_width_panel_does_not_poison_the_default_layout() {
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
        egui::Panel::top("alpha").exact_size(240.0).show(ui, |_| {});
    });
    let recovered = frame(&ctx, &registered());
    assert!(recovered.panels["alpha"].width() < 250.0, "a saved horizontal rectangle must not become the side width: {:?}", recovered.panels["alpha"]);
    assert!(recovered.panels["omega"].width() > 900.0, "the default layout must recover its centre with existing UI memory");
}
