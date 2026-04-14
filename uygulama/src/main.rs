use gpui::*;
use ortak_tema::Tema;
use sol_menu::SolMenu;
use uygulama_kabugu::{kapatma_istegi, CalismaYuzeyi};

// ── Actions ───────────────────────────────────────────────

actions!(app, [Quit]);

// ── App ───────────────────────────────────────────────────

struct App {
    tema: Tema,
    sol_menu: SolMenu,
    calisma_yuzeyi: CalismaYuzeyi,
}

impl Render for App {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let kavis = self.tema.pencere_kavis;

        div()
            .size_full()
            .flex()
            .flex_row()
            .bg(self.tema.pencere_arka_plan)
            .rounded(kavis)
            .overflow_hidden()
            .child(self.sol_menu.render(&self.tema))
            .child(self.calisma_yuzeyi.render(&self.tema))
    }
}

// ── Cekirdek kurulum ve ana pencere ───────────────────────

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
                        tema,
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
