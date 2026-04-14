use gpui::*;
use ortak_tema::Tema;

pub struct SolMenu;

impl SolMenu {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, tema: &Tema) -> impl IntoElement {
        div()
            .id("sol-panel")
            .w(tema.sol_panel_genislik)
            .h_full()
            .flex_shrink_0()
    }
}
