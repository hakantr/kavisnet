use gpui::prelude::*;
use gpui::*;
use ortak_tema::Tema;

#[cfg(target_os = "linux")]
use crate::kontroller_linux::linux_kontrol_duzeni;
#[cfg(target_os = "linux")]
use crate::pencere::kapatma_istegi;

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum KontrolTipi {
    Kucult,
    Buyut,
    Kapat,
}

#[derive(Clone, Copy)]
pub enum KontrolTarafi {
    Sol,
    Sag,
}

#[allow(dead_code)]
impl KontrolTipi {
    /// Kontrol butonunun SVG yol adını döndürür. Yol `ortak_ikonlar`
    /// `AssetSource` kaydında aranır; GPUI SVG'yi monokrom alpha mask olarak
    /// render eder, bu yüzden `text_color` ile verilen renk tüm çizime
    /// uygulanır (Zed `IconName::Generic*` ile aynı yaklaşım).
    pub fn ikon_yolu(&self, pencere_buyuk_mu: bool) -> &'static str {
        match self {
            Self::Kucult => "ikonlar/kontrol_kucult.svg",
            Self::Buyut if pencere_buyuk_mu => "ikonlar/kontrol_restore.svg",
            Self::Buyut => "ikonlar/kontrol_buyut.svg",
            Self::Kapat => "ikonlar/kontrol_kapat.svg",
        }
    }

    pub fn grup_adi(&self) -> &'static str {
        match self {
            Self::Kucult => "kontrol-kucult",
            Self::Buyut => "kontrol-buyut",
            Self::Kapat => "kontrol-kapat",
        }
    }

    #[cfg(target_os = "linux")]
    pub fn gnome_adindan_coz(ad: &str) -> Option<Self> {
        match ad {
            "minimize" => Some(Self::Kucult),
            "maximize" => Some(Self::Buyut),
            "close" => Some(Self::Kapat),
            _ => None,
        }
    }

    #[cfg(target_os = "linux")]
    pub fn siralama_indeksi(&self) -> usize {
        match self {
            Self::Kucult => 0,
            Self::Buyut => 1,
            Self::Kapat => 2,
        }
    }

    pub fn window_control(&self) -> WindowControlArea {
        match self {
            Self::Kucult => WindowControlArea::Min,
            Self::Buyut => WindowControlArea::Max,
            Self::Kapat => WindowControlArea::Close,
        }
    }
}

#[allow(dead_code)]
fn kontrol_destekleniyor_mu(tip: KontrolTipi, kontroller: WindowControls) -> bool {
    match tip {
        KontrolTipi::Kucult => kontroller.minimize,
        KontrolTipi::Buyut => kontroller.maximize,
        KontrolTipi::Kapat => true,
    }
}

#[allow(dead_code)]
fn kontrol_butonu(
    tip: KontrolTipi,
    tema: &Tema,
    pencere_buyuk_mu: bool,
) -> Stateful<Div> {
    let ikon_hover = match tip {
        KontrolTipi::Kapat => tema.renkler.ikon_kritik,
        _ => tema.renkler.ikon_vurgu,
    };
    // Zemin hover için aynı vurgu rengini düşük alpha ile kullan: kavisli
    // pencere köşelerinde daire buton boyutundan (20px) küçük kaldığı için
    // border-radius dışına taşmaz. Zed'in ghost_element_hover eşdeğeri.
    #[cfg(target_os = "linux")]
    let zemin_hover = {
        let mut renk = ikon_hover;
        renk.a = 0.18;
        renk
    };
    let metin_rengi = tema.renkler.ikon;
    let grup_adi = SharedString::from(tip.grup_adi());

    // Zed `IconName::Generic*` ile aynı: 16x16 viewBox'lı SVG, size_4() ile
    // 16px kareye çizilir. text_color SVG'yi alpha mask olarak tutup renk
    // verir — aynı dosya hem normal hem hover'da kullanılır.
    let ikon_yolu = SharedString::from(tip.ikon_yolu(pencere_buyuk_mu));
    let ikon = svg()
        .size_4()
        .flex_none()
        .path(ikon_yolu)
        .text_color(metin_rengi)
        .group_hover(grup_adi.clone(), move |s| s.text_color(ikon_hover));

    // Windows/eski yerleşim: geniş dikdörtgen buton (46px) — native stil.
    // Linux: Zed'in `platform_linux::WindowControl` pattern'i — 20x20 daire.
    #[cfg(not(target_os = "linux"))]
    let base = div()
        .id(grup_adi.clone())
        .group(grup_adi.clone())
        .flex()
        .items_center()
        .justify_center()
        .w(px(46.))
        .h_full()
        .on_mouse_move(|_, _, cx| cx.stop_propagation())
        .child(ikon);

    // Zed `WindowControl::render` ile birebir: `.group("")` + container
    // `.hover(|s| s.bg(bg_hover))` + child `.group_hover("", …)` ikon rengi.
    // 20x20 rounded_full daire, buton etrafındaki kavise taşmadan hover zemini
    // gösterir.
    #[cfg(target_os = "linux")]
    let base = div()
        .id(grup_adi.clone())
        .group(grup_adi.clone())
        .flex()
        .items_center()
        .justify_center()
        .w_5()
        .h_5()
        .rounded_full()
        .cursor_pointer()
        .hover(move |s| s.bg(zemin_hover))
        .on_mouse_move(|_, _, cx| cx.stop_propagation())
        .child(ikon);

    // Windows: `.occlude()` sart — parent ust_bar `WindowControlArea::Drag`
    // hitbox'i cocuk butonu ortecegi icin NCHITTEST yanlis geri doner
    // (HTCAPTION). `.occlude()` child hitbox'i `BlockMouse` yapip
    // `Window::hit_test` ters iterasyonda parent'tan once kesiyor; sonucta
    // cursor buton uzerindeyken sadece `HTMINBUTTON/HTMAXBUTTON/HTCLOSE`
    // donuyor ve Windows native buton davranisi (hover + click) calisiyor.
    #[cfg(target_os = "windows")]
    let base = base
        .occlude()
        .window_control_area(tip.window_control());

    // Zed `platform_title_bar::platforms::platform_linux::WindowControl` ile
    // birebir aynı davranış: mouse_down'a dokunulmaz (prevent_default
    // tıklamayı öldürüyordu), click event'i cx.stop_propagation() + işlem.
    // Container row'unun mouse_down'ı parent drag'ını engellemek için ayrıca
    // `pencere_kontrolleri_taraf` içinde stop_propagation ediliyor.
    #[cfg(target_os = "linux")]
    let base = base.on_click(move |_, window, cx| {
        cx.stop_propagation();
        match tip {
            KontrolTipi::Kucult => window.minimize_window(),
            KontrolTipi::Buyut => window.zoom_window(),
            KontrolTipi::Kapat => {
                if kapatma_istegi(window, cx) {
                    cx.quit();
                }
            }
        }
    });

    base
}

#[cfg_attr(target_os = "macos", allow(unused_mut, unreachable_code))]
pub fn pencere_kontrolleri_taraf(
    taraf: KontrolTarafi,
    window: &Window,
    tema: &Tema,
) -> Stateful<Div> {
    let id = match taraf {
        KontrolTarafi::Sol => "window-controls-sol",
        KontrolTarafi::Sag => "window-controls-sag",
    };

    let mut satir = div()
        .id(id)
        .flex()
        .flex_row()
        .items_center()
        .flex_shrink_0()
        .h_full();

    #[cfg(target_os = "macos")]
    {
        let _ = (window, tema);
        return satir;
    }

    let desteklenen = window.window_controls();
    let pencere_buyuk_mu = window.is_maximized();

    #[cfg(target_os = "windows")]
    {
        if matches!(taraf, KontrolTarafi::Sol) {
            return satir;
        }

        for tip in [
            KontrolTipi::Kucult,
            KontrolTipi::Buyut,
            KontrolTipi::Kapat,
        ] {
            if kontrol_destekleniyor_mu(tip, desteklenen) {
                satir = satir.child(kontrol_butonu(tip, tema, pencere_buyuk_mu));
            }
        }

        return satir;
    }

    #[cfg(target_os = "linux")]
    {
        if !matches!(window.window_decorations(), Decorations::Client { .. }) {
            return satir;
        }

        let duzen = linux_kontrol_duzeni();
        let secilen = match taraf {
            KontrolTarafi::Sol => duzen.sol,
            KontrolTarafi::Sag => duzen.sag,
        };

        let mut buton_sayisi = 0;
        for tip in secilen.into_iter().flatten() {
            if kontrol_destekleniyor_mu(tip, desteklenen) {
                satir = satir.child(kontrol_butonu(tip, tema, pencere_buyuk_mu));
                buton_sayisi += 1;
            }
        }

        // Zed `LinuxWindowControls` ile aynı: butonlu container'da sol-tık
        // event'ini üst bar'a geçirme (drag tetiklenmesin) + gap_3 px_3 ile
        // 20x20 daireleri yan yana dizme.
        if buton_sayisi > 0 {
            satir = satir
                .gap_3()
                .px_3()
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation());
        }
    }

    satir
}
