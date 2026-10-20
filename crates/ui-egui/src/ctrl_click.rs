//! ⌃-click is a right click on macOS, as in AppKit and Photoshop: it opens the Layers panel's
//! layer menu, a tab's menu, a text field's menu and the canvas tool menu. winit reports it as a
//! left press with the Control modifier, so [`as_right_click`] turns that press, and its release,
//! into the secondary button before egui sees them.
//!
//! Only a plain ⌃ press converts. ⌃ with ⌥ or ⌘ stays a left press for the shortcuts that use
//! it: ⌃⌥-drag brush resizing (`brush_resize.rs`), ⌘⌥⌃-click layer picking (`quick_pick.rs`) and
//! Liquify's ⌥⌃ (`liquify_ui.rs`).

use egui::{Event, Modifiers, PointerButton};

/// Rewrites a plain ⌃ primary press and its matching release as the secondary button.
/// `converting` remembers a converted press until its release, which may arrive in a later frame
/// and after ⌃ was let go.
pub(crate) fn as_right_click(converting: &mut bool, events: &mut [Event]) {
    for e in events {
        let Event::PointerButton { button, pressed, modifiers, .. } = e else { continue };
        if *button != PointerButton::Primary {
            continue;
        }
        let convert = if *pressed {
            *converting = is_plain_ctrl(*modifiers);
            *converting
        } else {
            std::mem::take(converting)
        };
        if convert {
            *button = PointerButton::Secondary;
        }
    }
}

fn is_plain_ctrl(m: Modifiers) -> bool {
    m.ctrl && !m.alt && !m.mac_cmd && !m.command
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    fn button(button: PointerButton, pressed: bool, modifiers: Modifiers) -> Event {
        Event::PointerButton { pos: pos2(10.0, 10.0), button, pressed, modifiers }
    }

    fn buttons(events: &[Event]) -> Vec<PointerButton> {
        events.iter().filter_map(|e| if let Event::PointerButton { button, .. } = e { Some(*button) } else { None }).collect()
    }

    #[test]
    fn ctrl_click_becomes_a_right_click_including_a_release_in_a_later_frame() {
        let mut converting = false;
        let mut press = [button(PointerButton::Primary, true, Modifiers::CTRL)];
        as_right_click(&mut converting, &mut press);
        // ⌃ released before the mouse button: the release still matches the press.
        let mut release = [button(PointerButton::Primary, false, Modifiers::NONE)];
        as_right_click(&mut converting, &mut release);
        assert_eq!(buttons(&press), [PointerButton::Secondary]);
        assert_eq!(buttons(&release), [PointerButton::Secondary]);
        assert!(!converting);
    }

    #[test]
    fn ctrl_with_option_or_command_and_plain_clicks_stay_left_clicks() {
        let mods = [Modifiers::NONE, Modifiers::SHIFT, Modifiers::CTRL | Modifiers::ALT, Modifiers::CTRL | Modifiers::MAC_CMD | Modifiers::ALT];
        for m in mods {
            let mut converting = false;
            let mut events = [button(PointerButton::Primary, true, m), button(PointerButton::Primary, false, m)];
            as_right_click(&mut converting, &mut events);
            assert_eq!(buttons(&events), [PointerButton::Primary, PointerButton::Primary], "{m:?}");
        }
    }

    #[test]
    fn a_ctrl_click_opens_a_context_menu() {
        let ctx = egui::Context::default();
        let mut converting = false;
        let mut opened = false;
        for frame in 0..3 {
            let mut events = vec![Event::PointerMoved(pos2(10.0, 10.0))];
            if frame == 1 {
                events.push(button(PointerButton::Primary, true, Modifiers::CTRL));
                events.push(button(PointerButton::Primary, false, Modifiers::CTRL));
            }
            as_right_click(&mut converting, &mut events);
            let input = egui::RawInput { events, modifiers: if frame == 1 { Modifiers::CTRL } else { Modifiers::NONE }, ..Default::default() };
            let _ = ctx.run_ui(input, |ui| {
                let resp = ui.add(egui::Label::new("Background").sense(egui::Sense::click()));
                resp.context_menu(|ui| {
                    opened = true;
                    ui.label("Duplicate Layer…");
                });
            });
        }
        assert!(opened);
    }
}
