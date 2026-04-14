use gpui::*;
use ortak_tema::{tema_dosya_yolu, Tema};
use sol_menu::SolMenu;
use uygulama_kabugu::{kapatma_istegi, CalismaYuzeyi};
use notify::{Watcher, RecursiveMode};
use std::sync::mpsc::channel;
use std::time::Duration;

// ── Actions ───────────────────────────────────────────────

actions!(app, [Quit]);

// ── App ───────────────────────────────────────────────────

struct App {
    sol_menu: SolMenu,
    calisma_yuzeyi: CalismaYuzeyi,
}

impl Render for App {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tema = cx.global::<Tema>();
        let kavis = tema.pencere_kavis;

        div()
            .size_full()
            .flex()
            .flex_row()
            .bg(tema.pencere_arka_plan)
            .rounded(kavis)
            .overflow_hidden()
            .child(self.sol_menu.render(tema))
            .child(self.calisma_yuzeyi.render(tema))
    }
}

// ── Cekirdek kurulum ve ana pencere ───────────────────────

fn main() {
    let tema = Tema::yukle();

    Application::new().run(move |cx| {
        cx.set_global(tema);
        
        cx.on_action(|_: &Quit, cx| {
            cx.quit();
        });

        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);

        cx.set_menus(vec![Menu {
            name: "gpui_app".into(),
            items: vec![MenuItem::action("Quit", Quit)],
        }]);

        // ── Tema İzleyici (Async/Non-blocking) ──
        let yol = tema_dosya_yolu();
        cx.spawn(async move |mut cx| {
            let (tx, rx) = channel();
            // Watcher'i closure icinde tutarak drop olmasini engelliyoruz
            let mut _watcher = notify::recommended_watcher(move |res| {
                if let Ok(_) = res {
                    let _ = tx.send(());
                }
            }).expect("Tema izleyici başlatılamadı");

            if let Some(parent) = yol.parent() {
                let _ = _watcher.watch(parent, RecursiveMode::NonRecursive);
            }

            loop {
                // try_recv ile kontrol et, blocking recv() kullanma
                let mut degisiklik_var = false;
                while let Ok(_) = rx.try_recv() {
                    degisiklik_var = true;
                }

                if degisiklik_var {
                    // Dosya yazma islemi icin kisa bir bekleme (async yield)
                    Timer::after(Duration::from_millis(100)).await;
                    
                    let yeni_yol = yol.clone();
                    let _ = cx.update(|cx| {
                        if let Some(yeni_tema) = Tema::kontrol_et_ve_yukle(&yeni_yol) {
                            cx.set_global(yeni_tema);
                        }
                    });
                }

                // CPU tukkanmasini onlemek ve diger tasklara izin vermek icin yield et
                Timer::after(Duration::from_millis(250)).await;
            }
        }).detach();

        // ── Ana Pencere ──
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
                        sol_menu: SolMenu::new(),
                        calisma_yuzeyi: CalismaYuzeyi::new(),
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
