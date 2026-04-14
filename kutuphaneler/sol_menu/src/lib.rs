use gpui::*;
use ortak_tema::Tema;

pub struct SolMenu;

impl SolMenu {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, tema: &Tema) -> impl IntoElement {
        let mut base = div()
            .id("sol-panel")
            .w(tema.sol_panel_genislik)
            .h_full()
            .flex_shrink_0()
            .flex()
            .flex_col();

        #[cfg(not(target_os = "macos"))]
        {
            base = base.child(
                div()
                    .id("sol-panel-surukle")
                    .w_full()
                    .h(px(40.))
                    .flex_shrink_0()
                    .window_control_area(WindowControlArea::Drag)
                    .on_mouse_down(MouseButton::Left, |ev, window, _cx| {
                        if ev.click_count == 2 {
                            window.zoom_window();
                        }
                    }),
            );
        }

        base
    }
}
