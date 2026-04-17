# 1. Hedef workspace omurgası

Mevcut yapı korunacak, ama bazı crate’lerin rolü netleşecek.

```text id="gxg5ev"
hbr_sstm_ui/
├─ Cargo.toml
├─ uygulama/
│  ├─ Cargo.toml
│  └─ src/
│     ├─ main.rs
│     ├─ uygulama_durumu.rs
│     ├─ acilis.rs
│     └─ moduller.rs
│
├─ kutuphaneler/
│  ├─ uygulama_kabugu/
│  ├─ ortak_tema/
│  ├─ ortak_ikonlar/
│  ├─ ortak_bilesenler/
│  ├─ ortak_tipler/
│  ├─ modul_sistemi/
│  ├─ calisma_sekmeleri/
│  ├─ sol_menu/
│  ├─ sunucu_istemcisi/
│  └─ tema/
│
└─ moduller/
   ├─ arsiv/
   ├─ kullanici_yonetimi/
   ├─ kurum/
   ├─ yetki/
   ├─ nobet/              # sonra
   ├─ personel/           # sonra
   └─ ebys/               # sonra
```

---

# 2. Crate rolleri

## `uygulama`

Sadece bootstrap değil; host kompozisyon noktası olacak.

Görevleri:

* uygulama açılışı
* global durum kurulumu
* giriş akışı
* modül kayıtları
* ana pencere açılışı
* host servislerinin enjekte edilmesi

## `uygulama_kabugu`

Sadece görsel kabuk olacak.

Görevleri:

* pencere
* üst bar
* gölge/CSD
* ana panel
* shell yerleşimi

Buraya iş mantığı girmemeli.

## `ortak_tipler`

Tüm statik veri modelleri ve DTO’lar burada olacak.

Görevleri:

* kullanıcı
* oturum
* yetki
* kurum
* modül kimlikleri
* sürüm / uyumluluk yapıları
* ortak sonuç / hata türleri

## `modul_sistemi`

Asıl çekirdek burada.

Görevleri:

* modül trait’leri
* modül manifesti
* modül kayıt defteri
* host API sözleşmesi
* modül bağlamı
* görünürlük / aktivasyon kararı
* ileri aşamada katalog/yükleyiciye evrilecek temel katman

## `sunucu_istemcisi`

Backend istemcisi.

Görevleri:

* giriş
* oturum doğrulama
* yetki çekme
* modül kataloğu çekme
* sürüm uyumluluğu çekme
* veri çağrıları

## `calisma_sekmeleri`

Merkez çalışma alanı.

Görevleri:

* açık sekmeler
* sekme anahtarları
* aç / kapat / odakla
* modül ekranlarını taşıma

## `sol_menu`

Modül listesi gösterimi.

Görevleri:

* kayıtlı modülleri listelemek
* yetkili ve görünür modülleri filtrelemek
* tıklamada sekme açılışını tetiklemek

## `moduller/*`

Her modül kendi iş alanı olacak.

Görevleri:

* modül manifesti
* ekranlar
* komutlar
* domain servisleri
* modüle özgü DTO’lar
* modüle özgü UI

---

# 3. İlk teknik ayrım: statik modül vs dinamik modül

Bu ayrımı en baştan kodda görünür yapmanı öneririm.

## Şimdilik kullanılacak model

Statik modül:

```rust id="sgrzxh"
pub trait UygulamaModulu: Send + Sync {
    fn manifest(&self) -> ModulManifesti;
    fn yetki_tanimi(&self) -> &'static [YetkiKodu];
    fn ana_ekran(&self, baglam: ModulBaglami) -> ModulEkrani;
}
```

Bu model Faz 1–4 için yeterli.

## Sonraki aşamada açılacak model

Dinamik modül için ayrıca ikinci bir katman eklenecek:

```rust id="4rut9f"
pub trait DinamikModulKoprusu {
    fn modulu_yukle(&self, yol: &Path) -> Result<YuklenmisModul, ModulYuklemeHatasi>;
}
```

Ama bunu şimdi implemente etmiyoruz.
Sadece yapıyı buna evrilebilir bırakıyoruz.

---

# 4. `ortak_tipler` teknik iskeleti

Önerilen dosya ağacı:

```text id="vjlwm2"
kutuphaneler/ortak_tipler/src/
├─ lib.rs
├─ kimlikler.rs
├─ oturum.rs
├─ kullanici.rs
├─ yetki.rs
├─ kurum.rs
├─ modul.rs
├─ surum.rs
├─ sonuc.rs
└─ hata.rs
```

## `lib.rs`

```rust id="lt9zxp"
pub mod hata;
pub mod kimlikler;
pub mod kurum;
pub mod kullanici;
pub mod modul;
pub mod oturum;
pub mod sonuc;
pub mod surum;
pub mod yetki;

pub use hata::*;
pub use kimlikler::*;
pub use kurum::*;
pub use kullanici::*;
pub use modul::*;
pub use oturum::*;
pub use sonuc::*;
pub use surum::*;
pub use yetki::*;
```

## `kimlikler.rs`

```rust id="5csmrb"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KullaniciId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KurumId(pub i64);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OturumId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModulKimligi(pub &'static str);
```

## `oturum.rs`

```rust id="9n8tcy"
use crate::{KullaniciId, KurumId, OturumId};

#[derive(Debug, Clone)]
pub struct KullaniciOturumu {
    pub oturum_id: OturumId,
    pub kullanici_id: KullaniciId,
    pub kurum_id: KurumId,
    pub kullanici_adi: String,
    pub gorunen_ad: String,
}
```

## `yetki.rs`

```rust id="mwtypk"
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct YetkiKodu(pub String);

#[derive(Debug, Clone, Default)]
pub struct YetkiOzeti {
    pub moduller: Vec<String>,
    pub islemler: Vec<YetkiKodu>,
}

impl YetkiOzeti {
    pub fn modul_gorebilir_mi(&self, modul: &str) -> bool {
        self.moduller.iter().any(|m| m == modul)
    }

    pub fn yetki_var_mi(&self, kod: &str) -> bool {
        self.islemler.iter().any(|y| y.0 == kod)
    }
}
```

## `modul.rs`

```rust id="33oivv"
use crate::ModulKimligi;

#[derive(Debug, Clone)]
pub struct ModulManifesti {
    pub kimlik: ModulKimligi,
    pub ad: &'static str,
    pub aciklama: &'static str,
    pub ikon_yolu: &'static str,
    pub varsayilan_sira: u16,
    pub kategori: ModulKategorisi,
}

#[derive(Debug, Clone, Copy)]
pub enum ModulKategorisi {
    Yonetim,
    Operasyon,
    Arsiv,
    Kurumsal,
}
```

## `surum.rs`

```rust id="h9xw11"
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SemaSurumu(pub i32);

#[derive(Debug, Clone)]
pub struct UygulamaSurumu {
    pub ana: u16,
    pub min: u16,
    pub yama: u16,
}

#[derive(Debug, Clone)]
pub struct ModulUyumlulukBilgisi {
    pub min_host_surumu: UygulamaSurumu,
    pub min_sema_surumu: SemaSurumu,
    pub max_sema_surumu: Option<SemaSurumu>,
}
```

## `sonuc.rs`

```rust id="tadhw5"
pub type UygulamaSonucu<T> = Result<T, crate::UygulamaHatasi>;
```

## `hata.rs`

```rust id="zvm5g7"
#[derive(Debug)]
pub enum UygulamaHatasi {
    GirisBasarisiz,
    YetkiYok,
    ModulBulunamadi,
    UyumsuzSurum,
    SunucuBaglantiHatasi(String),
    Beklenmeyen(String),
}
```

---

# 5. `modul_sistemi` teknik iskeleti

Önerilen dosya ağacı:

```text id="kpj8jn"
kutuphaneler/modul_sistemi/src/
├─ lib.rs
├─ baglam.rs
├─ host_api.rs
├─ kayit.rs
├─ manifest.rs
├─ modul.rs
├─ modul_ekrani.rs
├─ kayit_defteri.rs
├─ gorunurluk.rs
└─ acilis.rs
```

## `lib.rs`

```rust id="z57v4q"
pub mod acilis;
pub mod baglam;
pub mod gorunurluk;
pub mod host_api;
pub mod kayit;
pub mod kayit_defteri;
pub mod manifest;
pub mod modul;
pub mod modul_ekrani;

pub use acilis::*;
pub use baglam::*;
pub use gorunurluk::*;
pub use host_api::*;
pub use kayit::*;
pub use kayit_defteri::*;
pub use manifest::*;
pub use modul::*;
pub use modul_ekrani::*;
```

## `manifest.rs`

```rust id="cjnd2l"
pub use ortak_tipler::{ModulKategorisi, ModulManifesti};
```

## `baglam.rs`

```rust id="v8l8v7"
use std::sync::Arc;

use ortak_tipler::{KullaniciOturumu, YetkiOzeti};

use crate::HostApi;

#[derive(Clone)]
pub struct ModulBaglami {
    pub oturum: Arc<KullaniciOturumu>,
    pub yetkiler: Arc<YetkiOzeti>,
    pub host_api: Arc<dyn HostApi>,
}
```

## `host_api.rs`

```rust id="lu80qq"
use ortak_tipler::UygulamaSonucu;

pub trait HostApi: Send + Sync {
    fn ekran_ac(&self, modul: &str, ekran: &str) -> UygulamaSonucu<()>;
    fn bildirim_goster(&self, baslik: &str, mesaj: &str);
    fn veri_cagir(&self, rota: &str, govde_json: &str) -> UygulamaSonucu<String>;
}
```

Bu kritik.
İlk aşamada host ile modül arasındaki servis sözleşmesi burada oluşacak.

## `modul_ekrani.rs`

İlk aşamada bunu basit tut.

```rust id="jeqid1"
use gpui::AnyView;

pub struct ModulEkrani {
    pub anahtar: String,
    pub baslik: String,
    pub gorunum: AnyView,
}
```

Bu fazda tek binary içinde olduğumuz için `AnyView` kullanmak sorun değil.

## `modul.rs`

```rust id="nzq7ac"
use ortak_tipler::YetkiKodu;

use crate::{ModulBaglami, ModulEkrani, ModulManifesti};

pub trait UygulamaModulu: Send + Sync {
    fn manifest(&self) -> ModulManifesti;
    fn gereken_yetkiler(&self) -> &'static [YetkiKodu];
    fn ana_ekran(&self, baglam: ModulBaglami) -> ModulEkrani;
}
```

## `kayit.rs`

```rust id="1ohbhw"
use std::sync::Arc;

use crate::UygulamaModulu;

pub struct ModulKaydi {
    pub modul: Arc<dyn UygulamaModulu>,
}
```

## `kayit_defteri.rs`

```rust id="04slwl"
use std::collections::BTreeMap;
use std::sync::Arc;

use ortak_tipler::ModulKimligi;

use crate::{ModulKaydi, UygulamaModulu};

pub struct ModulKayitDefteri {
    kayitlar: BTreeMap<&'static str, Arc<dyn UygulamaModulu>>,
}

impl ModulKayitDefteri {
    pub fn new() -> Self {
        Self {
            kayitlar: BTreeMap::new(),
        }
    }

    pub fn kaydet(&mut self, modul: Arc<dyn UygulamaModulu>) {
        let kimlik = modul.manifest().kimlik.0;
        self.kayitlar.insert(kimlik, modul);
    }

    pub fn getir(&self, kimlik: &str) -> Option<Arc<dyn UygulamaModulu>> {
        self.kayitlar.get(kimlik).cloned()
    }

    pub fn hepsi(&self) -> Vec<Arc<dyn UygulamaModulu>> {
        self.kayitlar.values().cloned().collect()
    }
}
```

## `gorunurluk.rs`

```rust id="5ft5h2"
use ortak_tipler::YetkiOzeti;

pub fn modul_gorunur_mu(modul_kimligi: &str, yetkiler: &YetkiOzeti) -> bool {
    yetkiler.modul_gorebilir_mi(modul_kimligi)
}
```

## `acilis.rs`

```rust id="1i8y9u"
use std::sync::Arc;

use ortak_tipler::UygulamaSonucu;

use crate::{ModulBaglami, ModulEkrani, ModulKayitDefteri};

pub fn modul_ac(
    kayit_defteri: &ModulKayitDefteri,
    modul_kimligi: &str,
    baglam: ModulBaglami,
) -> UygulamaSonucu<ModulEkrani> {
    let modul = kayit_defteri
        .getir(modul_kimligi)
        .ok_or(ortak_tipler::UygulamaHatasi::ModulBulunamadi)?;

    Ok(modul.ana_ekran(baglam))
}
```

---

# 6. `sunucu_istemcisi` teknik iskeleti

Önerilen dosya ağacı:

```text id="8wwle1"
kutuphaneler/sunucu_istemcisi/src/
├─ lib.rs
├─ istemci.rs
├─ giris.rs
├─ oturum.rs
├─ modul_katalogu.rs
└─ hata.rs
```

## `lib.rs`

```rust id="0cxr7d"
pub mod hata;
pub mod giris;
pub mod istemci;
pub mod modul_katalogu;
pub mod oturum;

pub use hata::*;
pub use giris::*;
pub use istemci::*;
pub use modul_katalogu::*;
pub use oturum::*;
```

## `istemci.rs`

```rust id="53u0zy"
#[derive(Clone)]
pub struct SunucuIstemcisi {
    pub temel_url: String,
}

impl SunucuIstemcisi {
    pub fn new(temel_url: impl Into<String>) -> Self {
        Self {
            temel_url: temel_url.into(),
        }
    }
}
```

## `giris.rs`

```rust id="jzbj1o"
use ortak_tipler::{KullaniciOturumu, UygulamaSonucu};

use crate::SunucuIstemcisi;

pub struct GirisIstegi {
    pub kullanici_adi: String,
    pub parola: String,
}

impl SunucuIstemcisi {
    pub async fn giris_yap(&self, _istek: GirisIstegi) -> UygulamaSonucu<KullaniciOturumu> {
        todo!("backend entegrasyonu")
    }
}
```

## `oturum.rs`

```rust id="spzsah"
use ortak_tipler::{KullaniciOturumu, YetkiOzeti, UygulamaSonucu};

use crate::SunucuIstemcisi;

impl SunucuIstemcisi {
    pub async fn yetki_ozeti_getir(
        &self,
        _oturum: &KullaniciOturumu,
    ) -> UygulamaSonucu<YetkiOzeti> {
        todo!("backend entegrasyonu")
    }

    pub async fn veri_cagir(
        &self,
        _oturum: &KullaniciOturumu,
        _rota: &str,
        _govde_json: &str,
    ) -> UygulamaSonucu<String> {
        todo!("backend entegrasyonu")
    }
}
```

## `modul_katalogu.rs`

```rust id="3pvti9"
use ortak_tipler::{ModulUyumlulukBilgisi, UygulamaSonucu};

use crate::SunucuIstemcisi;

#[derive(Debug, Clone)]
pub struct ModulKatalogKaydi {
    pub modul_kimligi: String,
    pub surum: String,
    pub uyumluluk: ModulUyumlulukBilgisi,
}

impl SunucuIstemcisi {
    pub async fn modul_katalogu_getir(&self) -> UygulamaSonucu<Vec<ModulKatalogKaydi>> {
        todo!("backend entegrasyonu")
    }
}
```

---

# 7. `calisma_sekmeleri` teknik iskeleti

Önerilen dosya ağacı:

```text id="i3q4ql"
kutuphaneler/calisma_sekmeleri/src/
├─ lib.rs
├─ sekme.rs
├─ durum.rs
└─ yonetici.rs
```

## `sekme.rs`

```rust id="5w7v4w"
use gpui::AnyView;

#[derive(Clone)]
pub struct CalismaSekmesi {
    pub anahtar: String,
    pub baslik: String,
    pub modul_kimligi: String,
    pub gorunum: AnyView,
}
```

## `durum.rs`

```rust id="ubgl2e"
use crate::CalismaSekmesi;

pub struct CalismaSekmesiDurumu {
    pub sekmeler: Vec<CalismaSekmesi>,
    pub aktif_index: Option<usize>,
}
```

## `yonetici.rs`

```rust id="ywax5m"
use crate::{CalismaSekmesi, CalismaSekmesiDurumu};

pub struct CalismaSekmesiYonetici {
    pub durum: CalismaSekmesiDurumu,
}

impl CalismaSekmesiYonetici {
    pub fn new() -> Self {
        Self {
            durum: CalismaSekmesiDurumu {
                sekmeler: Vec::new(),
                aktif_index: None,
            },
        }
    }

    pub fn sekme_ac(&mut self, sekme: CalismaSekmesi) {
        self.durum.sekmeler.push(sekme);
        self.durum.aktif_index = Some(self.durum.sekmeler.len() - 1);
    }
}
```

---

# 8. `sol_menu` teknik iskeleti

Şu an sadece genişlik taşıyor.
Bunu gerçek menüye çevirmelisin.

Önerilen dosya ağacı:

```text id="ch4stx"
kutuphaneler/sol_menu/src/
├─ lib.rs
├─ model.rs
└─ olay.rs
```

## `model.rs`

```rust id="fbnq7h"
#[derive(Clone)]
pub struct SolMenuOgesi {
    pub modul_kimligi: String,
    pub ad: String,
    pub ikon_yolu: String,
    pub secili: bool,
}
```

## `olay.rs`

```rust id="kz6g10"
#[derive(Clone)]
pub enum SolMenuOlayi {
    ModulSecildi(String),
}
```

## `lib.rs`

Burada render, `Vec<SolMenuOgesi>` almalı ve seçim olayını host’a bildirmeli.

İlk aşamada bunu callback ile çözebilirsin.

---

# 9. `uygulama` crate iskeleti

Önerilen dosya ağacı:

```text id="s1hpn9"
uygulama/src/
├─ main.rs
├─ uygulama_durumu.rs
├─ acilis.rs
├─ host_api.rs
└─ moduller.rs
```

## `uygulama_durumu.rs`

```rust id="ezx10b"
use std::sync::Arc;

use modul_sistemi::ModulKayitDefteri;
use ortak_tipler::{KullaniciOturumu, YetkiOzeti};
use sunucu_istemcisi::SunucuIstemcisi;

pub struct UygulamaDurumu {
    pub sunucu: Arc<SunucuIstemcisi>,
    pub oturum: Option<Arc<KullaniciOturumu>>,
    pub yetkiler: Option<Arc<YetkiOzeti>>,
    pub moduller: ModulKayitDefteri,
}
```

## `moduller.rs`

Burada statik kayıt yapılacak.

```rust id="2v4w7o"
use std::sync::Arc;

use arsiv::ArsivModulu;
use kurum::KurumModulu;
use kullanici_yonetimi::KullaniciYonetimiModulu;
use modul_sistemi::ModulKayitDefteri;
use yetki::YetkiModulu;

pub fn varsayilan_modulleri_kaydet() -> ModulKayitDefteri {
    let mut defter = ModulKayitDefteri::new();
    defter.kaydet(Arc::new(ArsivModulu::new()));
    defter.kaydet(Arc::new(KullaniciYonetimiModulu::new()));
    defter.kaydet(Arc::new(KurumModulu::new()));
    defter.kaydet(Arc::new(YetkiModulu::new()));
    defter
}
```

## `host_api.rs`

`modul_sistemi::HostApi` implementasyonu burada olur.

```rust id="j6vqr7"
use std::sync::Arc;

use modul_sistemi::HostApi;
use ortak_tipler::UygulamaSonucu;

use crate::uygulama_durumu::UygulamaDurumu;

pub struct HostApiUygulamasi {
    pub durum: Arc<UygulamaDurumu>,
}

impl HostApi for HostApiUygulamasi {
    fn ekran_ac(&self, _modul: &str, _ekran: &str) -> UygulamaSonucu<()> {
        todo!()
    }

    fn bildirim_goster(&self, _baslik: &str, _mesaj: &str) {}

    fn veri_cagir(&self, _rota: &str, _govde_json: &str) -> UygulamaSonucu<String> {
        todo!()
    }
}
```

## `acilis.rs`

Bu dosya giriş akışını ve açılış durumlarını yönetecek.

---

# 10. `moduller/arsiv` ilk örnek modül

İlk çalışan örnek modül olarak bunu seçmek doğru.

Önerilen dosya ağacı:

```text id="qql6zy"
moduller/arsiv/src/
├─ lib.rs
├─ modul.rs
├─ ekranlar/
│  ├─ mod.rs
│  └─ ana_sayfa.rs
├─ komutlar.rs
└─ tanimlar/
   └─ mod.rs
```

## `lib.rs`

```rust id="7697bg"
mod ekranlar;
mod komutlar;
mod modul;
pub mod tanimlar;

pub use modul::ArsivModulu;
```

## `modul.rs`

```rust id="e5v4vy"
use gpui::{div, px, AnyView, IntoElement};
use modul_sistemi::{ModulBaglami, ModulEkrani, UygulamaModulu};
use ortak_tipler::{ModulKategorisi, ModulKimligi, ModulManifesti, YetkiKodu};

pub struct ArsivModulu;

impl ArsivModulu {
    pub fn new() -> Self {
        Self
    }
}

impl UygulamaModulu for ArsivModulu {
    fn manifest(&self) -> ModulManifesti {
        ModulManifesti {
            kimlik: ModulKimligi("arsiv"),
            ad: "Arşiv",
            aciklama: "Belge ve arşiv işlemleri",
            ikon_yolu: "modul_ikonlar/arsiv.png",
            varsayilan_sira: 10,
            kategori: ModulKategorisi::Arsiv,
        }
    }

    fn gereken_yetkiler(&self) -> &'static [YetkiKodu] {
        &[]
    }

    fn ana_ekran(&self, _baglam: ModulBaglami) -> ModulEkrani {
        let gorunum: AnyView = div()
            .size_full()
            .child("Arşiv modülü ana sayfası")
            .into_any_element()
            .into();

        ModulEkrani {
            anahtar: "arsiv:ana".into(),
            baslik: "Arşiv".into(),
            gorunum,
        }
    }
}
```

İlk aşamada böyle basit başla.
Sonra bunu entity tabanlı GPUI ekranlarına dönüştürürsün.

---

# 11. `uygulama_kabugu` ile bağlama noktası

Şu an `ana_panel.rs` sadece boş çalışma alanı veriyor.
Bunu üç parçaya ayırman gerekecek:

* sol menü görünümü
* merkez sekme görünümü
* sağ/alt alanlar için genişleme yüzeyi

Şimdilik ilk hedef:
`AnaPanel`, `UygulamaDurumu` veya en azından `ModulKayitDefteri + YetkiOzeti` okuyabilmeli.

Bunun için `AnaPanel::yeni` imzasını ileride genişletmeye uygun bırak:

```rust id="sevjlwm"
pub struct AnaPanelBaglami {
    pub moduller: ModulKayitDefteri,
    pub aktif_oturum_var_mi: bool,
}
```

İlk aşamada bunu sade tutabilirsin.
Ama `AnaPanel`in gelecekte menü + sekme + modül aktivasyonu yöneteceği kesin.

---

# 12. Giriş akışı için açılış durum makinesi

Şu ayrımı erkenden kurmanı öneririm:

```rust id="7p6mvx"
pub enum AcilisDurumu {
    Yukleniyor,
    GirisBekleniyor,
    OturumAcildi,
    UyumsuzlukVar(String),
    Hata(String),
}
```

Bu çok işine yarar.
Çünkü biraz sonra aynı akışa şunlar eklenecek:

* modül kataloğu bekleniyor
* sürüm uyumluluğu denetleniyor
* modül paketi indiriliyor
* yönetici güncellemesi gerekiyor

---

# 13. Faz 4+ için bugünden bırakılacak ama şimdi tam uygulanmayacak yapılar

Şimdi sadece tiplerini koy, içini sonra doldur.

## `ortak_tipler/modul.rs` içine ileriden eklenecekler

```rust id="j4vvwy"
pub struct YerelKuruluModul {
    pub kimlik: String,
    pub surum: String,
    pub kurulum_yolu: String,
}

pub struct KurumLisansliModul {
    pub kimlik: String,
    pub aktif: bool,
}

pub struct KullaniciModulYetkisi {
    pub kimlik: String,
    pub gorunur: bool,
    pub kullanabilir: bool,
}
```

Bu üçlü ayrım çok önemli:

* lisans
* kurulum
* kullanıcı görünürlüğü

---

# 14. Şimdi hemen uygulanacak minimum bağımlılıklar

Bu aşamada eklemen gereken crate bağımlılıkları, ileride de boşa gitmez:

## `sunucu_istemcisi`

* `serde`
* `serde_json`
* HTTP istemcisi olarak sonra `reqwest` veya senin backend tercihin

## `modul_sistemi`

* `gpui`
* `ortak_tipler`

## `uygulama`

* mevcutlere ek olarak:

  * `modul_sistemi`
  * `sunucu_istemcisi`
  * `ortak_tipler`
  * ilgili modüller

---

# 15. İlk implementasyon sırası

Ben olsam tam şu sırayla kodlarım:

## Adım 1

`ortak_tipler`

* `kimlikler.rs`
* `oturum.rs`
* `yetki.rs`
* `modul.rs`
* `hata.rs`

## Adım 2

`modul_sistemi`

* `modul.rs`
* `manifest.rs`
* `baglam.rs`
* `host_api.rs`
* `kayit_defteri.rs`
* `modul_ekrani.rs`

## Adım 3

`moduller/arsiv`

* `ArsivModulu`
* boş ama çalışan ana ekran

## Adım 4

`uygulama/moduller.rs`

* statik modül kaydı

## Adım 5

`sol_menu`

* registry’den okuyan gerçek menü

## Adım 6

`calisma_sekmeleri`

* aç/kapat/aktif sekme

## Adım 7

`sunucu_istemcisi`

* giriş
* yetki özeti
* veri çağrısı iskeleti

## Adım 8

`uygulama_durumu`

* oturum + yetki + modüller

Bu sırayla gidersen 1–2 büyük refactor’dan kaçınırsın.

---

# 16. En kritik mimari kurallar

Bu kuralları şimdiden sabitle:

### Kural 1

`uygulama_kabugu` iş mantığı bilmeyecek.

### Kural 2

`moduller/*` doğrudan veritabanına bağlanmayacak.

### Kural 3

Modül veri isteği her zaman host/back-end oturumu üzerinden geçecek.

### Kural 4

Yetki kontrolü iki katmanlı olacak:

* menü görünürlüğü
* işlem yetkisi

### Kural 5

“kurulu modül” ile “yetkili modül” ayrı kavram olacak.

### Kural 6

Bugün statik Rust trait, yarın dinamik yükleme.
Yani bugünkü sözleşme yarınki ABI sınırına evrilebilir olmalı.

---

# 17. Sana önerdiğim ilk somut milestone

İlk çalışan milestone şu olsun:

* uygulama açılıyor
* 4 modül statik registry’ye kayıt oluyor
* sol menüde sadece yetkili modül görünüyor
* tıklanan modül merkez alanda sekme olarak açılıyor
* modül host API çağırabiliyor
* giriş sonrası kullanıcı bağlamı bellekte duruyor
