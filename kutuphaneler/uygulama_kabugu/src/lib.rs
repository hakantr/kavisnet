use gpui::prelude::*;
use gpui::*;
use ortak_tema::Tema;
use sol_menu::SolMenu;
#[cfg(target_os = "linux")]
use std::process::Command;
#[cfg(target_os = "linux")]
use std::sync::{Mutex, OnceLock};
#[cfg(target_os = "linux")]
use std::time::{Duration, Instant};

// ── Ana Panel (Uygulamanın Kök Bileşeni) ──────────────────

pub struct AnaPanel {
    pub ust_bar: UstBar,
    pub sol_menu: SolMenu,
    pub calisma_yuzeyi: CalismaYuzeyi,
    son_gorunum: Option<WindowBackgroundAppearance>,
}

impl AnaPanel {
    pub fn new() -> Self {
        Self {
            ust_bar: UstBar,
            sol_menu: SolMenu::new(),
            calisma_yuzeyi: CalismaYuzeyi::new(),
            son_gorunum: None,
        }
    }
}

/// Pencere gölgesi boyutu (CSD modunda).
const GOLGE_BOYUTU: Pixels = px(10.0);

impl Render for AnaPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tema = cx.global::<Tema>();
        self.sol_menu.baslangic_genisligi_ayarla(tema);

        // Sadece degisiklik oldugunda cagir; aksi halde Wayland'da
        // update_window → re-render → set_background_appearance sonsuz
        // dongusu olusur ve CPU kullanimi artar.
        if self.son_gorunum != Some(tema.pencere_gorunum) {
            window.set_background_appearance(tema.pencere_gorunum);
            self.son_gorunum = Some(tema.pencere_gorunum);
        }

        let dekorasyon = window.window_decorations();
        match dekorasyon {
            Decorations::Client { .. } => window.set_client_inset(GOLGE_BOYUTU),
            Decorations::Server => window.set_client_inset(px(0.0)),
        }

        let icerik_satiri = div()
            .flex_1()
            .flex()
            .flex_row()
            .overflow_hidden()
            .child(self.sol_menu.render(tema))
            .child(self.calisma_yuzeyi.render(tema));

        let base = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(tema.pencere_arka_plan)
            .rounded(tema.pencere_kavis)
            .border_1()
            .border_color(tema.kenarlik)
            .overflow_hidden();

        let base = if tema.ust_sinir {
            base.relative().child(icerik_satiri).child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .child(self.ust_bar.render(window, tema)),
            )
        } else {
            base.child(self.ust_bar.render(window, tema)).child(icerik_satiri)
        };

        div()
            .id("window-backdrop")
            .bg(transparent_black())
            .size_full()
            .map(|div| match dekorasyon {
                Decorations::Server => div,
                Decorations::Client { tiling, .. } => {
                    let golge = GOLGE_BOYUTU;
                    div.child(
                        canvas(
                            |_bounds, window, _cx| {
                                window.insert_hitbox(
                                    Bounds::new(
                                        point(px(0.0), px(0.0)),
                                        window.window_bounds().get_bounds().size,
                                    ),
                                    HitboxBehavior::Normal,
                                )
                            },
                            move |_bounds, hitbox, window, _cx| {
                                let fare = window.mouse_position();
                                let boyut = window.window_bounds().get_bounds().size;
                                let Some(kenar) =
                                    golge_kenar_bul(fare, golge, boyut)
                                else {
                                    return;
                                };
                                window.set_cursor_style(
                                    match kenar {
                                        ResizeEdge::Top | ResizeEdge::Bottom => {
                                            CursorStyle::ResizeUpDown
                                        }
                                        ResizeEdge::Left | ResizeEdge::Right => {
                                            CursorStyle::ResizeLeftRight
                                        }
                                        ResizeEdge::TopLeft | ResizeEdge::BottomRight => {
                                            CursorStyle::ResizeUpLeftDownRight
                                        }
                                        ResizeEdge::TopRight | ResizeEdge::BottomLeft => {
                                            CursorStyle::ResizeUpRightDownLeft
                                        }
                                    },
                                    &hitbox,
                                );
                            },
                        )
                        .size_full()
                        .absolute(),
                    )
                    .when(!tiling.top, |div| div.pt(golge))
                    .when(!tiling.bottom, |div| div.pb(golge))
                    .when(!tiling.left, |div| div.pl(golge))
                    .when(!tiling.right, |div| div.pr(golge))
                    .on_mouse_move(|_e, window, _cx| window.refresh())
                    .on_mouse_down(MouseButton::Left, move |e, window, _cx| {
                        let boyut = window.window_bounds().get_bounds().size;
                        if let Some(kenar) = golge_kenar_bul(e.position, golge, boyut)
                        {
                            window.start_window_resize(kenar);
                        }
                    })
                }
            })
            .child(
                div()
                    .size_full()
                    .map(|div| match dekorasyon {
                        Decorations::Server => div,
                        Decorations::Client { tiling } => {
                            div.when(!tiling.is_tiled(), |div| {
                                div.shadow(vec![BoxShadow {
                                    color: Hsla {
                                        h: 0.,
                                        s: 0.,
                                        l: 0.,
                                        a: 0.4,
                                    },
                                    blur_radius: GOLGE_BOYUTU / 2.,
                                    spread_radius: px(0.),
                                    offset: point(px(0.0), px(0.0)),
                                }])
                            })
                        }
                    })
                    .on_mouse_move(|_e, _, cx| cx.stop_propagation())
                    .child(base),
            )
    }
}

fn golge_kenar_bul(
    pos: Point<Pixels>,
    golge: Pixels,
    boyut: Size<Pixels>,
) -> Option<ResizeEdge> {
    let kenar = if pos.y < golge && pos.x < golge {
        ResizeEdge::TopLeft
    } else if pos.y < golge && pos.x > boyut.width - golge {
        ResizeEdge::TopRight
    } else if pos.y < golge {
        ResizeEdge::Top
    } else if pos.y > boyut.height - golge && pos.x < golge {
        ResizeEdge::BottomLeft
    } else if pos.y > boyut.height - golge && pos.x > boyut.width - golge {
        ResizeEdge::BottomRight
    } else if pos.y > boyut.height - golge {
        ResizeEdge::Bottom
    } else if pos.x < golge {
        ResizeEdge::Left
    } else if pos.x > boyut.width - golge {
        ResizeEdge::Right
    } else {
        return None;
    };
    Some(kenar)
}

/// Uygulama app_id (Linux'ta pencere ↔ .desktop eslesmesi icin).
pub const UYGULAMA_APP_ID: &str = "KavisNet";

/// Uygulamanın ana penceresini açar ve yapılandırır.
pub fn ana_pencere_ac(cx: &mut App) {
    let tema = *cx.global::<Tema>();

    #[cfg(target_os = "linux")]
    linux_ikon_kur();

    cx.spawn(async move |cx| {
        let options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                appears_transparent: true,
                traffic_light_position: Some(point(px(8.), px(12.))),
                ..Default::default()
            }),
            window_background: tema.pencere_gorunum,
            window_decorations: Some(WindowDecorations::Client),
            is_resizable: true,
            app_id: Some(UYGULAMA_APP_ID.to_string()),
            ..Default::default()
        };

        let window_handle = cx
            .open_window(options, |_window, cx| cx.new(|_cx| AnaPanel::new()))
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

// ── Pencere kontrol butonlari (Windows / Linux) ────────────

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum KontrolTipi {
    Kucult,
    Buyut,
    Kapat,
}

#[derive(Clone, Copy)]
enum KontrolTarafi {
    Sol,
    Sag,
}

#[cfg(target_os = "linux")]
const KONTROL_BUTON_SINIRI: usize = 3;

#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
struct KontrolDuzeni {
    sol: [Option<KontrolTipi>; KONTROL_BUTON_SINIRI],
    sag: [Option<KontrolTipi>; KONTROL_BUTON_SINIRI],
}

#[cfg(target_os = "linux")]
impl KontrolDuzeni {
    fn standart() -> Self {
        Self {
            sol: [None; KONTROL_BUTON_SINIRI],
            sag: [
                Some(KontrolTipi::Kucult),
                Some(KontrolTipi::Buyut),
                Some(KontrolTipi::Kapat),
            ],
        }
    }

    fn metinden_coz(metin: &str) -> Option<Self> {
        let (sol_metin, sag_metin) = metin.split_once(':').unwrap_or(("", metin));

        let mut goruldu = [false; KONTROL_BUTON_SINIRI];
        let sol = Self::tarafi_coz(sol_metin, &mut goruldu);
        let sag = Self::tarafi_coz(sag_metin, &mut goruldu);

        if sol.iter().all(Option::is_none) && sag.iter().all(Option::is_none) {
            return None;
        }

        Some(Self { sol, sag })
    }

    fn tarafi_coz(
        metin: &str,
        goruldu: &mut [bool; KONTROL_BUTON_SINIRI],
    ) -> [Option<KontrolTipi>; KONTROL_BUTON_SINIRI] {
        let mut sonuc = [None; KONTROL_BUTON_SINIRI];
        let mut sira = 0;

        for ad in metin.split(',') {
            let ad = ad.trim();
            let Some(tip) = KontrolTipi::gnome_adindan_coz(ad) else {
                continue;
            };

            let indeks = tip.siralama_indeksi();
            if goruldu[indeks] {
                continue;
            }

            if let Some(yuva) = sonuc.get_mut(sira) {
                *yuva = Some(tip);
                goruldu[indeks] = true;
                sira += 1;
            }
        }

        sonuc
    }
}

#[allow(dead_code)]
impl KontrolTipi {
    fn simge(&self, pencere_buyuk_mu: bool) -> &'static str {
        match self {
            Self::Kucult => "\u{2013}",
            Self::Buyut if pencere_buyuk_mu => "\u{2750}",
            Self::Buyut => "\u{25A1}",
            Self::Kapat => "\u{2715}",
        }
    }

    fn grup_adi(&self) -> &'static str {
        match self {
            Self::Kucult => "kontrol-kucult",
            Self::Buyut => "kontrol-buyut",
            Self::Kapat => "kontrol-kapat",
        }
    }

    #[cfg(target_os = "linux")]
    fn gnome_adindan_coz(ad: &str) -> Option<Self> {
        match ad {
            "minimize" => Some(Self::Kucult),
            "maximize" => Some(Self::Buyut),
            "close" => Some(Self::Kapat),
            _ => None,
        }
    }

    #[cfg(target_os = "linux")]
    fn siralama_indeksi(&self) -> usize {
        match self {
            Self::Kucult => 0,
            Self::Buyut => 1,
            Self::Kapat => 2,
        }
    }

    fn window_control(&self) -> WindowControlArea {
        match self {
            Self::Kucult => WindowControlArea::Min,
            Self::Buyut => WindowControlArea::Max,
            Self::Kapat => WindowControlArea::Close,
        }
    }
}

#[cfg(target_os = "linux")]
struct KontrolDuzeniOnbellek {
    son_yenileme: Instant,
    duzen: KontrolDuzeni,
}

#[cfg(target_os = "linux")]
fn linux_sistem_kontrol_duzeni_oku() -> Option<KontrolDuzeni> {
    let cikti = Command::new("gsettings")
        .args([
            "get",
            "org.gnome.desktop.wm.preferences",
            "button-layout",
        ])
        .output()
        .ok()?;

    if !cikti.status.success() {
        return None;
    }

    let ham = String::from_utf8_lossy(&cikti.stdout);
    let metin = ham.trim().trim_matches('\'').trim_matches('"');

    KontrolDuzeni::metinden_coz(metin)
}

#[cfg(target_os = "linux")]
fn linux_kontrol_duzeni() -> KontrolDuzeni {
    static ONBELLEK: OnceLock<Mutex<KontrolDuzeniOnbellek>> = OnceLock::new();

    let onbellek = ONBELLEK.get_or_init(|| {
        let ilk_duzen =
            linux_sistem_kontrol_duzeni_oku().unwrap_or_else(KontrolDuzeni::standart);
        Mutex::new(KontrolDuzeniOnbellek {
            son_yenileme: Instant::now(),
            duzen: ilk_duzen,
        })
    });

    let mut kilit = match onbellek.lock() {
        Ok(kilit) => kilit,
        Err(kilit) => kilit.into_inner(),
    };

    if kilit.son_yenileme.elapsed() >= Duration::from_secs(2) {
        kilit.son_yenileme = Instant::now();
        if let Some(yeni_duzen) = linux_sistem_kontrol_duzeni_oku() {
            kilit.duzen = yeni_duzen;
        }
    }

    kilit.duzen
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

    #[cfg(target_os = "linux")]
    let base = base
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            window.prevent_default();
            cx.stop_propagation();
        })
        .on_click(move |_, window, cx| match tip {
            KontrolTipi::Kucult => window.minimize_window(),
            KontrolTipi::Buyut => window.zoom_window(),
            KontrolTipi::Kapat => {
                if kapatma_istegi(window, cx) {
                    cx.quit();
                }
            }
        });

    base
}

#[cfg_attr(target_os = "macos", allow(unused_mut, unreachable_code))]
fn pencere_kontrolleri_taraf(
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

        for tip in secilen.into_iter().flatten() {
            if kontrol_destekleniyor_mu(tip, desteklenen) {
                satir = satir.child(kontrol_butonu(tip, tema, pencere_buyuk_mu));
            }
        }
    }

    satir
}

// ── Kapatma kontrolu ──────────────────────────────────────

pub fn kapatma_istegi(_window: &mut Window, _cx: &mut gpui::App) -> bool {
    true
}

// ── Ust bar ───────────────────────────────────────────────

pub struct UstBar;

impl UstBar {
    pub fn render(&self, window: &Window, tema: &Tema) -> impl IntoElement {
        let sol_kontroller =
            pencere_kontrolleri_taraf(KontrolTarafi::Sol, window, tema);
        let sag_kontroller =
            pencere_kontrolleri_taraf(KontrolTarafi::Sag, window, tema);

        let mut kok = div()
            .id("ust-bar")
            .w_full()
            .h(tema.ust_bar_yukseklik)
            .flex_shrink_0()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .pl(tema.ust_bar_sol_bosluk)
            .window_control_area(WindowControlArea::Drag);

        if !tema.ust_sinir {
            kok = kok.border_b_1().border_color(tema.ust_bar_ayirici);
        }

        kok.on_mouse_down(MouseButton::Left, |ev, window, _cx| {
            if ev.click_count == 2 {
                #[cfg(target_os = "macos")]
                window.titlebar_double_click();
                #[cfg(not(target_os = "macos"))]
                window.zoom_window();
            } else {
                #[cfg(target_os = "linux")]
                window.start_window_move();
            }
        })
        .on_mouse_down(MouseButton::Right, |ev, window, _cx| {
            let _ = (&ev, &window);
            #[cfg(target_os = "linux")]
            {
                if matches!(window.window_decorations(), Decorations::Client { .. })
                    && window.window_controls().window_menu
                {
                    window.show_window_menu(ev.position);
                }
            }
        })
        .child(sol_kontroller)
        .child(
            div()
                .id("ust-bar-icerik")
                .flex()
                .flex_row()
                .items_center()
                .h_full()
                .flex_1()
                .child(
                    div()
                        .text_color(tema.ust_bar_metin)
                        .text_size(px(14.))
                        .child("Merhaba Dünya!"),
                ),
        )
        .child(sag_kontroller)
    }
}

// ── Calisma yuzeyi ────────────────────────────────────────

pub struct CalismaYuzeyi;

impl CalismaYuzeyi {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, tema: &Tema) -> impl IntoElement {
        let mut base = div()
            .id("calisma-yuzeyi")
            .flex_1()
            .flex()
            .flex_col()
            .bg(tema.yuzey_1)
            .overflow_hidden()
            .border_l_1()
            .border_color(tema.kenarlik)
            .rounded_br(tema.pencere_kavis);

        if tema.ust_sinir {
            base = base.rounded_tr(tema.pencere_kavis);
        }

        if tema.calisma_yuzeyi_kavisli_mi && tema.ust_sinir {
            base = base.rounded_tl(tema.calisma_yuzeyi_kavis);
        }

        base.child(div().id("icerik").flex_1())
    }
}

// ── Linux: ikon + .desktop kurulumu ───────────────────────

/// XDG Icon Theme spec'ine gore yuklenecek hicolor boyutlari ve gomulu PNG
/// verileri. DE'ler bagclama gore en uygun olani secer (16: sistem tepsisi,
/// 32–48: pencere basligi / gorev degistirici, 128+: dock / baslatici).
#[cfg(target_os = "linux")]
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

/// Linux'ta dock/taskbar ikonu icin .desktop ve PNG'yi kullanici veri dizinine
/// yazar. XDG standardi olan bu yaklasim GNOME, KDE Plasma, XFCE, MATE,
/// Cinnamon, LXQt, Sway, i3, Hyprland vb. pencere yoneticilerinde hem X11 hem
/// Wayland oturumlarinda calisir. GPUI app_id degeri X11 WM_CLASS'a ve
/// Wayland xdg-shell app_id'ye yazildigindan .desktop StartupWMClass alaniyla
/// eslesir.
#[cfg(target_os = "linux")]
fn linux_ikon_kur() {
    let Some(veri_dizini) = dirs::data_dir() else {
        return;
    };

    // 1) PNG'leri hicolor ikon temasi altina coklu boyut olarak yaz.
    //    Hicolor, XDG Icon Theme spec'ine gore tum DE'lerin bakmak zorunda
    //    oldugu varsayilan temadir.
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
        if guncellenmeli {
            if std::fs::write(&yol, veri).is_ok() {
                herhangi_guncellendi = true;
            }
        }
        son_ikon_yolu = Some(yol);
    }

    // 2) .desktop dosyasini yaz. Icon= icin tema adi yerine mutlak yol
    //    kullanmak herhangi bir tema aramasina ihtiyac birakmaz; ikon
    //    bulunamama ihtimalini sifira indirir. En buyuk boyuttaki dosyaya
    //    isaret ederiz (eski/ekstra DE'ler icin).
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

    // 3) GTK/KDE ikon onbelleklerini yenile (best-effort; cogu modern DE
    //    zaten otomatik algiliyor, ama eski kurulumlarda yardimci olur).
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
