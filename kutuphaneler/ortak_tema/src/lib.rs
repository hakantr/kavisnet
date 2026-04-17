#![allow(dead_code)]

use chrono::Local;
use gpui::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// ── Hata Kayitlari ve Loglama ─────────────────────────────

/// Hata loglarinin tutulacagi dizin yolu.
pub fn hata_log_dizini() -> PathBuf {
    let mut yol = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    yol.pop();
    yol.push("hata_kayitlari");

    if !yol.exists() {
        let _ = fs::create_dir_all(&yol);
    }
    yol
}

/// Temadaki bir hatayi log dosyasina kaydeder.
fn hatayi_kaydet(hata: &str) {
    let dizin = hata_log_dizini();
    let dosya_adi = format!("tema_hatalari_{}.log", Local::now().format("%Y-%m-%d"));
    let tam_yol = dizin.join(dosya_adi);

    let zaman = Local::now().format("%H:%M:%S");
    let log_satiri = format!("[{}] HATA: {}\n", zaman, hata);

    if let Ok(mut dosya) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&tam_yol)
    {
        use std::io::Write;
        let _ = writeln!(dosya, "{}", log_satiri);
    }
}

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

// ── Gorunum (Zed `Appearance` esdegeri) ────────────────────

/// Temanin aydinlik/koyu yonu. Zed'in `Appearance` enum'u ile eslestiriliyor;
/// sistem tema algilama ve otomatik varyant secimi icin referans noktasi.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Gorunum {
    #[default]
    Koyu,
    Aydinlik,
}

// ── Sistem gorunumu (Zed `SystemAppearance` esdegeri) ──────

/// Isletim sistemi genelinde aydinlik/koyu tercihini tasiyan `Global`.
/// Zed'in `SystemAppearance(pub Appearance)` newtype pattern'i ile ayni rol.
/// `dark-light` crate'i macOS'ta `NSApp.effectiveAppearance`, Linux'ta
/// gsettings/xdg-desktop-portal, Windows'ta Personalize registry anahtarini
/// sorgular.
#[derive(Clone, Copy, Debug, Default)]
pub struct SistemGorunumu(pub Gorunum);

impl SistemGorunumu {
    /// Sistemi tek seferlik sorgular. `Mode::Default` (tercih bildirilmemis)
    /// durumunda Koyu'ya dusulur — uygulama icin nihai bir default gerekli.
    pub fn tespit_et() -> Self {
        let gorunum = match dark_light::detect() {
            dark_light::Mode::Dark => Gorunum::Koyu,
            dark_light::Mode::Light => Gorunum::Aydinlik,
            dark_light::Mode::Default => Gorunum::Koyu,
        };
        Self(gorunum)
    }

    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }
}

impl Global for SistemGorunumu {}

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

// ── Tema secimi (Zed `ThemeSelection` esdegeri) ────────────

/// Kullanicinin aktif varyant tercihi. `Sabit`: tek bir varyant ismi.
/// `Sistem`: aydinlik/koyu icin ayri iki isim, `SistemGorunumu`'na gore
/// secilir. Zed'in `ThemeSettings::theme_selection` ayirimiyla ayni.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "mod", rename_all = "lowercase")]
pub enum TemaSecimi {
    Sabit {
        varyant: String,
    },
    Sistem {
        aydinlik: String,
        koyu: String,
    },
}

impl TemaSecimi {
    /// Verilen sistem gorunumuyle aktif varyantin adini dondurur.
    fn hedef_ad(&self, sistem: Gorunum) -> &str {
        match self {
            TemaSecimi::Sabit { varyant } => varyant.as_str(),
            TemaSecimi::Sistem { aydinlik, koyu } => match sistem {
                Gorunum::Aydinlik => aydinlik.as_str(),
                Gorunum::Koyu => koyu.as_str(),
            },
        }
    }
}

// ── TOML dosya yapisi ──────────────────────────────────────

/// Bir tema ailesinin TOML temsili. Zed'in `ThemeFamilyContent` + `ThemeRegistry`
/// ikilisiyle ayni rolu tasir: tek dosyada birden cok varyant, aktif varyant
/// `TemaSecimi` ile secilir.
#[derive(Deserialize, Serialize)]
pub struct TemaAilesiDosyasi {
    pub ad: String,
    pub yazar: String,
    pub secim: TemaSecimi,
    pub varyantlar: Vec<TemaVaryantDosyasi>,
}

/// Tek bir tema varyantinin TOML temsili. Zed'in `ThemeContent` esdegeri;
/// aile icinde birden cok tanesi bulunur.
#[derive(Deserialize, Serialize)]
pub struct TemaVaryantDosyasi {
    pub ad: String,
    #[serde(default)]
    pub gorunum: Gorunum,
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
    pub sol_panel_min_genislik: f64,
    pub calisma_yuzeyi_kavis: f64,
    pub calisma_yuzeyi_kavisli_mi: bool,
    pub ust_sinir: bool,
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

impl TemaAilesiDosyasi {
    /// Varsayilan tema ailesi: "KavisNet Koyu" + "KavisNet Aydinlik", sistem
    /// gorunumune gore otomatik secim aktif.
    pub fn varsayilan() -> Self {
        Self {
            ad: "KavisNet".into(),
            yazar: "KavisNet".into(),
            secim: TemaSecimi::Sistem {
                aydinlik: "KavisNet Aydinlik".into(),
                koyu: "KavisNet Koyu".into(),
            },
            varyantlar: vec![
                TemaVaryantDosyasi::varsayilan_koyu(),
                TemaVaryantDosyasi::varsayilan_aydinlik(),
            ],
        }
    }
}

impl TemaVaryantDosyasi {
    /// Varsayilan koyu varyant.
    pub fn varsayilan_koyu() -> Self {
        Self {
            ad: "KavisNet Koyu".into(),
            gorunum: Gorunum::Koyu,
            pencere: PencereBolumu {
                pencere_modu: PencereModu::Otomatik,
                arka_plan: "#1A1A2E".into(),
                blur_seffaflik: 0.45,
                seffaf_seffaflik: 0.80,
                kavis: 10.0,
            },
            yerlesim: YerlesimBolumu {
                sol_panel_min_genislik: 220.0,
                calisma_yuzeyi_kavis: 10.0,
                calisma_yuzeyi_kavisli_mi: true,
                ust_sinir: true,
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
                hover: "#B8C7FF".into(),
                kapat_hover: "#FF7A7A".into(),
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

    /// Varsayilan aydinlik varyant. Koyunun tamamlayicisi; ayni semantik
    /// alanlar dolu — kullanici bazi alanlari silse de tutarli kalsin.
    pub fn varsayilan_aydinlik() -> Self {
        Self {
            ad: "KavisNet Aydinlik".into(),
            gorunum: Gorunum::Aydinlik,
            pencere: PencereBolumu {
                pencere_modu: PencereModu::Otomatik,
                arka_plan: "#F5F5FA".into(),
                blur_seffaflik: 0.55,
                seffaf_seffaflik: 0.85,
                kavis: 10.0,
            },
            yerlesim: YerlesimBolumu {
                sol_panel_min_genislik: 220.0,
                calisma_yuzeyi_kavis: 10.0,
                calisma_yuzeyi_kavisli_mi: true,
                ust_sinir: true,
            },
            ust_bar: UstBarBolumu {
                yukseklik: 40.0,
                sol_bosluk: varsayilan_ust_bar_sol_bosluk(),
                arka_plan: "#ECECF5".into(),
                metin: "#1A1A2E".into(),
                ayirici: "#D0D0DD".into(),
            },
            metin: MetinBolumu {
                birincil: "#1A1A2E".into(),
                ikincil: "#4A4A5E".into(),
                soluk: "#8A8A9E".into(),
            },
            buton: ButonBolumu {
                arka_plan: "#DDDDEE".into(),
                hover: "#CCCCDD".into(),
                aktif: "#BBBBCC".into(),
                metin: "#1A1A2E".into(),
            },
            kontrol: KontrolBolumu {
                hover: "#3355BB".into(),
                kapat_hover: "#DD3333".into(),
            },
            kenarlik: KenarlikBolumu {
                renk: "#D0D0DD".into(),
                ayirici: "#E0E0EE".into(),
            },
            vurgu: VurguBolumu {
                renk: "#3377CC".into(),
                hover: "#2266BB".into(),
                metin: "#FFFFFF".into(),
            },
            yuzey: YuzeyBolumu {
                katman_1: "#FFFFFF".into(),
                katman_2: "#F8F8FC".into(),
                katman_3: "#F0F0F5".into(),
            },
            durum: DurumBolumu {
                basari: "#33AA55".into(),
                uyari: "#CC8800".into(),
                hata: "#CC3333".into(),
                bilgi: "#3377CC".into(),
            },
            golge: GolgeBolumu {
                renk: "#000000".into(),
                seffaflik: 0.20,
            },
        }
    }
}

// ── Tema dosya yolu ────────────────────────────────────────

/// Tema dosyasinin yolunu dondurur.
/// Linux:   ~/.config/KavisNet/tema.toml
/// macOS:   ~/Library/Application Support/KavisNet/tema.toml
/// Windows: %APPDATA%\KavisNet\tema.toml
pub fn tema_dosya_yolu() -> PathBuf {
    let mut yol = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    yol.push("KavisNet");
    yol.push("tema.toml");
    yol
}

// ── Calisma zamani tema yapisi ─────────────────────────────

/// Tum uygulama renklerini tek bir yerden yoneten tema yapisi.
///
/// Zed'in `Theme` yapisi `Arc` sarmaliyken bizim `Tema` dogrudan `Global`
/// olarak `cx` icinde tutuluyor; `SharedString` iceren alanlar yuzunden
/// `Copy` degil `Clone`.
#[derive(Clone)]
pub struct Tema {
    // ── Kimlik ──
    pub ad: SharedString,
    pub gorunum: Gorunum,

    // ── Pencere (sadece burasi blur/seffaf) ──
    pub pencere_gorunum: WindowBackgroundAppearance,
    pub pencere_arka_plan: Hsla,
    pub pencere_kavis: Pixels,

    // ── Yerlesim ──
    pub ust_bar_yukseklik: Pixels,
    pub sol_panel_min_genislik: Pixels,
    pub ust_bar_sol_bosluk: Pixels,
    pub calisma_yuzeyi_kavis: Pixels,
    pub calisma_yuzeyi_kavisli_mi: bool,
    pub ust_sinir: bool,

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
    /// Tek bir varyanttan calisma zamani `Tema`'sini olusturur. `TemaKaydi`
    /// butun varyantlari yuklerken bunu kullanir.
    fn varyanttan_olustur(d: &TemaVaryantDosyasi) -> Self {
        // ust_sinir = false iken pencere geleneksel/klasik gorunum alir:
        // seffaflik/blur devre disi, kose kavisi 0.
        let pencere_gorunum = if !d.yerlesim.ust_sinir {
            WindowBackgroundAppearance::Opaque
        } else {
            match d.pencere.pencere_modu {
                PencereModu::Otomatik => sistem_blur_destegi(),
                PencereModu::Seffaf => WindowBackgroundAppearance::Transparent,
                PencereModu::Opak => WindowBackgroundAppearance::Opaque,
            }
        };

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

        let pencere_kavis = match pencere_gorunum {
            WindowBackgroundAppearance::Opaque => px(0.),
            _ => px(d.pencere.kavis as f32),
        };

        let mut golge = hex_renk(&d.golge.renk);
        golge.a = d.golge.seffaflik as f32;

        Self {
            ad: SharedString::from(d.ad.clone()),
            gorunum: d.gorunum,

            pencere_gorunum,
            pencere_arka_plan: pencere_bg,
            pencere_kavis,

            ust_bar_yukseklik: px(d.ust_bar.yukseklik as f32),
            sol_panel_min_genislik: px(d.yerlesim.sol_panel_min_genislik as f32),
            ust_bar_sol_bosluk: px(d.ust_bar.sol_bosluk as f32),
            calisma_yuzeyi_kavis: px(d.yerlesim.calisma_yuzeyi_kavis as f32),
            calisma_yuzeyi_kavisli_mi: d.yerlesim.calisma_yuzeyi_kavisli_mi,
            ust_sinir: d.yerlesim.ust_sinir,

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

impl Global for Tema {}

// ── AktifTema (Zed `ActiveTheme` esdegeri) ──────────────────

/// `cx.tema()` cagrisi icin ergonomik trait. Zed'in `ActiveTheme` pattern'i
/// ile ayni: `App`/`Context` deref zinciri uzerinden global tema referansi.
pub trait AktifTema {
    fn tema(&self) -> &Tema;
}

impl AktifTema for App {
    fn tema(&self) -> &Tema {
        self.global::<Tema>()
    }
}

// ── TemaKaydi (Zed `ThemeRegistry` esdegeri) ────────────────

/// Yuklenmis tum tema varyantlarini barindiran global defter. Varsayilan
/// Koyu + Aydinlik her zaman icerir; kullanici ekledigi varyantlar ayni
/// ada sahipse ustune yazar (Zed `ThemeRegistry::register` ile ayni).
pub struct TemaKaydi {
    temalar: Vec<Tema>,
}

impl TemaKaydi {
    /// Tamamen bos kayit. Tek basina yeterli degildir — `aktif_temayi_sec`
    /// fallback icin en az bir girdiye ihtiyac duyar. Genellikle
    /// `varsayilan_ile` ile olusturulur.
    pub fn yeni_bos() -> Self {
        Self { temalar: Vec::new() }
    }

    /// Varsayilan Koyu + Aydinlik temalari onceden eklenmis kayit.
    /// Kullanici dosyasi bu iki temayi ezebilir veya ek tema ekleyebilir.
    pub fn varsayilan_ile() -> Self {
        let mut kayit = Self::yeni_bos();
        kayit.kaydet(Tema::varyanttan_olustur(&TemaVaryantDosyasi::varsayilan_koyu()));
        kayit.kaydet(Tema::varyanttan_olustur(&TemaVaryantDosyasi::varsayilan_aydinlik()));
        kayit
    }

    /// Ad cakismasi varsa mevcut kaydin uzerine yazar, yoksa sona ekler.
    pub fn kaydet(&mut self, tema: Tema) {
        if let Some(yer) = self.temalar.iter().position(|t| t.ad == tema.ad) {
            self.temalar[yer] = tema;
        } else {
            self.temalar.push(tema);
        }
    }

    pub fn al(&self, ad: &str) -> Option<&Tema> {
        self.temalar.iter().find(|t| t.ad.as_ref() == ad)
    }

    pub fn adlar(&self) -> Vec<SharedString> {
        self.temalar.iter().map(|t| t.ad.clone()).collect()
    }

    /// Belirtilen gorunume sahip (koyu veya aydinlik) tum temalar.
    pub fn gorunume_gore(&self, gorunum: Gorunum) -> Vec<&Tema> {
        self.temalar
            .iter()
            .filter(|t| t.gorunum == gorunum)
            .collect()
    }

    /// Bos olmayan bir kayitta ilk tema; mutlak fallback.
    fn ilk(&self) -> Option<&Tema> {
        self.temalar.first()
    }

    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }
}

impl Global for TemaKaydi {}

// ── Tema kurulumu ve yukleme ───────────────────────────────

/// Tema modulunu `App`'e kurar: `SistemGorunumu`, `TemaKaydi` ve aktif `Tema`
/// global'lerini yerlestirir. Zed'in `theme::init` + `LoadThemes` + ayar
/// observer'larinin ettigi isi tek fonksiyonda ozetliyor.
///
/// `temayi_izle` sonradan cagrilmasi beklenen ayri bir watcher — `kurulum`
/// sadece baslangic durumunu hazirlar.
pub fn kurulum(cx: &mut App) {
    let sistem = SistemGorunumu::tespit_et();
    let (kayit, aktif) = yukleme_bileseni(sistem.0);
    cx.set_global(sistem);
    cx.set_global(kayit);
    cx.set_global(aktif);
}

/// Dosyadan (yoksa varsayilandan) kayit + aktif tema uretir.
/// `SistemGorunumu` `TemaSecimi::Sistem` icin aktif varyanti belirler.
fn yukleme_bileseni(sistem: Gorunum) -> (TemaKaydi, Tema) {
    let yol = tema_dosya_yolu();
    let aile = aileyi_yukle_veya_yaz(&yol);

    let mut kayit = TemaKaydi::varsayilan_ile();
    for v in &aile.varyantlar {
        kayit.kaydet(Tema::varyanttan_olustur(v));
    }

    let aktif = secim_ile_aktif_tema(&aile.secim, sistem, &kayit);
    (kayit, aktif)
}

/// Dosyayi okur/ayristirir; hata varsa varsayilan aileyi dondurur.
/// Dosya hic yoksa varsayilan aileyi diske yazar.
fn aileyi_yukle_veya_yaz(yol: &Path) -> TemaAilesiDosyasi {
    if yol.exists() {
        match std::fs::read_to_string(yol) {
            Ok(icerik) => match toml::from_str::<TemaAilesiDosyasi>(&icerik) {
                Ok(a) => a,
                Err(e) => {
                    let hata_mesaji = format!("Tema dosyasi ayristirilamadi: {e}");
                    eprintln!("{hata_mesaji}");
                    hatayi_kaydet(&hata_mesaji);
                    TemaAilesiDosyasi::varsayilan()
                }
            },
            Err(e) => {
                let hata_mesaji = format!("Tema dosyasi okunamadi: {e}");
                eprintln!("{hata_mesaji}");
                hatayi_kaydet(&hata_mesaji);
                TemaAilesiDosyasi::varsayilan()
            }
        }
    } else {
        let varsayilan = TemaAilesiDosyasi::varsayilan();
        if let Some(dizin) = yol.parent() {
            let _ = std::fs::create_dir_all(dizin);
        }
        match toml::to_string_pretty(&varsayilan) {
            Ok(icerik) => {
                let baslik = "\
# KavisNet Tema Dosyasi
#
# Bu dosya bir tema AILESIDIR: birden cok varyant (koyu/aydinlik) icerir.
# Aktif varyanti `secim` bloguyla secersin:
#
#   [secim]
#   mod = \"sabit\"            # tek bir varyanti zorla
#   varyant = \"KavisNet Koyu\"
#
#   [secim]
#   mod = \"sistem\"           # sistemin koyu/aydinlik tercihini izle
#   aydinlik = \"KavisNet Aydinlik\"
#   koyu = \"KavisNet Koyu\"
#
# Pencere modu:
#   \"otomatik\" - Sistem blur destekliyorsa blur, yoksa seffaf
#   \"seffaf\"   - Her zaman seffaf (blur yok)
#   \"opak\"     - Her zaman opak
#
# Gorunum alani: \"koyu\" veya \"aydinlik\" — `secim` sistem modunda iken
#               hangi varyantin aydinlik/koyu sayilacagini belirtir.
#
# Renkler: \"#RRGGBB\" veya \"#RRGGBBAA\" (alfa ile)
# Seffaflik: 0.0 (gorunmez) - 1.0 (opak)

";
                let _ = std::fs::write(yol, format!("{baslik}{icerik}"));
                eprintln!("Varsayilan tema ailesi olusturuldu: {}", yol.display());
            }
            Err(e) => eprintln!("Tema dosyasi yazilamadi: {e}"),
        }
        varsayilan
    }
}

/// Secimi ve sistemi birlikte degerlendirerek kayittan aktif temayi dondurur.
/// Hedef varyant bulunamazsa kayittaki ilk temaya, o da yoksa yerlesik
/// varsayilan koyuya duser.
fn secim_ile_aktif_tema(secim: &TemaSecimi, sistem: Gorunum, kayit: &TemaKaydi) -> Tema {
    let hedef = secim.hedef_ad(sistem);
    kayit
        .al(hedef)
        .or_else(|| kayit.ilk())
        .cloned()
        .unwrap_or_else(|| {
            hatayi_kaydet(&format!(
                "Aktif tema '{hedef}' bulunamadi ve kayit bos; varsayilan koyu kullaniliyor."
            ));
            Tema::varyanttan_olustur(&TemaVaryantDosyasi::varsayilan_koyu())
        })
}

/// Tema dosyasini arka planda izler; degisiklik oldugunda kayit + aktif
/// temayi birlikte gunceller. Zed'in tema dosyasi watcher'inin esdegeri.
pub fn temayi_izle(cx: &mut App) {
    use notify::{RecursiveMode, Watcher};
    use std::sync::mpsc::channel;
    use std::time::Duration;

    let yol = tema_dosya_yolu();

    cx.spawn(async move |cx| {
        let (tx, rx) = channel();
        let izlenen_yol = yol.clone();
        let _watcher = notify::recommended_watcher(move |res| {
            if let Ok(notify::Event { kind, paths, .. }) = res {
                let bizi_ilgilendiriyor = paths.iter().any(|p| p == &izlenen_yol);
                if !bizi_ilgilendiriyor {
                    return;
                }
                if matches!(
                    kind,
                    notify::EventKind::Modify(_)
                        | notify::EventKind::Create(_)
                        | notify::EventKind::Remove(_)
                ) {
                    let _ = tx.send(());
                }
            }
        })
        .expect("Tema izleyici başlatılamadı");

        let mut _watcher = _watcher;
        if let Some(parent) = yol.parent() {
            let _ = _watcher.watch(parent, RecursiveMode::NonRecursive);
        }

        let mut son_icerik = std::fs::read_to_string(&yol).unwrap_or_default();

        loop {
            let mut olay_var = false;
            while let Ok(_) = rx.try_recv() {
                olay_var = true;
            }

            if olay_var {
                smol::Timer::after(Duration::from_millis(100)).await;
                let durum = std::fs::read_to_string(&yol);
                match durum {
                    Ok(yeni_icerik) if yeni_icerik != son_icerik => {
                        son_icerik = yeni_icerik;
                        let _ = cx.update(|cx| {
                            let sistem = SistemGorunumu::global(cx).0;
                            let (kayit, aktif) = yukleme_bileseni(sistem);
                            cx.set_global(kayit);
                            cx.set_global(aktif);
                            println!("Tema canli olarak guncellendi.");
                        });
                    }
                    Ok(_) => {}
                    Err(_) => {
                        son_icerik.clear();
                        let _ = cx.update(|cx| {
                            let sistem = SistemGorunumu::global(cx).0;
                            // Dosya yok — varsayilan aileden yeniden kur.
                            let mut kayit = TemaKaydi::varsayilan_ile();
                            let aile = TemaAilesiDosyasi::varsayilan();
                            for v in &aile.varyantlar {
                                kayit.kaydet(Tema::varyanttan_olustur(v));
                            }
                            let aktif = secim_ile_aktif_tema(&aile.secim, sistem, &kayit);
                            cx.set_global(kayit);
                            cx.set_global(aktif);
                            println!("Tema dosyasi bulunamadi, varsayilan temaya donuldu.");
                        });
                    }
                }
            }
            smol::Timer::after(Duration::from_millis(250)).await;
        }
    })
    .detach();
}

// ── Platform varsayilan degerleri ──────────────────────────

/// macOS'ta trafik isiklari icin genis bosluk, diger platformlarda dar.
fn varsayilan_ust_bar_sol_bosluk() -> f64 {
    #[cfg(target_os = "macos")]
    {
        80.0
    }
    #[cfg(not(target_os = "macos"))]
    {
        12.0
    }
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
        // GPUI v0.232.2 Wayland tarafında `org_kde_kwin_blur` protokolünü
        // kullanarak compositor-side blur uyguluyor. Protokol whitelist'i:
        //   KDE Plasma / KWin — protokol sahibi.
        //   Hyprland         — `org_kde_kwin_blur_manager` global'ini sunar.
        //   Wayfire (+plugin) — aynı protokolü sunar.
        // GNOME Mutter, Sway, Weston bu protokolü sunmadığı için Transparent
        // kalıyor (GPUI blur isteğini sessizce etkisiz bırakır, ama alpha
        // `blur_seffaflik` değerine takılacağı için silik görünüm olur).
        let oturum = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
        let masaustu = std::env::var("XDG_CURRENT_DESKTOP")
            .unwrap_or_default()
            .to_uppercase();

        let blur_destekli = masaustu.contains("KDE")
            || masaustu.contains("HYPRLAND")
            || masaustu.contains("WAYFIRE");

        if oturum == "wayland" && blur_destekli {
            WindowBackgroundAppearance::Blurred
        } else {
            WindowBackgroundAppearance::Transparent
        }
    }
}
