use gpui::prelude::*;
use gpui::*;
use ortak_tema::{AktifTema, Tema};
use sol_menu::SolMenu;

use crate::csd_golge::{golge_kenar_bul, GOLGE_BOYUTU};
use crate::ust_bar::UstBar;

/// Uygulama kök view'ı: üst bar, sol menü ve çalışma yüzeyi konteyneri.
///
/// Zed'in `MultiWorkspace` yaklaşımıyla hizalı: kök bir Entity, Focusable,
/// tema global'i observe ediliyor, alt view'lar Entity olarak tutuluyor.
pub struct AnaPanel {
    pub ust_bar: Entity<UstBar>,
    pub sol_menu: SolMenu,
    pub calisma_yuzeyi: CalismaYuzeyi,
    son_gorunum: Option<WindowBackgroundAppearance>,
    focus_handle: FocusHandle,
}

impl AnaPanel {
    pub fn yeni(cx: &mut Context<Self>) -> Self {
        let ust_bar = cx.new(UstBar::yeni);
        let focus_handle = cx.focus_handle();
        // Tema değişince kök re-render olsun. Alt Entity'ler (UstBar)
        // kendi observe'lerini ayrıca yürütür; GPUI'de parent render'ı
        // child Render'ı otomatik tetiklemez.
        cx.observe_global::<Tema>(|_this, cx| cx.notify()).detach();
        // TODO: SolMenu ve CalismaYuzeyi plain struct — ileride state
        // (seçili menü, açık sekme) eklendiğinde bunlar da Entity'ye çevrilip
        // kendi observe'leri tanımlanmalı.
        Self {
            ust_bar,
            sol_menu: SolMenu::new(),
            calisma_yuzeyi: CalismaYuzeyi::new(),
            son_gorunum: None,
            focus_handle,
        }
    }
}

impl Focusable for AnaPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AnaPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tema = cx.tema();
        self.sol_menu.baslangic_genisligi_ayarla(tema);

        // Wayland'da her render'da set_background_appearance çağrısı
        // update_window → re-render → set_background_appearance sonsuz
        // döngüsü yaratıyor; sadece değiştiğinde uyar.
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
                    .child(self.ust_bar.clone()),
            )
        } else {
            base.child(self.ust_bar.clone()).child(icerik_satiri)
        };

        div()
            .id("window-backdrop")
            .track_focus(&self.focus_handle)
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
                                let Some(kenar) = golge_kenar_bul(fare, golge, boyut) else {
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
                        if let Some(kenar) = golge_kenar_bul(e.position, golge, boyut) {
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
