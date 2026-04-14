use gpui::*;
use ortak_tema::Tema;

// ── Pencere kontrol butonlari (Windows / Linux) ────────────

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum KontrolTipi {
    Kucult,
    Buyut,
    Kapat,
}

#[allow(dead_code)]
impl KontrolTipi {
    fn label(&self) -> &'static str {
        match self {
            Self::Kucult => "\u{2013}",
            Self::Buyut => "\u{25A1}",
            Self::Kapat => "\u{2715}",
        }
    }

    fn window_control(&self) -> WindowControlArea {
        match self {
            Self::Kucult => WindowControlArea::Min,
            Self::Buyut => WindowControlArea::Max,
            Self::Kapat => WindowControlArea::Close,
        }
    }
}

#[allow(dead_code)]
fn kontrol_butonu(tip: KontrolTipi, tema: &Tema) -> Stateful<Div> {
    let hover_bg = match tip {
        KontrolTipi::Kapat => tema.kontrol_kapat_hover,
        _ => tema.kontrol_hover,
    };
    let metin_rengi = tema.ust_bar_metin;

    let base = div()
        .id(SharedString::from(tip.label()))
        .flex()
        .items_center()
        .justify_center()
        .w(px(46.))
        .h_full()
        .text_color(metin_rengi)
        .text_size(px(13.))
        .hover(move |s| s.bg(hover_bg))
        .child(tip.label());

    #[cfg(target_os = "windows")]
    let base = base.window_control_area(tip.window_control());

    #[cfg(target_os = "linux")]
    let base = base
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            window.prevent_default();
            cx.stop_propagation();
        })
        .on_click(move |_, window, cx| match tip {
            KontrolTipi::Kucult => window.minimize_window(),
            KontrolTipi::Buyut => window.zoom_window(),
            KontrolTipi::Kapat => {
                if kapatma_istegi(window, cx) {
                    cx.quit();
                }
            }
        });

    base
}

fn pencere_kontrolleri(tema: &Tema) -> Stateful<Div> {
    #[cfg(target_os = "macos")]
    {
        let _ = tema;
        return div().id("window-controls");
    }

    #[cfg(not(target_os = "macos"))]
    {
        div()
            .id("window-controls")
            .flex()
            .flex_row()
            .items_center()
            .flex_shrink_0()
            .h_full()
            .child(kontrol_butonu(KontrolTipi::Kucult, tema))
            .child(kontrol_butonu(KontrolTipi::Buyut, tema))
            .child(kontrol_butonu(KontrolTipi::Kapat, tema))
    }
}

// ── Kapatma kontrolu ──────────────────────────────────────

pub fn kapatma_istegi(_window: &mut Window, _cx: &mut gpui::App) -> bool {
    true
}

// ── Ust bar ───────────────────────────────────────────────

pub struct UstBar;

impl UstBar {
    pub fn render(&self, tema: &Tema) -> impl IntoElement {
        div()
            .id("ust-bar")
            .w_full()
            .h(tema.ust_bar_yukseklik)
            .flex_shrink_0()
            .bg(tema.ust_bar_arka_plan)
            .border_b_1()
            .border_color(tema.ust_bar_ayirici)
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .pl(tema.ust_bar_sol_bosluk)
            .on_mouse_down(MouseButton::Left, |ev, window, _cx| {
                if ev.click_count == 2 {
                    if cfg!(target_os = "macos") {
                        window.titlebar_double_click();
                    } else {
                        window.zoom_window();
                    }
                }
            })
            .child(
                div()
                    .id("ust-bar-icerik")
                    .window_control_area(WindowControlArea::Drag)
                    .flex()
                    .flex_row()
                    .items_center()
                    .h_full()
                    .flex_1()
                    .child(
                        div()
                            .text_color(tema.ust_bar_metin)
                            .text_size(px(14.))
                            .child("Merhaba Dünya!"),
                    ),
            )
            .child(pencere_kontrolleri(tema))
    }
}

// ── Calisma yuzeyi ────────────────────────────────────────

pub struct CalismaYuzeyi {
    ust_bar: UstBar,
}

impl CalismaYuzeyi {
    pub fn new() -> Self {
        Self { ust_bar: UstBar }
    }

    pub fn render(&self, tema: &Tema) -> impl IntoElement {
        let kavis = tema.pencere_kavis;

        div()
            .id("calisma-yuzeyi")
            .flex_1()
            .flex()
            .flex_col()
            .bg(tema.yuzey_1)
            .rounded_tr(kavis)
            .rounded_br(kavis)
            .overflow_hidden()
            .border_l_1()
            .border_color(tema.kenarlik)
            .child(self.ust_bar.render(tema))
            .child(div().id("icerik").flex_1())
    }
}
