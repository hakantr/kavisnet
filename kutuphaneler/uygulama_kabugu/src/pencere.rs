use gpui::*;
use ortak_tema::Tema;

use crate::ana_panel::AnaPanel;

/// Linux'ta pencere ↔ .desktop eşleşmesi için app_id. GPUI bu değeri
/// X11 WM_CLASS'a ve Wayland xdg-shell app_id'ye yazıyor.
pub const UYGULAMA_APP_ID: &str = "KavisNet";

/// Ana pencere için WindowOptions fabrikası.
///
/// Zed'in `build_window_options()` yaklaşımıyla uyumlu: seçenekleri tek
/// yerden döndürür, çağrı yerinde inline kalabalık bırakmaz.
pub fn pencere_secenekleri(tema: &Tema) -> WindowOptions {
    WindowOptions {
        titlebar: Some(TitlebarOptions {
            appears_transparent: true,
            traffic_light_position: Some(point(px(8.), px(12.))),
            ..Default::default()
        }),
        window_background: tema.pencere_gorunum,
        window_decorations: Some(WindowDecorations::Client),
        is_resizable: true,
        app_id: Some(UYGULAMA_APP_ID.to_string()),
        // TODO: window_bounds — kullanıcının son pencere boyutunu/konumunu
        // hatırlatmak istersek tema dosyasına kaydedip buradan yüklenmeli
        // (Zed'in WindowStoredSize/last_window_bounds yaklaşımı).
        // TODO: display_id — birden fazla monitörde aynı ekranda açmak için
        // kaydedilen ekran kimliği verilmeli.
        ..Default::default()
    }
}

/// Uygulamanın ana penceresini açar ve kapatma handler'ını kurar.
pub fn ana_pencere_ac(cx: &mut App) {
    let tema = *cx.global::<Tema>();

    #[cfg(target_os = "linux")]
    crate::linux_ikon::linux_ikon_kur();

    cx.spawn(async move |cx| {
        let options = pencere_secenekleri(&tema);

        let window_handle = cx
            .open_window(options, |_window, cx| cx.new(AnaPanel::yeni))
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

/// Kapatma isteğinin onaylanıp onaylanmadığını döner.
///
/// Şu an koşulsuz true; pencere kapanır, uygulama quit eder.
// TODO: Unsaved changes / dirty state eklenince Zed'in
// `workspace::prompt_and_save_items` akışına benzer bir onay diyaloğu
// buradan tetiklenecek.
pub fn kapatma_istegi(_window: &mut Window, _cx: &mut App) -> bool {
    true
}
