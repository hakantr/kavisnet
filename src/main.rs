#[allow(unused_imports)]
use gpui::{prelude::FluentBuilder as _, *};

mod tema;
use tema::Tema;

const UST_BAR_YÜKSEKLİĞİ: Pixels = px(40.);
const SOL_PANEL_GENİŞLİĞİ: Pixels = px(120.);

#[cfg(target_os = "macos")]
const UST_BAR_SOL_BOŞLUK: Pixels = px(80.);
#[cfg(not(target_os = "macos"))]
const UST_BAR_SOL_BOŞLUK: Pixels = px(12.);

// --- Quit action ---

actions!(app, [Quit]);

// --- Pencere kontrol butonlari (Windows / Linux) ---

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

// --- Ust bar ---

struct UstBar;

impl UstBar {
    fn render(&self, tema: &Tema, _window: &Window, _cx: &Context<App>) -> impl IntoElement {
        div()
            .id("ust-bar")
            .w_full()
            .h(UST_BAR_YÜKSEKLİĞİ)
            .flex_shrink_0()
            .bg(tema.ust_bar_arka_plan)
            .border_b_1()
            .border_color(tema.ust_bar_ayirici)
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .pl(UST_BAR_SOL_BOŞLUK)
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

// --- Kapatma kontrolu ---

fn kapatma_istegi(_window: &mut Window, _cx: &mut gpui::App) -> bool {
    true
}

// --- Calisma yuzeyi ---

struct CalismaYuzeyi {
    ust_bar: UstBar,
}

impl CalismaYuzeyi {
    fn render(&self, tema: &Tema, window: &Window, cx: &Context<App>) -> impl IntoElement {
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
            // Ust bar
            .child(self.ust_bar.render(tema, window, cx))
            // Icerik alani (ileride dolacak)
            .child(div().id("icerik").flex_1())
    }
}

// --- App ---

struct App {
    calisma_yuzeyi: CalismaYuzeyi,
    tema: Tema,
}

impl Render for App {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let kavis = self.tema.pencere_kavis;

        div()
            .size_full()
            .flex()
            .flex_row()
            .bg(self.tema.pencere_arka_plan)
            .rounded(kavis)
            .overflow_hidden()
            // Sol panel alani (120px sabit)
            .child(
                div()
                    .id("sol-panel")
                    .w(SOL_PANEL_GENİŞLİĞİ)
                    .h_full()
                    .flex_shrink_0(),
            )
            // Calisma yuzeyi (kalan alani doldurur)
            .child(self.calisma_yuzeyi.render(&self.tema, window, cx))
    }
}

fn main() {
    let tema = Tema::yukle();

    Application::new().run(move |cx| {
        cx.on_action(|_: &Quit, cx| {
            cx.quit();
        });

        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);

        cx.set_menus(vec![Menu {
            name: "gpui_app".into(),
            items: vec![MenuItem::action("Quit", Quit)],
        }]);

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
                .open_window(options, |_window, cx| {
                    cx.new(|_cx| App {
                        calisma_yuzeyi: CalismaYuzeyi { ust_bar: UstBar },
                        tema,
                    })
                })
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
    });
}
