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
    pub fn simge(&self, pencere_buyuk_mu: bool) -> &'static str {
        match self {
            Self::Kucult => "\u{2013}",
            Self::Buyut if pencere_buyuk_mu => "\u{2750}",
            Self::Buyut => "\u{25A1}",
            Self::Kapat => "\u{2715}",
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
    let hover_renk = match tip {
        KontrolTipi::Kapat => tema.kontrol_kapat_hover,
        _ => tema.kontrol_hover,
    };
    let metin_rengi = tema.ust_bar_metin;
    let grup_adi = SharedString::from(tip.grup_adi());

    let base = div()
        .id(grup_adi.clone())
        .group(grup_adi.clone())
        .flex()
        .items_center()
        .justify_center()
        .w(px(46.))
        .h_full()
        .text_size(px(13.))
        .on_mouse_move(|_, _, cx| cx.stop_propagation())
        .child(
            div()
                .text_color(metin_rengi)
                .group_hover(grup_adi, move |s| s.text_color(hover_renk))
                .child(tip.simge(pencere_buyuk_mu)),
        );

    #[cfg(target_os = "windows")]
    let base = base.window_control_area(tip.window_control());

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
        // event'ini üst bar'a geçirme (drag tetiklenmesin).
        if buton_sayisi > 0 {
            satir =
                satir.on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation());
        }
    }

    satir
}
