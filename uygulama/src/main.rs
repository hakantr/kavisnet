use gpui::*;
use ortak_ikonlar::VarlikKaynagi;
use ortak_tema::{Tema, temayi_izle};
use uygulama_kabugu::ana_pencere_ac;

// ── Actions ───────────────────────────────────────────────

actions!(app, [Quit]);

// ── Cekirdek kurulum ve ana pencere ───────────────────────

fn main() {
    let tema = Tema::yukle();

    gpui_platform::application()
        .with_assets(VarlikKaynagi)
        .run(move |cx: &mut App| {
            // 1. Temayi kur
            cx.set_global(tema);

            // 2. Aksiyonlari ve menuleri ayarla
            cx.on_action(|_: &Quit, cx: &mut App| cx.quit());
            cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
            cx.set_menus(vec![Menu {
                name: "KavisNet".into(),
                items: vec![MenuItem::action("Quit", Quit)],
                disabled: false,
            }]);

            // 3. Tema izleyiciyi baslat
            temayi_izle(cx);

            // 4. Ana pencereyi ac
            ana_pencere_ac(cx);
        });
}
