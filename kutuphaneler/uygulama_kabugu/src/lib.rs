use gpui::*;
use ortak_tema::Tema;
use sol_menu::SolMenu;

// ── Ana Panel (Uygulamanın Kök Bileşeni) ──────────────────

pub struct AnaPanel {
    pub ust_bar: UstBar,
    pub sol_menu: SolMenu,
    pub calisma_yuzeyi: CalismaYuzeyi,
}

impl AnaPanel {
    pub fn new() -> Self {
        Self {
            ust_bar: UstBar,
            sol_menu: SolMenu::new(),
            calisma_yuzeyi: CalismaYuzeyi::new(),
        }
    }
}

impl Render for AnaPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tema = cx.global::<Tema>();

        let icerik_satiri = div()
            .flex_1()
            .flex()
            .flex_row()
            .overflow_hidden()
            .child(self.sol_menu.render(tema))
            .child(self.calisma_yuzeyi.render(tema));

        let base = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(tema.pencere_arka_plan)
            .rounded(tema.pencere_kavis)
            .overflow_hidden();

        if tema.ust_sinir {
            base.relative().child(icerik_satiri).child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .child(self.ust_bar.render(tema)),
            )
        } else {
            base.child(self.ust_bar.render(tema)).child(icerik_satiri)
        }
    }
}

/// Uygulamanın ana penceresini açar ve yapılandırır.
pub fn ana_pencere_ac(cx: &mut App) {
    let tema = *cx.global::<Tema>();

    cx.spawn(async move |cx| {
        let options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                appears_transparent: true,
                traffic_light_position: Some(point(px(8.), px(12.))),
                ..Default::default()
            }),
            window_background: tema.pencere_gorunum,
            is_resizable: true,
            ..Default::default()
        };

        let window_handle = cx
            .open_window(options, |_window, cx| cx.new(|_cx| AnaPanel::new()))
            .expect("Pencere açılamadı");

        cx.update_window(window_handle.into(), |_root, window, cx| {
            window.on_window_should_close(cx, |window, cx| {
                if kapatma_istegi(window, cx) {
                    cx.quit();
                    true
                } else {
                    false
                }
            });
        })
        .ok();
    })
    .detach();
}

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
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .pl(tema.ust_bar_sol_bosluk)
            .window_control_area(WindowControlArea::Drag)
            .on_mouse_down(MouseButton::Left, |ev, window, _cx| {
                if ev.click_count == 2 {
                    #[cfg(target_os = "macos")]
                    window.titlebar_double_click();
                    #[cfg(not(target_os = "macos"))]
                    window.zoom_window();
                } else {
                    #[cfg(target_os = "linux")]
                    window.start_window_move();
                }
            })
            .child(
                div()
                    .id("ust-bar-icerik")
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

pub struct CalismaYuzeyi;

impl CalismaYuzeyi {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, tema: &Tema) -> impl IntoElement {
        let mut base = div()
            .id("calisma-yuzeyi")
            .flex_1()
            .flex()
            .flex_col()
            .bg(tema.yuzey_1)
            .overflow_hidden()
            .border_l_1()
            .border_color(tema.kenarlik)
            .rounded_tr(tema.pencere_kavis)
            .rounded_br(tema.pencere_kavis);

        if tema.calisma_yuzeyi_kavisli_mi {
            base = base.rounded_tl(tema.calisma_yuzeyi_kavis);
        }

        base.child(div().id("icerik").flex_1())
    }
}
