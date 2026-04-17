//! Uygulama kabuğu: pencere ömrü, üst bar, CSD dekorasyon ve kontrol
//! butonları. Yapı Zed'in `workspace` + `platform_title_bar` ayrımından
//! esinlenildi.

mod ana_panel;
mod csd_golge;
mod kontroller;
#[cfg(target_os = "linux")]
mod kontroller_linux;
#[cfg(target_os = "linux")]
mod linux_ikon;
mod pencere;
mod ust_bar;

pub use ana_panel::{AnaPanel, CalismaYuzeyi};
pub use pencere::{ana_pencere_ac, kapatma_istegi, pencere_secenekleri, UYGULAMA_APP_ID};
pub use ust_bar::UstBar;
