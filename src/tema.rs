#![allow(dead_code)]

use gpui::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ── Hex renk donusturme ────────────────────────────────────

/// "#RRGGBB" veya "#RRGGBBAA" hex stringini Hsla'ya donusturur.
fn hex_renk(hex: &str) -> Hsla {
    let hex = hex.trim_start_matches('#');
    let deger = u32::from_str_radix(hex, 16).unwrap_or(0);

    let (r, g, b, a) = if hex.len() > 6 {
        (
            ((deger >> 24) & 0xFF) as f32 / 255.0,
            ((deger >> 16) & 0xFF) as f32 / 255.0,
            ((deger >> 8) & 0xFF) as f32 / 255.0,
            (deger & 0xFF) as f32 / 255.0,
        )
    } else {
        (
            ((deger >> 16) & 0xFF) as f32 / 255.0,
            ((deger >> 8) & 0xFF) as f32 / 255.0,
            (deger & 0xFF) as f32 / 255.0,
            1.0,
        )
    };

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < f32::EPSILON {
        return hsla(0.0, 0.0, l, a);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if (max - r).abs() < f32::EPSILON {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if (max - g).abs() < f32::EPSILON {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };

    hsla(h, s, l, a)
}

// ── Pencere modu ───────────────────────────────────────────

/// Pencere arka plan modu.
///   otomatik - Sistem destekliyorsa blur, yoksa seffaf
///   seffaf   - Her zaman seffaf (blur olmadan)
///   opak     - Her zaman opak
#[derive(Deserialize, Serialize, Clone, Copy, Default, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PencereModu {
    #[default]
    Otomatik,
    Seffaf,
    Opak,
}

// ── TOML dosya yapisi ──────────────────────────────────────

#[derive(Deserialize, Serialize)]
pub struct TemaDosyasi {
    pub pencere: PencereBolumu,
    pub yerlesim: YerlesimBolumu,
    pub ust_bar: UstBarBolumu,
    pub metin: MetinBolumu,
    pub buton: ButonBolumu,
    pub kontrol: KontrolBolumu,
    pub kenarlik: KenarlikBolumu,
    pub vurgu: VurguBolumu,
    pub yuzey: YuzeyBolumu,
    pub durum: DurumBolumu,
    pub golge: GolgeBolumu,
}

#[derive(Deserialize, Serialize)]
pub struct YerlesimBolumu {
    pub sol_panel_genislik: f64,
}

#[derive(Deserialize, Serialize)]
pub struct PencereBolumu {
    /// "otomatik", "seffaf", "opak"
    #[serde(rename = "mod")]
    pub pencere_modu: PencereModu,
    /// Pencere arka plan rengi (hex, alpha yok)
    pub arka_plan: String,
    /// Blur aktifken seffaflik (0.0 - 1.0)
    pub blur_seffaflik: f64,
    /// Seffaf modda seffaflik (0.0 - 1.0)
    pub seffaf_seffaflik: f64,
    /// Pencere kose kavisi (piksel). 0 = kavis yok.
    /// Sadece seffaf/blur modda gorunur, opak modda etkisiz.
    pub kavis: f64,
}

#[derive(Deserialize, Serialize)]
pub struct UstBarBolumu {
    pub yukseklik: f64,
    pub sol_bosluk: f64,
    pub arka_plan: String,
    pub metin: String,
    pub ayirici: String,
}

#[derive(Deserialize, Serialize)]
pub struct MetinBolumu {
    pub birincil: String,
    pub ikincil: String,
    pub soluk: String,
}

#[derive(Deserialize, Serialize)]
pub struct ButonBolumu {
    pub arka_plan: String,
    pub hover: String,
    pub aktif: String,
    pub metin: String,
}

#[derive(Deserialize, Serialize)]
pub struct KontrolBolumu {
    pub hover: String,
    pub kapat_hover: String,
}

#[derive(Deserialize, Serialize)]
pub struct KenarlikBolumu {
    pub renk: String,
    pub ayirici: String,
}

#[derive(Deserialize, Serialize)]
pub struct VurguBolumu {
    pub renk: String,
    pub hover: String,
    pub metin: String,
}

#[derive(Deserialize, Serialize)]
pub struct YuzeyBolumu {
    pub katman_1: String,
    pub katman_2: String,
    pub katman_3: String,
}

#[derive(Deserialize, Serialize)]
pub struct DurumBolumu {
    pub basari: String,
    pub uyari: String,
    pub hata: String,
    pub bilgi: String,
}

#[derive(Deserialize, Serialize)]
pub struct GolgeBolumu {
    pub renk: String,
    pub seffaflik: f64,
}

impl TemaDosyasi {
    /// Varsayilan tema degerlerini olusturur.
    pub fn varsayilan() -> Self {
        Self {
            pencere: PencereBolumu {
                pencere_modu: PencereModu::Otomatik,
                arka_plan: "#1A1A2E".into(),
                blur_seffaflik: 0.45,
                seffaf_seffaflik: 0.80,
                kavis: 10.0,
            },
            yerlesim: YerlesimBolumu {
                sol_panel_genislik: 220.0,
            },
            ust_bar: UstBarBolumu {
                yukseklik: 40.0,
                sol_bosluk: varsayilan_ust_bar_sol_bosluk(),
                arka_plan: "#141420".into(),
                metin: "#E8E8F0".into(),
                ayirici: "#2A2A3E".into(),
            },
            metin: MetinBolumu {
                birincil: "#EDEDED".into(),
                ikincil: "#B3B3B3".into(),
                soluk: "#737373".into(),
            },
            buton: ButonBolumu {
                arka_plan: "#2A2A3E".into(),
                hover: "#3A3A4E".into(),
                aktif: "#4A4A5E".into(),
                metin: "#EDEDED".into(),
            },
            kontrol: KontrolBolumu {
                hover: "#2A2A3E".into(),
                kapat_hover: "#E05555".into(),
            },
            kenarlik: KenarlikBolumu {
                renk: "#2A2A3E".into(),
                ayirici: "#222233".into(),
            },
            vurgu: VurguBolumu {
                renk: "#5599DD".into(),
                hover: "#66AAEE".into(),
                metin: "#FFFFFF".into(),
            },
            yuzey: YuzeyBolumu {
                katman_1: "#1E1E32".into(),
                katman_2: "#24243A".into(),
                katman_3: "#2A2A42".into(),
            },
            durum: DurumBolumu {
                basari: "#55BB77".into(),
                uyari: "#DDAA33".into(),
                hata: "#DD5555".into(),
                bilgi: "#5599DD".into(),
            },
            golge: GolgeBolumu {
                renk: "#000000".into(),
                seffaflik: 0.40,
            },
        }
    }
}

// ── Tema dosya yolu ────────────────────────────────────────

/// Tema dosyasinin yolunu dondurur.
/// Linux:   ~/.config/gpui_app/tema.toml
/// macOS:   ~/Library/Application Support/gpui_app/tema.toml
/// Windows: %APPDATA%\gpui_app\tema.toml
pub fn tema_dosya_yolu() -> PathBuf {
    let mut yol = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    yol.push("gpui_app");
    yol.push("tema.toml");
    yol
}

// ── Calismna zamani tema yapisi ────────────────────────────

/// Tum uygulama renklerini tek bir yerden yoneten tema yapisi.
/// TOML dosyasindan yuklenir, pencere modu otomatik cozumlenir.
#[derive(Clone, Copy)]
pub struct Tema {
    // ── Pencere (sadece burasi blur/seffaf) ──
    pub pencere_gorunum: WindowBackgroundAppearance,
    pub pencere_arka_plan: Hsla,
    pub pencere_kavis: Pixels,

    // ── Yerlesim ──
    pub ust_bar_yukseklik: Pixels,
    pub sol_panel_genislik: Pixels,
    pub ust_bar_sol_bosluk: Pixels,

    // ── Ust bar ──
    pub ust_bar_arka_plan: Hsla,
    pub ust_bar_metin: Hsla,
    pub ust_bar_ayirici: Hsla,

    // ── Metin ──
    pub metin_birincil: Hsla,
    pub metin_ikincil: Hsla,
    pub metin_soluk: Hsla,

    // ── Butonlar ──
    pub buton_arka_plan: Hsla,
    pub buton_hover: Hsla,
    pub buton_aktif: Hsla,
    pub buton_metin: Hsla,

    // ── Pencere kontrol butonlari ──
    pub kontrol_hover: Hsla,
    pub kontrol_kapat_hover: Hsla,

    // ── Kenarlik ve ayiricilar ──
    pub kenarlik: Hsla,
    pub ayirici: Hsla,

    // ── Vurgu (accent) ──
    pub vurgu: Hsla,
    pub vurgu_hover: Hsla,
    pub vurgu_metin: Hsla,

    // ── Yuzey katmanlari ──
    pub yuzey_1: Hsla,
    pub yuzey_2: Hsla,
    pub yuzey_3: Hsla,

    // ── Durum renkleri ──
    pub basari: Hsla,
    pub uyari: Hsla,
    pub hata: Hsla,
    pub bilgi: Hsla,

    // ── Golge ──
    pub golge: Hsla,
}

impl Tema {
    /// TOML dosyasindan tema yukler.
    /// Dosya yoksa varsayilan tema olusturulur ve diske yazilir.
    /// Dosya okunamazsa varsayilan degerler kullanilir.
    pub fn yukle() -> Self {
        let yol = tema_dosya_yolu();
        let dosya = if yol.exists() {
            match std::fs::read_to_string(&yol) {
                Ok(icerik) => match toml::from_str::<TemaDosyasi>(&icerik) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("Tema dosyasi ayristirilamadi: {e}");
                        eprintln!("Varsayilan tema kullaniliyor.");
                        TemaDosyasi::varsayilan()
                    }
                },
                Err(e) => {
                    eprintln!("Tema dosyasi okunamadi: {e}");
                    TemaDosyasi::varsayilan()
                }
            }
        } else {
            let varsayilan = TemaDosyasi::varsayilan();
            // Varsayilan tema dosyasini diske yaz
            if let Some(dizin) = yol.parent() {
                let _ = std::fs::create_dir_all(dizin);
            }
            match toml::to_string_pretty(&varsayilan) {
                Ok(icerik) => {
                    let baslik = "\
# gpui_app Tema Dosyasi
#
# Pencere modu:
#   \"otomatik\" - Sistem blur destekliyorsa blur, yoksa seffaf
#   \"seffaf\"   - Her zaman seffaf (blur yok)
#   \"opak\"     - Her zaman opak
#
# Renkler: \"#RRGGBB\" veya \"#RRGGBBAA\" (alfa ile)
# Seffaflik: 0.0 (gorunmez) - 1.0 (opak)
#
# Blur ve seffaflik sadece ana pencere icin gecerlidir.
# Diger tum bilesenler (buton, metin, panel vs.) opak renk kullanir.

";
                    let _ = std::fs::write(&yol, format!("{baslik}{icerik}"));
                    eprintln!("Varsayilan tema olusturuldu: {}", yol.display());
                }
                Err(e) => eprintln!("Tema dosyasi yazilamadi: {e}"),
            }
            varsayilan
        };

        Self::dosyadan_olustur(&dosya)
    }

    /// TemaDosyasi'ndan calisma zamani Tema'yi olusturur.
    fn dosyadan_olustur(d: &TemaDosyasi) -> Self {
        // Pencere gorunumunu cozumle
        let pencere_gorunum = match d.pencere.pencere_modu {
            PencereModu::Otomatik => sistem_blur_destegi(),
            PencereModu::Seffaf => WindowBackgroundAppearance::Transparent,
            PencereModu::Opak => WindowBackgroundAppearance::Opaque,
        };

        // Pencere arka plan rengini moda gore alpha uygula
        let mut pencere_bg = hex_renk(&d.pencere.arka_plan);
        match pencere_gorunum {
            WindowBackgroundAppearance::Blurred => {
                pencere_bg.a = d.pencere.blur_seffaflik as f32;
            }
            WindowBackgroundAppearance::Transparent => {
                pencere_bg.a = d.pencere.seffaf_seffaflik as f32;
            }
            _ => {
                pencere_bg.a = 1.0;
            }
        }

        // Pencere kavisi: opak modda kavis uygulanmaz
        let pencere_kavis = match pencere_gorunum {
            WindowBackgroundAppearance::Opaque => px(0.),
            _ => px(d.pencere.kavis as f32),
        };

        // Golge rengi + seffaflik
        let mut golge = hex_renk(&d.golge.renk);
        golge.a = d.golge.seffaflik as f32;

        Self {
            pencere_gorunum,
            pencere_arka_plan: pencere_bg,
            pencere_kavis,

            ust_bar_yukseklik: px(d.ust_bar.yukseklik as f32),
            sol_panel_genislik: px(d.yerlesim.sol_panel_genislik as f32),
            ust_bar_sol_bosluk: px(d.ust_bar.sol_bosluk as f32),

            ust_bar_arka_plan: hex_renk(&d.ust_bar.arka_plan),
            ust_bar_metin: hex_renk(&d.ust_bar.metin),
            ust_bar_ayirici: hex_renk(&d.ust_bar.ayirici),

            metin_birincil: hex_renk(&d.metin.birincil),
            metin_ikincil: hex_renk(&d.metin.ikincil),
            metin_soluk: hex_renk(&d.metin.soluk),

            buton_arka_plan: hex_renk(&d.buton.arka_plan),
            buton_hover: hex_renk(&d.buton.hover),
            buton_aktif: hex_renk(&d.buton.aktif),
            buton_metin: hex_renk(&d.buton.metin),

            kontrol_hover: hex_renk(&d.kontrol.hover),
            kontrol_kapat_hover: hex_renk(&d.kontrol.kapat_hover),

            kenarlik: hex_renk(&d.kenarlik.renk),
            ayirici: hex_renk(&d.kenarlik.ayirici),

            vurgu: hex_renk(&d.vurgu.renk),
            vurgu_hover: hex_renk(&d.vurgu.hover),
            vurgu_metin: hex_renk(&d.vurgu.metin),

            yuzey_1: hex_renk(&d.yuzey.katman_1),
            yuzey_2: hex_renk(&d.yuzey.katman_2),
            yuzey_3: hex_renk(&d.yuzey.katman_3),

            basari: hex_renk(&d.durum.basari),
            uyari: hex_renk(&d.durum.uyari),
            hata: hex_renk(&d.durum.hata),
            bilgi: hex_renk(&d.durum.bilgi),

            golge,
        }
    }
}

// ── Platform varsayilan degerleri ──────────────────────────

/// macOS'ta trafik isiklari icin genis bosluk, diger platformlarda dar.
fn varsayilan_ust_bar_sol_bosluk() -> f64 {
    #[cfg(target_os = "macos")]
    { 80.0 }
    #[cfg(not(target_os = "macos"))]
    { 12.0 }
}

// ── Sistem blur destegi algilama ───────────────────────────

/// Mevcut platformda compositor blur destegi olup olmadigini tespit eder.
fn sistem_blur_destegi() -> WindowBackgroundAppearance {
    #[cfg(target_os = "macos")]
    {
        return WindowBackgroundAppearance::Blurred;
    }

    #[cfg(target_os = "windows")]
    {
        return WindowBackgroundAppearance::Blurred;
    }

    #[cfg(target_os = "linux")]
    {
        let oturum = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
        let masaustu = std::env::var("XDG_CURRENT_DESKTOP")
            .unwrap_or_default()
            .to_uppercase();

        if oturum == "wayland" && masaustu.contains("KDE") {
            WindowBackgroundAppearance::Blurred
        } else {
            WindowBackgroundAppearance::Transparent
        }
    }
}
