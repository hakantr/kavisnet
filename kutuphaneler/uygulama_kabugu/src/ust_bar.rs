use gpui::prelude::*;
use gpui::*;
use ortak_tema::{AktifTema, Tema};

use crate::kontroller::{pencere_kontrolleri_taraf, KontrolTarafi};

/// Üst bar: başlık alanı + CSD kontrol butonları + pencere sürükleme.
///
/// Zed'in `PlatformTitleBar` yaklaşımıyla uyumlu bir Entity; sürüklemeyi
/// doğrudan `on_mouse_down`'da değil, bayrak kurup `on_mouse_move`'da
/// başlatır. Bu ayrım Wayland/X11'de tek tıkla kayan pencere hatasını
/// ve double-click zoom ile çakışmayı önler.
pub struct UstBar {
    /// Sol tuş basılı, henüz hareket yok. Bir sonraki mouse-move olayında
    /// window.start_window_move() çağrılır (yalnızca Linux'ta).
    should_move: bool,
}

impl UstBar {
    pub fn yeni(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<Tema>(|_this, cx| cx.notify()).detach();
        Self { should_move: false }
    }
}

impl Render for UstBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tema = cx.tema();
        let sol_kontroller = pencere_kontrolleri_taraf(KontrolTarafi::Sol, window, tema);
        let sag_kontroller = pencere_kontrolleri_taraf(KontrolTarafi::Sag, window, tema);

        let mut kok = div()
            .id("ust-bar")
            .w_full()
            .h(tema.yerlesim.ust_bar_yukseklik)
            .flex_shrink_0()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .pl(tema.yerlesim.ust_bar_sol_bosluk)
            .window_control_area(WindowControlArea::Drag);

        if !tema.yerlesim.ust_sinir {
            kok = kok.border_b_1().border_color(tema.renkler.baslik_cubugu_ayirici);
        }

        kok.on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, ev: &MouseDownEvent, window, _cx| {
                if ev.click_count == 2 {
                    #[cfg(target_os = "macos")]
                    window.titlebar_double_click();
                    #[cfg(not(target_os = "macos"))]
                    window.zoom_window();
                    let _ = window;
                } else {
                    this.should_move = true;
                }
            }),
        )
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(|this, _ev, _window, _cx| {
                this.should_move = false;
            }),
        )
        .on_mouse_move(cx.listener(|this, _ev, window, _cx| {
            if !this.should_move {
                return;
            }
            this.should_move = false;
            #[cfg(target_os = "linux")]
            window.start_window_move();
            // TODO: Windows CSD drag — start_window_move() Windows'ta
            // desteklendiğinde buraya cfg eklenmeli.
            let _ = window;
        }))
        .on_mouse_down(
            MouseButton::Right,
            cx.listener(|_this, ev: &MouseDownEvent, window, _cx| {
                let _ = (&ev, &window);
                #[cfg(target_os = "linux")]
                {
                    if matches!(window.window_decorations(), Decorations::Client { .. })
                        && window.window_controls().window_menu
                    {
                        window.show_window_menu(ev.position);
                    }
                }
            }),
        )
        .child(sol_kontroller)
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
                        .text_color(tema.renkler.metin)
                        .text_size(px(14.))
                        .child("Merhaba Dünya!"),
                ),
        )
        .child(sag_kontroller)
    }
}
