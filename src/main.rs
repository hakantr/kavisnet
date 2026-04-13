#[allow(unused_imports)]
use gpui::{prelude::FluentBuilder as _, *};

const UST_BAR_YÜKSEKLİĞİ: Pixels = px(40.);

// --- Cam efekti algilama ---

/// Mevcut platformda compositor blur destegi olup olmadigini tespit eder.
/// Destekleniyorsa `Blurred`, desteklenmiyorsa `Transparent` doner.
fn cam_efekti_destegi() -> WindowBackgroundAppearance {
    // macOS: NSVisualEffectView ile tam destek
    #[cfg(target_os = "macos")]
    {
        return WindowBackgroundAppearance::Blurred;
    }

    // Windows: DWM Acrylic ile tam destek
    #[cfg(target_os = "windows")]
    {
        return WindowBackgroundAppearance::Blurred;
    }

    // Linux: Compositor ve oturum tipine bagli
    #[cfg(target_os = "linux")]
    {
        let oturum_tipi = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
        let masaustu = std::env::var("XDG_CURRENT_DESKTOP")
            .unwrap_or_default()
            .to_uppercase();

        match oturum_tipi.as_str() {
            "wayland" if masaustu.contains("KDE") => {
                // KDE Plasma: org_kde_kwin_blur protokolu ile destek var
                WindowBackgroundAppearance::Blurred
            }
            _ => {
                // X11, GNOME Wayland, Sway vs: GPUI blur destegi yok
                // Transparent ile yari saydam cam gorunumu sagliyoruz
                WindowBackgroundAppearance::Transparent
            }
        }
    }
}

/// Cam efekti durumuna gore arka plan rengini dondurur.
/// Blur varsa daha seffaf, yoksa daha opak renk kullanir.
fn cam_arka_plan(gorunum: WindowBackgroundAppearance) -> Hsla {
    match gorunum {
        // Blur aktif: compositor arkaplani bulaniklastirir, daha seffaf olabiliriz
        WindowBackgroundAppearance::Blurred => hsla(0.4, 0.6, 0.2, 0.4),
        // Blur yok: daha opak yaparak okunurlugu koruyoruz
        _ => hsla(0.4, 0.6, 0.2, 0.8),
    }
}

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

    fn hover_bg(&self) -> Hsla {
        match self {
            Self::Kapat => hsla(0.0, 0.7, 0.45, 1.0),
            _ => hsla(0.0, 0.0, 1.0, 0.1),
        }
    }
}

#[allow(dead_code)]
fn kontrol_butonu(tip: KontrolTipi) -> Stateful<Div> {
    let base = div()
        .id(SharedString::from(tip.label()))
        .flex()
        .items_center()
        .justify_center()
        .w(px(46.))
        .h_full()
        .text_color(rgb(0xCDD6F4))
        .text_size(px(13.))
        .hover(move |s| s.bg(tip.hover_bg()))
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

fn pencere_kontrolleri() -> Stateful<Div> {
    #[cfg(target_os = "macos")]
    {
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
            .child(kontrol_butonu(KontrolTipi::Kucult))
            .child(kontrol_butonu(KontrolTipi::Buyut))
            .child(kontrol_butonu(KontrolTipi::Kapat))
    }
}

// --- Ust bar ---

struct UstBar;

impl UstBar {
    fn render(&self, _window: &Window, _cx: &Context<App>) -> impl IntoElement {
        div()
            .id("ust-bar")
            .w_full()
            .h(UST_BAR_YÜKSEKLİĞİ)
            .flex_shrink_0()
            .bg(transparent_black())
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
                            .text_color(rgb(0xCDD6F4))
                            .text_size(px(14.))
                            .child("Merhaba Dünya!"),
                    ),
            )
            .child(pencere_kontrolleri())
    }
}

// --- Kapatma kontrolu ---

/// Pencere ve uygulama kapatma isteklerini yoneten fonksiyon.
/// `true` donerse kapatmaya izin verir, `false` donerse engeller.
fn kapatma_istegi(_window: &mut Window, _cx: &mut gpui::App) -> bool {
    // TODO: Ileride burada kontroller yapilabilir.
    // Ornegin: kaydedilmemis degisiklik varsa kullaniciya sor.
    true
}

// --- App ---

struct App {
    top_bar: UstBar,
    cam_gorunum: WindowBackgroundAppearance,
}

impl Render for App {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex_col()
            .bg(cam_arka_plan(self.cam_gorunum))
            .child(self.top_bar.render(window, cx))
    }
}

fn main() {
    Application::new().run(move |cx| {
        // Quit action: Cmd+Q (macOS) / Ctrl+Q (Windows/Linux)
        cx.on_action(|_: &Quit, cx| {
            cx.quit();
        });

        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);

        // macOS app menusu — Cmd+Q icin gerekli
        cx.set_menus(vec![Menu {
            name: "gpui_app".into(),
            items: vec![MenuItem::action("Quit", Quit)],
        }]);

        cx.spawn(async move |cx| {
            let cam_gorunum = cam_efekti_destegi();

            let options = WindowOptions {
                titlebar: Some(TitlebarOptions {
                    appears_transparent: true,
                    traffic_light_position: Some(point(px(8.), px(12.))),
                    ..Default::default()
                }),
                window_background: cam_gorunum,
                is_resizable: true,
                ..Default::default()
            };

            let window_handle = cx
                .open_window(options, |_window, cx| {
                    cx.new(|_cx| App {
                        top_bar: UstBar,
                        cam_gorunum,
                    })
                })
                .expect("Pencere açılamadı");

            // Pencere kapatilmak istendiginde kapatma_istegi fonksiyonunu cagir
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
