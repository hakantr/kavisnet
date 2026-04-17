use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::kontroller::KontrolTipi;

pub const KONTROL_BUTON_SINIRI: usize = 3;

#[derive(Clone, Copy)]
pub struct KontrolDuzeni {
    pub sol: [Option<KontrolTipi>; KONTROL_BUTON_SINIRI],
    pub sag: [Option<KontrolTipi>; KONTROL_BUTON_SINIRI],
}

impl KontrolDuzeni {
    pub fn standart() -> Self {
        Self {
            sol: [None; KONTROL_BUTON_SINIRI],
            sag: [
                Some(KontrolTipi::Kucult),
                Some(KontrolTipi::Buyut),
                Some(KontrolTipi::Kapat),
            ],
        }
    }

    pub fn metinden_coz(metin: &str) -> Option<Self> {
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

struct KontrolDuzeniOnbellek {
    son_yenileme: Instant,
    duzen: KontrolDuzeni,
}

fn linux_sistem_kontrol_duzeni_oku() -> Option<KontrolDuzeni> {
    let cikti = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.wm.preferences", "button-layout"])
        .output()
        .ok()?;

    if !cikti.status.success() {
        return None;
    }

    let ham = String::from_utf8_lossy(&cikti.stdout);
    let metin = ham.trim().trim_matches('\'').trim_matches('"');

    KontrolDuzeni::metinden_coz(metin)
}

/// GNOME button-layout düzenini 2 saniyede bir tazeleyen süreç-genel önbellek.
// TODO: KDE/XFCE/Hyprland gibi GNOME dışı masaüstlerinde doğru ayar anahtarını
// okumak için platform algılaması eklenecek.
pub fn linux_kontrol_duzeni() -> KontrolDuzeni {
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
