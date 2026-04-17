//! Linux: ikon + .desktop kurulumu.
//!
//! XDG Icon Theme spec'ine göre hicolor tema altına çoklu boyut PNG yazıp
//! `.desktop` dosyasını kullanıcı veri dizinine yerleştirir. app_id değeri
//! X11 WM_CLASS / Wayland xdg-shell app_id'ye yazıldığından .desktop
//! StartupWMClass alanıyla eşleşir ve dock/taskbar ikonu doğru görünür.

use crate::pencere::UYGULAMA_APP_ID;

/// XDG Icon Theme spec'ine göre yüklenecek hicolor boyutları ve gömülü PNG
/// verileri. DE'ler bağlama göre en uygun olanı seçer (16: sistem tepsisi,
/// 32–48: pencere başlığı / görev değiştirici, 128+: dock / başlatıcı).
const UYGULAMA_IKONLARI: &[(u32, &[u8])] = &[
    (
        16,
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../resimler/ikonlar/ikon_02_16.png"
        )),
    ),
    (
        32,
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../resimler/ikonlar/ikon_02_32.png"
        )),
    ),
    (
        48,
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../resimler/ikonlar/ikon_02_48.png"
        )),
    ),
    (
        64,
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../resimler/ikonlar/ikon_02_64.png"
        )),
    ),
    (
        128,
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../resimler/ikonlar/ikon_02_128.png"
        )),
    ),
    (
        256,
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../resimler/ikonlar/ikon_02_256.png"
        )),
    ),
    (
        512,
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../resimler/ikonlar/ikon_02_512.png"
        )),
    ),
];

/// Linux'ta dock/taskbar ikonu için .desktop ve PNG'yi kullanıcı veri dizinine
/// yazar. GNOME, KDE Plasma, XFCE, MATE, Cinnamon, LXQt, Sway, i3, Hyprland
/// vb. pencere yöneticilerinde X11 ve Wayland oturumlarında çalışır.
pub fn linux_ikon_kur() {
    let Some(veri_dizini) = dirs::data_dir() else {
        return;
    };

    // 1) PNG'leri hicolor ikon teması altına çoklu boyut olarak yaz.
    let hicolor_koku = veri_dizini.join("icons/hicolor");
    let mut herhangi_guncellendi = false;
    let mut son_ikon_yolu: Option<std::path::PathBuf> = None;

    for &(boyut, veri) in UYGULAMA_IKONLARI {
        let boyut_dizini = hicolor_koku.join(format!("{boyut}x{boyut}/apps"));
        if std::fs::create_dir_all(&boyut_dizini).is_err() {
            continue;
        }
        let yol = boyut_dizini.join(format!("{UYGULAMA_APP_ID}.png"));
        let guncellenmeli = match std::fs::metadata(&yol) {
            Ok(m) => m.len() as usize != veri.len(),
            Err(_) => true,
        };
        if guncellenmeli && std::fs::write(&yol, veri).is_ok() {
            herhangi_guncellendi = true;
        }
        son_ikon_yolu = Some(yol);
    }

    // 2) .desktop dosyasını yaz. Icon= için mutlak yol kullanmak herhangi
    //    bir tema aramasına ihtiyaç bırakmaz.
    let exe_yolu = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(String::from))
        .unwrap_or_else(|| UYGULAMA_APP_ID.into());

    let ikon_mutlak_yol = son_ikon_yolu
        .as_ref()
        .and_then(|p| p.to_str())
        .map(String::from)
        .unwrap_or_else(|| UYGULAMA_APP_ID.into());

    let desktop_dizini = veri_dizini.join("applications");
    if std::fs::create_dir_all(&desktop_dizini).is_err() {
        return;
    }
    let desktop_yolu = desktop_dizini.join(format!("{UYGULAMA_APP_ID}.desktop"));
    let desktop_icerik = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=KavisNet\n\
         Exec={exe_yolu}\n\
         Icon={ikon_mutlak_yol}\n\
         StartupWMClass={UYGULAMA_APP_ID}\n\
         StartupNotify=true\n\
         Terminal=false\n\
         Categories=Utility;\n"
    );

    let desktop_guncellenmeli = std::fs::read_to_string(&desktop_yolu)
        .map(|m| m != desktop_icerik)
        .unwrap_or(true);
    if desktop_guncellenmeli {
        let _ = std::fs::write(&desktop_yolu, desktop_icerik);
    }

    // 3) GTK/KDE ikon önbelleklerini yenile (best-effort).
    if herhangi_guncellendi {
        let _ = std::process::Command::new("gtk-update-icon-cache")
            .arg("-q")
            .arg("-t")
            .arg("-f")
            .arg(&hicolor_koku)
            .status();
    }
    if desktop_guncellenmeli {
        let _ = std::process::Command::new("update-desktop-database")
            .arg("-q")
            .arg(&desktop_dizini)
            .status();
    }
}
