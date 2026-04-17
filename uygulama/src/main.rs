// Release'de Windows GUI subsystem — aksi halde linker `console` subsystem'i
// sececegi icin `KavisNet.exe` calistiginda arkada bos bir cmd penceresi
// kaliyor. Debug'da `console` kalmali ki `println!` cikti goruntulenebilsin;
// Zed de ayni pattern'i `crates/zed/src/main.rs:2` satirinda kullaniyor.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gpui::*;
use ortak_ikonlar::VarlikKaynagi;
use ortak_tema::{kurulum as tema_kur, temayi_izle};
use uygulama_kabugu::ana_pencere_ac;

// ── Actions ───────────────────────────────────────────────

actions!(app, [Quit]);

// ── Cekirdek kurulum ve ana pencere ───────────────────────

fn main() {
    gpui_platform::application()
        .with_assets(VarlikKaynagi)
        .run(|cx: &mut App| {
            // 1. Temayi kur: SistemGorunumu + TemaKaydi + aktif Tema global'leri
            tema_kur(cx);

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
