use gpui::*;
use ortak_tema::Tema;

pub struct SolMenu {
    genislik: Pixels,
}

impl SolMenu {
    pub fn new() -> Self {
        Self {
            genislik: px(0.0),
        }
    }

    pub fn baslangic_genisligi_ayarla(&mut self, tema: &Tema) {
        if self.genislik < tema.sol_panel_min_genislik {
            self.genislik = tema.sol_panel_min_genislik;
        }
    }

    pub fn render(&self, tema: &Tema) -> impl IntoElement {
        let mut base = div()
            .id("sol-panel")
            .w(self.genislik)
            .min_w(tema.sol_panel_min_genislik)
            .h_full()
            .flex_shrink_0();

        if tema.ust_sinir {
            base = base.pt(tema.ust_bar_yukseklik);
        }

        base
    }
}
