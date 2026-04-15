use gpui::*;
use ortak_tema::{Tema, temayi_izle};
use uygulama_kabugu::ana_pencere_ac;

// ── Actions ───────────────────────────────────────────────

actions!(app, [Quit]);

// ── Cekirdek kurulum ve ana pencere ───────────────────────

fn main() {
    let tema = Tema::yukle();

    Application::new().run(move |cx| {
        // 1. Temayi kur
        cx.set_global(tema);

        // 2. Aksiyonlari ve menuleri ayarla
        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.set_menus(vec![Menu {
            name: "KavisNet".into(),
            items: vec![MenuItem::action("Quit", Quit)],
        }]);

        // 3. Tema izleyiciyi baslat
        temayi_izle(cx);

        // 4. Ana pencereyi ac
        ana_pencere_ac(cx);
    });
}
