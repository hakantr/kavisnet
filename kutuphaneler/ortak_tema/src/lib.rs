#![allow(dead_code)]

use chrono::Local;
use gpui::*;
use schemars::JsonSchema;
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
#[derive(Deserialize, Serialize, JsonSchema, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Gorunum {
    #[default]
    Koyu,
    Aydinlik,
}

// ── Sistem gorunumu (Zed `SystemAppearance` esdegeri) ──────

/// Isletim sistemi genelinde aydinlik/koyu tercihini tasiyan `Global`.
#[derive(Clone, Copy, Debug, Default)]
pub struct SistemGorunumu(pub Gorunum);

impl SistemGorunumu {
    /// Acilista pencere olusmadan once kullanilan en-iyi-tahmin.
    /// `dark_light::detect()` macOS'ta `NSAppearance currentAppearance`
    /// thread-local cache yuzunden bazen stale donuyor; Ubuntu/GNOME'da
    /// ise v1.1 `gtk-theme` okuyor ama GNOME 42+ `color-scheme` kullaniyor
    /// (gtk-theme artik "Adwaita"/"Yaru" olarak sabit). Bu yuzden canli
    /// takip `pencere_gorunumunu_uygula()` + `observe_window_appearance`
    /// ile yapiliyor; burasi sadece window olusana kadarki ilk render'i
    /// besliyor.
    pub fn tespit_et() -> Self {
        let gorunum = match dark_light::detect() {
            dark_light::Mode::Dark => Gorunum::Koyu,
            dark_light::Mode::Light => Gorunum::Aydinlik,
            // Belirsiz — pencere acildigi an `window.appearance()` ile
            // duzelecegi icin daha yaygin varsayilan olan Aydinlik'a koy.
            dark_light::Mode::Default => Gorunum::Aydinlik,
        };
        Self(gorunum)
    }

    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }
}

/// Pencere olustuktan sonra GPUI'nin platform dinleyicisiyle gelen
/// `WindowAppearance`'i okuyup `SistemGorunumu` + `TemaKaydi` + aktif
/// `Tema` global'lerini senkronlar. Hem ilk cagri (initial) hem de
/// `observe_window_appearance` callback'i icin ayni giris noktasi.
///
/// macOS: `-[NSView viewDidChangeEffectiveAppearance]` uzerinden anlik.
/// Linux: xdg-desktop-portal `org.freedesktop.appearance/color-scheme`
///        sinyali; GNOME 42+'nin gercek kaynaginda dinleme (gtk-theme
///        okumakla hic alakasi yok).
pub fn pencere_gorunumunu_uygula(window: &mut Window, cx: &mut App) {
    let gorunum = match window.appearance() {
        WindowAppearance::Dark | WindowAppearance::VibrantDark => Gorunum::Koyu,
        WindowAppearance::Light | WindowAppearance::VibrantLight => Gorunum::Aydinlik,
    };
    if SistemGorunumu::global(cx).0 != gorunum {
        cx.set_global(SistemGorunumu(gorunum));
        let (kayit, aktif) = yukleme_bileseni(gorunum);
        cx.set_global(kayit);
        cx.set_global(aktif);
        println!("Sistem gorunumu degisti: {gorunum:?} — tema yeniden secildi.");
    }
}

impl Global for SistemGorunumu {}

// ── Pencere modu ───────────────────────────────────────────

/// Pencere arka plan modu.
///   otomatik - Sistem destekliyorsa blur, yoksa seffaf
///   seffaf   - Her zaman seffaf (blur olmadan)
///   opak     - Her zaman opak
#[derive(Deserialize, Serialize, JsonSchema, Clone, Copy, Default, Debug)]
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
/// secilir.
#[derive(Deserialize, Serialize, JsonSchema, Clone, Debug)]
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

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct TemaAilesiDosyasi {
    pub ad: String,
    pub yazar: String,
    pub secim: TemaSecimi,
    /// Kullanici varyantlari YAMA (refinement) olarak tutulur; bos birakilan
    /// alanlar `temel` varyanttan kalitilir. Zed `ThemeContent` +
    /// `Refineable` akisina karsi gelir.
    pub varyantlar: Vec<TemaVaryantYamasi>,
}

/// Zed'in `ThemeContent` esdegeri: bir varyantin tum yapisi (tum alanlar
/// zorunlu). Yerlesik varsayilanlari ifade etmek icin kullanilir; kullanici
/// dosyasi yerine `TemaVaryantYamasi` yaziyor.
#[derive(Deserialize, Serialize, Clone)]
pub struct TemaVaryantDosyasi {
    pub ad: String,
    #[serde(default)]
    pub gorunum: Gorunum,
    pub pencere: PencereBolumu,
    pub yerlesim: YerlesimBolumu,
    pub renkler: RenklerBolumu,
    pub durum: DurumBolumu,
}

#[derive(Deserialize, Serialize, Clone)]
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

#[derive(Deserialize, Serialize, Clone)]
pub struct YerlesimBolumu {
    pub ust_bar_yukseklik: f64,
    pub ust_bar_sol_bosluk: f64,
    pub sol_panel_min_genislik: f64,
    pub calisma_yuzeyi_kavis: f64,
    pub calisma_yuzeyi_kavisli_mi: bool,
    pub ust_sinir: bool,
}

/// Zed `ThemeColors`'inin bizim uygulamamiza indirgenmis hali.
/// Alanlar Zed semantiginin Turkce karsiligi: `text` → `metin`,
/// `element_background` → `eleman_arka_plan`, `ghost_element_hover`
/// yerine icon-hover olarak acik semantik (`ikon_vurgu`, `ikon_kritik`).
#[derive(Deserialize, Serialize, Clone)]
pub struct RenklerBolumu {
    // Yuzeyler
    pub yuzey_arka_plan: String,
    pub yuksek_yuzey_arka_plan: String,
    pub panel_arka_plan: String,
    pub baslik_cubugu_arka_plan: String,
    pub baslik_cubugu_ayirici: String,

    // Etkilesimli eleman (buton vs.)
    pub eleman_arka_plan: String,
    pub eleman_hover: String,
    pub eleman_aktif: String,
    pub eleman_metin: String,

    // Metin
    pub metin: String,
    pub metin_sessiz: String,
    pub metin_yer_tutucu: String,

    // Ikon (kontrol butonlari ve genel ikon rengi)
    pub ikon: String,
    pub ikon_vurgu: String,
    pub ikon_kritik: String,

    // Kenarlik
    pub kenarlik: String,
    pub kenarlik_varyant: String,

    // Vurgu (accent)
    pub vurgu: String,
    pub vurgu_hover: String,

    // Golge
    pub golge: String,
    pub golge_seffaflik: f64,
}

/// Zed `StatusColors`'inin kisa versiyonu — editor-spesifik (modified,
/// conflict, hint, vb.) alanlar uygulama kapsaminda gereksiz.
#[derive(Deserialize, Serialize, Clone)]
pub struct DurumBolumu {
    pub basari: String,
    pub uyari: String,
    pub hata: String,
    pub bilgi: String,
}

// ── Yama (Refineable) tipleri ──────────────────────────────

/// Zed `Refineable` / `ThemeContent` patch modeli: her alan `Option<T>`;
/// bos birakilanlar `temel` varyantin degerinden kalitilir.
///
/// `temel = None` iken kaynak, `gorunum` alanina gore secilir (koyu →
/// "KavisNet Koyu", aydinlik → "KavisNet Aydinlik"). Boylece kullanici
/// sadece degistirmek istedigi alanlari yazar.
#[derive(Deserialize, Serialize, JsonSchema, Default, Clone)]
pub struct TemaVaryantYamasi {
    pub ad: String,
    /// Kalitim icin kaynak varyant adi. Yoksa `gorunum`'e gore varsayilan
    /// kullanilir.
    #[serde(default)]
    pub temel: Option<String>,
    #[serde(default)]
    pub gorunum: Option<Gorunum>,
    #[serde(default)]
    pub pencere: PencereYamasi,
    #[serde(default)]
    pub yerlesim: YerlesimYamasi,
    #[serde(default)]
    pub renkler: RenklerYamasi,
    #[serde(default)]
    pub durum: DurumYamasi,
}

#[derive(Deserialize, Serialize, JsonSchema, Default, Clone)]
pub struct PencereYamasi {
    #[serde(default, rename = "mod")]
    pub pencere_modu: Option<PencereModu>,
    #[serde(default)]
    pub arka_plan: Option<String>,
    #[serde(default)]
    pub blur_seffaflik: Option<f64>,
    #[serde(default)]
    pub seffaf_seffaflik: Option<f64>,
    #[serde(default)]
    pub kavis: Option<f64>,
}

#[derive(Deserialize, Serialize, JsonSchema, Default, Clone)]
pub struct YerlesimYamasi {
    #[serde(default)]
    pub ust_bar_yukseklik: Option<f64>,
    #[serde(default)]
    pub ust_bar_sol_bosluk: Option<f64>,
    #[serde(default)]
    pub sol_panel_min_genislik: Option<f64>,
    #[serde(default)]
    pub calisma_yuzeyi_kavis: Option<f64>,
    #[serde(default)]
    pub calisma_yuzeyi_kavisli_mi: Option<bool>,
    #[serde(default)]
    pub ust_sinir: Option<bool>,
}

#[derive(Deserialize, Serialize, JsonSchema, Default, Clone)]
pub struct RenklerYamasi {
    #[serde(default)]
    pub yuzey_arka_plan: Option<String>,
    #[serde(default)]
    pub yuksek_yuzey_arka_plan: Option<String>,
    #[serde(default)]
    pub panel_arka_plan: Option<String>,
    #[serde(default)]
    pub baslik_cubugu_arka_plan: Option<String>,
    #[serde(default)]
    pub baslik_cubugu_ayirici: Option<String>,
    #[serde(default)]
    pub eleman_arka_plan: Option<String>,
    #[serde(default)]
    pub eleman_hover: Option<String>,
    #[serde(default)]
    pub eleman_aktif: Option<String>,
    #[serde(default)]
    pub eleman_metin: Option<String>,
    #[serde(default)]
    pub metin: Option<String>,
    #[serde(default)]
    pub metin_sessiz: Option<String>,
    #[serde(default)]
    pub metin_yer_tutucu: Option<String>,
    #[serde(default)]
    pub ikon: Option<String>,
    #[serde(default)]
    pub ikon_vurgu: Option<String>,
    #[serde(default)]
    pub ikon_kritik: Option<String>,
    #[serde(default)]
    pub kenarlik: Option<String>,
    #[serde(default)]
    pub kenarlik_varyant: Option<String>,
    #[serde(default)]
    pub vurgu: Option<String>,
    #[serde(default)]
    pub vurgu_hover: Option<String>,
    #[serde(default)]
    pub golge: Option<String>,
    #[serde(default)]
    pub golge_seffaflik: Option<f64>,
}

#[derive(Deserialize, Serialize, JsonSchema, Default, Clone)]
pub struct DurumYamasi {
    #[serde(default)]
    pub basari: Option<String>,
    #[serde(default)]
    pub uyari: Option<String>,
    #[serde(default)]
    pub hata: Option<String>,
    #[serde(default)]
    pub bilgi: Option<String>,
}

// ── Yama uygulama (refine) ────────────────────────────────

impl PencereBolumu {
    fn yama_uygula(&mut self, y: &PencereYamasi) {
        if let Some(v) = y.pencere_modu { self.pencere_modu = v; }
        if let Some(v) = &y.arka_plan { self.arka_plan = v.clone(); }
        if let Some(v) = y.blur_seffaflik { self.blur_seffaflik = v; }
        if let Some(v) = y.seffaf_seffaflik { self.seffaf_seffaflik = v; }
        if let Some(v) = y.kavis { self.kavis = v; }
    }
}

impl YerlesimBolumu {
    fn yama_uygula(&mut self, y: &YerlesimYamasi) {
        if let Some(v) = y.ust_bar_yukseklik { self.ust_bar_yukseklik = v; }
        if let Some(v) = y.ust_bar_sol_bosluk { self.ust_bar_sol_bosluk = v; }
        if let Some(v) = y.sol_panel_min_genislik { self.sol_panel_min_genislik = v; }
        if let Some(v) = y.calisma_yuzeyi_kavis { self.calisma_yuzeyi_kavis = v; }
        if let Some(v) = y.calisma_yuzeyi_kavisli_mi { self.calisma_yuzeyi_kavisli_mi = v; }
        if let Some(v) = y.ust_sinir { self.ust_sinir = v; }
    }
}

impl RenklerBolumu {
    fn yama_uygula(&mut self, y: &RenklerYamasi) {
        if let Some(v) = &y.yuzey_arka_plan { self.yuzey_arka_plan = v.clone(); }
        if let Some(v) = &y.yuksek_yuzey_arka_plan { self.yuksek_yuzey_arka_plan = v.clone(); }
        if let Some(v) = &y.panel_arka_plan { self.panel_arka_plan = v.clone(); }
        if let Some(v) = &y.baslik_cubugu_arka_plan { self.baslik_cubugu_arka_plan = v.clone(); }
        if let Some(v) = &y.baslik_cubugu_ayirici { self.baslik_cubugu_ayirici = v.clone(); }
        if let Some(v) = &y.eleman_arka_plan { self.eleman_arka_plan = v.clone(); }
        if let Some(v) = &y.eleman_hover { self.eleman_hover = v.clone(); }
        if let Some(v) = &y.eleman_aktif { self.eleman_aktif = v.clone(); }
        if let Some(v) = &y.eleman_metin { self.eleman_metin = v.clone(); }
        if let Some(v) = &y.metin { self.metin = v.clone(); }
        if let Some(v) = &y.metin_sessiz { self.metin_sessiz = v.clone(); }
        if let Some(v) = &y.metin_yer_tutucu { self.metin_yer_tutucu = v.clone(); }
        if let Some(v) = &y.ikon { self.ikon = v.clone(); }
        if let Some(v) = &y.ikon_vurgu { self.ikon_vurgu = v.clone(); }
        if let Some(v) = &y.ikon_kritik { self.ikon_kritik = v.clone(); }
        if let Some(v) = &y.kenarlik { self.kenarlik = v.clone(); }
        if let Some(v) = &y.kenarlik_varyant { self.kenarlik_varyant = v.clone(); }
        if let Some(v) = &y.vurgu { self.vurgu = v.clone(); }
        if let Some(v) = &y.vurgu_hover { self.vurgu_hover = v.clone(); }
        if let Some(v) = &y.golge { self.golge = v.clone(); }
        if let Some(v) = y.golge_seffaflik { self.golge_seffaflik = v; }
    }
}

impl DurumBolumu {
    fn yama_uygula(&mut self, y: &DurumYamasi) {
        if let Some(v) = &y.basari { self.basari = v.clone(); }
        if let Some(v) = &y.uyari { self.uyari = v.clone(); }
        if let Some(v) = &y.hata { self.hata = v.clone(); }
        if let Some(v) = &y.bilgi { self.bilgi = v.clone(); }
    }
}

impl TemaVaryantDosyasi {
    /// Yama alanlarindan dolu olanlari bu dosyanin uzerine yazar.
    /// `ad` her zaman yamanin adiyla degisir (varyant kimligi yamaya ait).
    fn yama_uygula(&mut self, y: &TemaVaryantYamasi) {
        self.ad = y.ad.clone();
        if let Some(g) = y.gorunum { self.gorunum = g; }
        self.pencere.yama_uygula(&y.pencere);
        self.yerlesim.yama_uygula(&y.yerlesim);
        self.renkler.yama_uygula(&y.renkler);
        self.durum.yama_uygula(&y.durum);
    }
}

impl TemaVaryantYamasi {
    /// Bir `TemaVaryantDosyasi`'ni tam dolu bir yamaya cevirir — dosya
    /// yazilirken (varsayilan ailesi) tum alanlarin gorunmesi icin.
    fn tamamen_dolu(d: &TemaVaryantDosyasi) -> Self {
        Self {
            ad: d.ad.clone(),
            temel: None,
            gorunum: Some(d.gorunum),
            pencere: PencereYamasi {
                pencere_modu: Some(d.pencere.pencere_modu),
                arka_plan: Some(d.pencere.arka_plan.clone()),
                blur_seffaflik: Some(d.pencere.blur_seffaflik),
                seffaf_seffaflik: Some(d.pencere.seffaf_seffaflik),
                kavis: Some(d.pencere.kavis),
            },
            yerlesim: YerlesimYamasi {
                ust_bar_yukseklik: Some(d.yerlesim.ust_bar_yukseklik),
                ust_bar_sol_bosluk: Some(d.yerlesim.ust_bar_sol_bosluk),
                sol_panel_min_genislik: Some(d.yerlesim.sol_panel_min_genislik),
                calisma_yuzeyi_kavis: Some(d.yerlesim.calisma_yuzeyi_kavis),
                calisma_yuzeyi_kavisli_mi: Some(d.yerlesim.calisma_yuzeyi_kavisli_mi),
                ust_sinir: Some(d.yerlesim.ust_sinir),
            },
            renkler: RenklerYamasi {
                yuzey_arka_plan: Some(d.renkler.yuzey_arka_plan.clone()),
                yuksek_yuzey_arka_plan: Some(d.renkler.yuksek_yuzey_arka_plan.clone()),
                panel_arka_plan: Some(d.renkler.panel_arka_plan.clone()),
                baslik_cubugu_arka_plan: Some(d.renkler.baslik_cubugu_arka_plan.clone()),
                baslik_cubugu_ayirici: Some(d.renkler.baslik_cubugu_ayirici.clone()),
                eleman_arka_plan: Some(d.renkler.eleman_arka_plan.clone()),
                eleman_hover: Some(d.renkler.eleman_hover.clone()),
                eleman_aktif: Some(d.renkler.eleman_aktif.clone()),
                eleman_metin: Some(d.renkler.eleman_metin.clone()),
                metin: Some(d.renkler.metin.clone()),
                metin_sessiz: Some(d.renkler.metin_sessiz.clone()),
                metin_yer_tutucu: Some(d.renkler.metin_yer_tutucu.clone()),
                ikon: Some(d.renkler.ikon.clone()),
                ikon_vurgu: Some(d.renkler.ikon_vurgu.clone()),
                ikon_kritik: Some(d.renkler.ikon_kritik.clone()),
                kenarlik: Some(d.renkler.kenarlik.clone()),
                kenarlik_varyant: Some(d.renkler.kenarlik_varyant.clone()),
                vurgu: Some(d.renkler.vurgu.clone()),
                vurgu_hover: Some(d.renkler.vurgu_hover.clone()),
                golge: Some(d.renkler.golge.clone()),
                golge_seffaflik: Some(d.renkler.golge_seffaflik),
            },
            durum: DurumYamasi {
                basari: Some(d.durum.basari.clone()),
                uyari: Some(d.durum.uyari.clone()),
                hata: Some(d.durum.hata.clone()),
                bilgi: Some(d.durum.bilgi.clone()),
            },
        }
    }
}

impl TemaAilesiDosyasi {
    /// Varsayilan tema ailesi: "KavisNet Koyu" + "KavisNet Aydinlik" tam
    /// yapilandirmali yamalar olarak. Kullanici silerek kalitima donebilir.
    pub fn varsayilan() -> Self {
        Self {
            ad: "KavisNet".into(),
            yazar: "KavisNet".into(),
            secim: TemaSecimi::Sistem {
                aydinlik: "KavisNet Aydinlik".into(),
                koyu: "KavisNet Koyu".into(),
            },
            varyantlar: vec![
                TemaVaryantYamasi::tamamen_dolu(&TemaVaryantDosyasi::varsayilan_koyu()),
                TemaVaryantYamasi::tamamen_dolu(&TemaVaryantDosyasi::varsayilan_aydinlik()),
            ],
        }
    }
}

impl TemaVaryantDosyasi {
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
                ust_bar_yukseklik: 40.0,
                ust_bar_sol_bosluk: varsayilan_ust_bar_sol_bosluk(),
                sol_panel_min_genislik: 220.0,
                calisma_yuzeyi_kavis: 10.0,
                calisma_yuzeyi_kavisli_mi: true,
                ust_sinir: true,
            },
            renkler: RenklerBolumu {
                yuzey_arka_plan: "#1E1E32".into(),
                yuksek_yuzey_arka_plan: "#24243A".into(),
                panel_arka_plan: "#141420".into(),
                baslik_cubugu_arka_plan: "#141420".into(),
                baslik_cubugu_ayirici: "#2A2A3E".into(),
                eleman_arka_plan: "#2A2A3E".into(),
                eleman_hover: "#3A3A4E".into(),
                eleman_aktif: "#4A4A5E".into(),
                eleman_metin: "#EDEDED".into(),
                metin: "#EDEDED".into(),
                metin_sessiz: "#B3B3B3".into(),
                metin_yer_tutucu: "#737373".into(),
                ikon: "#E8E8F0".into(),
                ikon_vurgu: "#B8C7FF".into(),
                ikon_kritik: "#FF7A7A".into(),
                kenarlik: "#2A2A3E".into(),
                kenarlik_varyant: "#222233".into(),
                vurgu: "#5599DD".into(),
                vurgu_hover: "#66AAEE".into(),
                golge: "#000000".into(),
                golge_seffaflik: 0.40,
            },
            durum: DurumBolumu {
                basari: "#55BB77".into(),
                uyari: "#DDAA33".into(),
                hata: "#DD5555".into(),
                bilgi: "#5599DD".into(),
            },
        }
    }

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
                ust_bar_yukseklik: 40.0,
                ust_bar_sol_bosluk: varsayilan_ust_bar_sol_bosluk(),
                sol_panel_min_genislik: 220.0,
                calisma_yuzeyi_kavis: 10.0,
                calisma_yuzeyi_kavisli_mi: true,
                ust_sinir: true,
            },
            renkler: RenklerBolumu {
                yuzey_arka_plan: "#FFFFFF".into(),
                yuksek_yuzey_arka_plan: "#F8F8FC".into(),
                panel_arka_plan: "#ECECF5".into(),
                baslik_cubugu_arka_plan: "#ECECF5".into(),
                baslik_cubugu_ayirici: "#D0D0DD".into(),
                eleman_arka_plan: "#DDDDEE".into(),
                eleman_hover: "#CCCCDD".into(),
                eleman_aktif: "#BBBBCC".into(),
                eleman_metin: "#1A1A2E".into(),
                metin: "#1A1A2E".into(),
                metin_sessiz: "#4A4A5E".into(),
                metin_yer_tutucu: "#8A8A9E".into(),
                ikon: "#1A1A2E".into(),
                ikon_vurgu: "#3355BB".into(),
                ikon_kritik: "#DD3333".into(),
                kenarlik: "#D0D0DD".into(),
                kenarlik_varyant: "#E0E0EE".into(),
                vurgu: "#3377CC".into(),
                vurgu_hover: "#2266BB".into(),
                golge: "#000000".into(),
                golge_seffaflik: 0.20,
            },
            durum: DurumBolumu {
                basari: "#33AA55".into(),
                uyari: "#CC8800".into(),
                hata: "#CC3333".into(),
                bilgi: "#3377CC".into(),
            },
        }
    }
}

// ── Tema dosya yolu ────────────────────────────────────────

pub fn tema_dosya_yolu() -> PathBuf {
    let mut yol = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    yol.push("KavisNet");
    yol.push("tema.toml");
    yol
}

pub fn tema_schema_dosya_yolu() -> PathBuf {
    tema_dosya_yolu().with_file_name("tema.schema.json")
}

// ── JSON Schema uretimi ────────────────────────────────────

/// Tema dosyasinin JSON Schema'sini uretir. Taplo ve benzeri TOML
/// editorleri `tema.schema.json` + `#:schema ./tema.schema.json`
/// direktifiyle tema.toml'u dogrular ve otomatik tamamlama sunar.
///
/// Zed'in `schemars::schema_for!` kullanimiyla ayni desen — kaynak
/// kod guncellendikce schema'yi yeniden yazmak gereksiz runtime
/// surprizi birakmaz.
pub fn tema_schema() -> schemars::Schema {
    schemars::schema_for!(TemaAilesiDosyasi)
}

fn tema_schema_yaz(hedef: &Path) {
    match serde_json::to_string_pretty(&tema_schema()) {
        Ok(icerik) => {
            if let Some(dizin) = hedef.parent() {
                let _ = std::fs::create_dir_all(dizin);
            }
            if let Err(e) = std::fs::write(hedef, icerik) {
                hatayi_kaydet(&format!("tema.schema.json yazilamadi: {e}"));
            }
        }
        Err(e) => hatayi_kaydet(&format!("tema.schema.json serilestirilemedi: {e}")),
    }
}

// ── Calisma zamani tema yapisi ─────────────────────────────

/// Zed `ThemeStyles` esdegeri: bir temanin tum semantik renk gruplari.
/// `renkler` = `ThemeColors`, `durum` = `StatusColors`. Flat alan yerine
/// gruplama: `cx.tema().renkler.metin`, `cx.tema().durum.hata`.
#[derive(Clone)]
pub struct Tema {
    // Kimlik
    pub ad: SharedString,
    pub gorunum: Gorunum,

    // Pencere kok (platform pencere moduna gore alpha ve kavis hesaplanir)
    pub pencere_gorunum: WindowBackgroundAppearance,
    pub arka_plan: Hsla,
    pub pencere_kavis: Pixels,

    pub yerlesim: YerlesimBoyutlari,
    pub renkler: TemaRenkleri,
    pub durum: DurumRenkleri,
}

/// Boyutsal ve davranissal UI bayraklari. Zed'de bu degerler `UiSettings`
/// altinda; biz tek tema dosyasinda tuttugumuz icin tema icinde.
#[derive(Clone, Copy)]
pub struct YerlesimBoyutlari {
    pub ust_bar_yukseklik: Pixels,
    pub ust_bar_sol_bosluk: Pixels,
    pub sol_panel_min_genislik: Pixels,
    pub calisma_yuzeyi_kavis: Pixels,
    pub calisma_yuzeyi_kavisli_mi: bool,
    pub ust_sinir: bool,
}

/// Zed `ThemeColors` esdegeri — UI widget/yuzey/metin/ikon renkleri.
#[derive(Clone, Copy)]
pub struct TemaRenkleri {
    // Yuzeyler
    pub yuzey_arka_plan: Hsla,
    pub yuksek_yuzey_arka_plan: Hsla,
    pub panel_arka_plan: Hsla,
    pub baslik_cubugu_arka_plan: Hsla,
    pub baslik_cubugu_ayirici: Hsla,

    // Etkilesimli eleman
    pub eleman_arka_plan: Hsla,
    pub eleman_hover: Hsla,
    pub eleman_aktif: Hsla,
    pub eleman_metin: Hsla,

    // Metin
    pub metin: Hsla,
    pub metin_sessiz: Hsla,
    pub metin_yer_tutucu: Hsla,

    // Ikon
    pub ikon: Hsla,
    pub ikon_vurgu: Hsla,
    pub ikon_kritik: Hsla,

    // Kenarlik
    pub kenarlik: Hsla,
    pub kenarlik_varyant: Hsla,

    // Vurgu
    pub vurgu: Hsla,
    pub vurgu_hover: Hsla,

    // Golge (alpha dahil)
    pub golge: Hsla,
}

/// Zed `StatusColors` esdegeri.
#[derive(Clone, Copy)]
pub struct DurumRenkleri {
    pub basari: Hsla,
    pub uyari: Hsla,
    pub hata: Hsla,
    pub bilgi: Hsla,
}

impl Tema {
    /// Tek bir varyanttan calisma zamani `Tema`'sini olusturur.
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

        let mut arka_plan = hex_renk(&d.pencere.arka_plan);
        match pencere_gorunum {
            WindowBackgroundAppearance::Blurred => {
                arka_plan.a = d.pencere.blur_seffaflik as f32;
            }
            WindowBackgroundAppearance::Transparent => {
                arka_plan.a = d.pencere.seffaf_seffaflik as f32;
            }
            _ => {
                arka_plan.a = 1.0;
            }
        }

        let pencere_kavis = match pencere_gorunum {
            WindowBackgroundAppearance::Opaque => px(0.),
            _ => px(d.pencere.kavis as f32),
        };

        let mut golge = hex_renk(&d.renkler.golge);
        golge.a = d.renkler.golge_seffaflik as f32;

        Self {
            ad: SharedString::from(d.ad.clone()),
            gorunum: d.gorunum,

            pencere_gorunum,
            arka_plan,
            pencere_kavis,

            yerlesim: YerlesimBoyutlari {
                ust_bar_yukseklik: px(d.yerlesim.ust_bar_yukseklik as f32),
                ust_bar_sol_bosluk: px(d.yerlesim.ust_bar_sol_bosluk as f32),
                sol_panel_min_genislik: px(d.yerlesim.sol_panel_min_genislik as f32),
                calisma_yuzeyi_kavis: px(d.yerlesim.calisma_yuzeyi_kavis as f32),
                calisma_yuzeyi_kavisli_mi: d.yerlesim.calisma_yuzeyi_kavisli_mi,
                ust_sinir: d.yerlesim.ust_sinir,
            },

            renkler: TemaRenkleri {
                yuzey_arka_plan: hex_renk(&d.renkler.yuzey_arka_plan),
                yuksek_yuzey_arka_plan: hex_renk(&d.renkler.yuksek_yuzey_arka_plan),
                panel_arka_plan: hex_renk(&d.renkler.panel_arka_plan),
                baslik_cubugu_arka_plan: hex_renk(&d.renkler.baslik_cubugu_arka_plan),
                baslik_cubugu_ayirici: hex_renk(&d.renkler.baslik_cubugu_ayirici),

                eleman_arka_plan: hex_renk(&d.renkler.eleman_arka_plan),
                eleman_hover: hex_renk(&d.renkler.eleman_hover),
                eleman_aktif: hex_renk(&d.renkler.eleman_aktif),
                eleman_metin: hex_renk(&d.renkler.eleman_metin),

                metin: hex_renk(&d.renkler.metin),
                metin_sessiz: hex_renk(&d.renkler.metin_sessiz),
                metin_yer_tutucu: hex_renk(&d.renkler.metin_yer_tutucu),

                ikon: hex_renk(&d.renkler.ikon),
                ikon_vurgu: hex_renk(&d.renkler.ikon_vurgu),
                ikon_kritik: hex_renk(&d.renkler.ikon_kritik),

                kenarlik: hex_renk(&d.renkler.kenarlik),
                kenarlik_varyant: hex_renk(&d.renkler.kenarlik_varyant),

                vurgu: hex_renk(&d.renkler.vurgu),
                vurgu_hover: hex_renk(&d.renkler.vurgu_hover),

                golge,
            },

            durum: DurumRenkleri {
                basari: hex_renk(&d.durum.basari),
                uyari: hex_renk(&d.durum.uyari),
                hata: hex_renk(&d.durum.hata),
                bilgi: hex_renk(&d.durum.bilgi),
            },
        }
    }
}

impl Global for Tema {}

// ── AktifTema (Zed `ActiveTheme` esdegeri) ──────────────────

pub trait AktifTema {
    fn tema(&self) -> &Tema;
}

impl AktifTema for App {
    fn tema(&self) -> &Tema {
        self.global::<Tema>()
    }
}

// ── TemaKaydi (Zed `ThemeRegistry` esdegeri) ────────────────

pub struct TemaKaydi {
    temalar: Vec<Tema>,
}

impl TemaKaydi {
    pub fn yeni_bos() -> Self {
        Self { temalar: Vec::new() }
    }

    pub fn varsayilan_ile() -> Self {
        let mut kayit = Self::yeni_bos();
        kayit.kaydet(Tema::varyanttan_olustur(&TemaVaryantDosyasi::varsayilan_koyu()));
        kayit.kaydet(Tema::varyanttan_olustur(&TemaVaryantDosyasi::varsayilan_aydinlik()));
        kayit
    }

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

    pub fn gorunume_gore(&self, gorunum: Gorunum) -> Vec<&Tema> {
        self.temalar
            .iter()
            .filter(|t| t.gorunum == gorunum)
            .collect()
    }

    fn ilk(&self) -> Option<&Tema> {
        self.temalar.first()
    }

    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }
}

impl Global for TemaKaydi {}

// ── Tema kurulumu ve yukleme ───────────────────────────────

pub fn kurulum(cx: &mut App) {
    // Schema'yi her baslangicta yenile — kod guncellendiginde editor
    // tarafi da guncel kalsin. Cok kucuk bir JSON, maliyet ihmal edilebilir.
    tema_schema_yaz(&tema_schema_dosya_yolu());

    let sistem = SistemGorunumu::tespit_et();
    let (kayit, aktif) = yukleme_bileseni(sistem.0);
    cx.set_global(sistem);
    cx.set_global(kayit);
    cx.set_global(aktif);
}

fn yukleme_bileseni(sistem: Gorunum) -> (TemaKaydi, Tema) {
    let yol = tema_dosya_yolu();
    let aile = aileyi_yukle_veya_yaz(&yol);

    let kayit = yamalardan_kayit_olustur(&aile.varyantlar);
    let aktif = secim_ile_aktif_tema(&aile.secim, sistem, &kayit);
    (kayit, aktif)
}

/// Yerlesik varsayilanlarla baslayip sirayla her yamayi uygun temel
/// uzerine birlestirir (Zed `Refineable::refine` akisi). Onceki yamalar
/// sonraki yamalara `temel` olabilir — zincirleme kalitim desteklenir.
fn yamalardan_kayit_olustur(yamalar: &[TemaVaryantYamasi]) -> TemaKaydi {
    let mut kayit = TemaKaydi::varsayilan_ile();
    let mut cozulen: Vec<TemaVaryantDosyasi> = vec![
        TemaVaryantDosyasi::varsayilan_koyu(),
        TemaVaryantDosyasi::varsayilan_aydinlik(),
    ];

    for yama in yamalar {
        let temel_adi = yama
            .temel
            .clone()
            .unwrap_or_else(|| match yama.gorunum.unwrap_or_default() {
                Gorunum::Koyu => "KavisNet Koyu".into(),
                Gorunum::Aydinlik => "KavisNet Aydinlik".into(),
            });

        let mut dosya = cozulen
            .iter()
            .find(|d| d.ad == temel_adi)
            .cloned()
            .unwrap_or_else(|| {
                hatayi_kaydet(&format!(
                    "'{}' yamasinin temeli '{}' bulunamadi; 'KavisNet Koyu' kullaniliyor.",
                    yama.ad, temel_adi
                ));
                TemaVaryantDosyasi::varsayilan_koyu()
            });

        dosya.yama_uygula(yama);

        kayit.kaydet(Tema::varyanttan_olustur(&dosya));

        if let Some(yer) = cozulen.iter().position(|d| d.ad == dosya.ad) {
            cozulen[yer] = dosya;
        } else {
            cozulen.push(dosya);
        }
    }

    kayit
}

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
#:schema ./tema.schema.json
# KavisNet Tema Dosyasi
#
# Ustteki `#:schema` direktifi Taplo/evensen.vscode-toml gibi TOML
# editorleri tarafindan okunur; yanlis alan adi, gecersiz enum degeri
# vb. icin anlik uyari + otomatik tamamlama sunar. Schema dosyasi
# uygulama her baslatildiginda yanindaki `tema.schema.json` olarak
# yeniden yazilir.
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
# YAMA (Refinement) destegi:
#   Her varyant bir YAMADIR — istemedigin alanlari silerek `temel`
#   varyanttan kalitabilirsin. Minimal ornek:
#
#     [[varyantlar]]
#     ad = \"Mavi Gece\"
#     temel = \"KavisNet Koyu\"    # (ops.) kaynak varyant; yoksa gorunum
#                                  #        alanina gore otomatik secilir
#     gorunum = \"koyu\"
#
#     [varyantlar.renkler]
#     vurgu = \"#4A90E2\"          # sadece degistirmek istedigin alan
#
#   Asagidaki iki yerlesik varyant tam yapilandirma ile yazildi; ancak
#   istedigin alanlari silerek varsayilana donebilirsin.
#
# Renk kategorileri (Zed ThemeColors/StatusColors tabanli):
#   [varyantlar.renkler] — UI widget, yuzey, metin, ikon, kenarlik, vurgu
#   [varyantlar.durum]   — basari/uyari/hata/bilgi
#
# Pencere modu:
#   \"otomatik\" - Sistem blur destekliyorsa blur, yoksa seffaf
#   \"seffaf\"   - Her zaman seffaf (blur yok)
#   \"opak\"     - Her zaman opak
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
                            let aile = TemaAilesiDosyasi::varsayilan();
                            let kayit = yamalardan_kayit_olustur(&aile.varyantlar);
                            let aktif = secim_ile_aktif_tema(&aile.secim, sistem, &kayit);
                            cx.set_global(kayit);
                            cx.set_global(aktif);
                            println!("Tema dosyasi bulunamadi, varsayilan temaya donuldu.");
                        });
                    }
                }
            }

            // Sistem aydinlik/koyu takibi burada yapilmiyor — pencere acilinca
            // `pencere_gorunumunu_uygula()` + `window.observe_window_appearance`
            // GPUI'nin platform-native kanali uzerinden gercek zamanli sinyali
            // dinliyor (macOS viewDidChangeEffectiveAppearance, Linux
            // xdg-desktop-portal color-scheme). `dark_light::detect()` polling'i
            // GNOME 42+ / macOS thread-local cache sorunlarindan oturu terk
            // edildi.
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
        // kalıyor.
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
