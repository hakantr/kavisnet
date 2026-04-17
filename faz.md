# Faz 1 — Çekirdek modül sözleşmesi ve statik modül sistemi

Bu fazın amacı:
**modül fikrini çalışır hale getirmek ama henüz DLL/SO/DYLIB işine girmemek**

## Hedef

* modül kimliği
* modül manifesti
* modül kayıt sistemi
* sol menü entegrasyonu
* ekran açma mantığı
* sekme/çalışma alanı akışı

## Bu fazda yapılacaklar

### 1. `kutuphaneler/modul_sistemi` doldurulmalı

Buraya şunlar gelmeli:

* `ModulKimligi`
* `ModulAdi`
* `ModulManifesti`
* `ModulKategorisi`
* `ModulSurumu`
* `ModulKaydi`
* `ModulKayitDefteri`
* `ModulGorunurlukKurali`
* `ModulAcmaIstegi`

Örnek düşünce:

```rust
pub struct ModulManifesti {
    pub kimlik: &'static str,
    pub ad: &'static str,
    pub surum: &'static str,
    pub ikon: &'static str,
    pub sira: u16,
}
```

### 2. Modüller trait ile tanımlanmalı

Bu aşamada tek binary olduğumuz için Rust trait kullanmakta sakınca yok.

Örnek:

```rust
pub trait UygulamaModulu {
    fn manifest(&self) -> ModulManifesti;
    fn acilis_ekrani(&self) -> ModulAcilisEkrani;
}
```

Burada henüz “Rust tiplerini geçirmeyelim” kuralı devreye girmez.
O kural **dinamik plugin** aşamasında kritik olacak.
Faz 1’de her şey aynı binary içinde olduğu için doğrudan Rust trait kullanmak doğru ve hızlıdır.

### 3. `moduller/*` crate’leri gerçek modüle çevrilmeli

Şu an sadece boş iskeletler var.

Her modülde en az şunlar olmalı:

* `manifest.rs`
* `ekranlar/`
* `komutlar/`
* `servisler/`
* `tanimlar/`

İlk örnek modül:

* `arsiv`

Sonra:

* `kullanici_yonetimi`
* `kurum`
* `yetki`

### 4. `sol_menu` modül kayıt defterinden beslenecek hale gelmeli

Şu an yalnızca genişlik taşıyor.
Bunu statik bir UI kutusu olmaktan çıkarıp registry okuyan hale getirmen lazım.

Yani:

* host açılır
* modül kayıtları toplanır
* sol menü bunları listeler
* tıklanınca merkez alana ilgili modül açılır

### 5. `calisma_sekmeleri` gerçek sekme yöneticisine dönüşmeli

Şu an boş. Burada:

* açık modül sayfaları
* tekil sekme anahtarı
* sekme başlığı
* aktif sekme
* kapatma
* tekrar açma

mantığı kurulmalı.

## Faz 1 sonucu

Bu faz sonunda sistem şuna dönüşmüş olmalı:

* uygulama açılır
* modüller kayıt olur
* sol menüde görünür
* modül seçilince merkez alanda açılır
* sekme mantığı çalışır

Henüz:

* giriş yok
* backend yok
* yetki yok
* lisans yok
* dinamik paket yok

Ama iskelet doğru kurulur.

---

# Faz 2 — Oturum, backend istemcisi ve kullanıcı bağlamı

Bu fazın amacı:
**modüllerin veriyi kendi başına değil, host oturumu üzerinden kullanması**

Bu, önceki konuşmalarımızla birebir uyumlu.

## Hedef

* giriş ekranı
* backend istemcisi
* oturum bilgisi
* kullanıcı bağlamı
* yetki özeti

## Bu fazda yapılacaklar

### 1. `kutuphaneler/sunucu_istemcisi` doldurulmalı

Şu an boş. Bu crate’e şunlar gelmeli:

* `giris_yap(kullanici_adi, parola)`
* `oturum_dogrula()`
* `modul_listesi_getir()`
* `yetki_ozeti_getir()`
* `surum_uyumlulugu_getir()`

Bu crate doğrudan DB istemcisi gibi değil, **backend API istemcisi** gibi düşünülmeli.

### 2. `ortak_tipler` gerçek DTO katmanı olmalı

Şu an boş.

Buraya koyman gereken şeyler:

* `KullaniciOturumu`
* `KullaniciOzeti`
* `YetkiOzeti`
* `KurumOzeti`
* `ModulYetkisi`
* `SemaSurumu`
* `SunucuSurumu`
* `ModulUyumlulukKaydi`

### 3. Host düzeyinde `OturumBaglami` oluşturulmalı

Ana uygulama girişten sonra bellekte bir global bağlam tutmalı.

Örnek:

```rust
pub struct OturumBaglami {
    pub oturum_id: String,
    pub kullanici_id: i64,
    pub kurum_id: i64,
    pub yetkiler: YetkiOzeti,
}
```

### 4. Modüller host API üzerinden veri istemeli

Bu fazda ana ilke:
**modül doğrudan DB’ye bağlanmaz**

Akış şöyle olmalı:

* modül `arsiv.belge_listele` ister
* host mevcut oturumla backend’e gider
* backend sonucu döndürür
* modül ekrana basar

Yani modülün eline şunlar verilir:

* kullanıcı bağlamı
* yetki özeti
* host servis çağrıları

Ama:

* parola
* ham DB bağlantısı
* connection pool

verilmez.

## Faz 2 sonucu

Bu faz sonunda:

* kullanıcı giriş yapar
* host oturumu alır
* modüller bu oturumla veri ister
* aynı kullanıcı bağlamı tüm modüllerce paylaşılır

Buradan sonra sistem gerçek uygulamaya dönüşmeye başlar.

---

# Faz 3 — Yetki tabanlı görünürlük ve modül aktivasyonu

Bu fazın amacı:
**kurulu modül ile yetkili modülü ayırmak**

Bu ayrım çok önemli.

## Hedef

* menüde sadece yetkili modülleri göstermek
* modül açılışını yetkiyle sınırlamak
* ekran/komut bazlı kontrol eklemek

## Bu fazda yapılacaklar

### 1. Yetkiyi iki seviyede ele al

* **modül seviyesi yetki**
* **işlem seviyesi yetki**

Örnek:

* `arsiv.goruntule`
* `arsiv.belge_ekle`
* `arsiv.belge_sil`

### 2. Sol menü filtrelenmeli

Registry tüm modülleri bilir, ama sol menü yalnızca kullanıcının görebildiği modülleri göstermeli.

### 3. Modül içine girişte ikinci kontrol olmalı

Menüde görünse bile modül açılırken tekrar kontrol yapılmalı.

### 4. Komut bazlı UI durumu eklenmeli

Örnek:

* buton disable
* sekme salt okunur
* bazı panel hiç görünmez

## Faz 3 sonucu

Bu faz sonunda sistem sadece “modül gösteren” değil, gerçekten **yetkiyle yaşayan** hale gelir.

---

# Faz 4 — Modül sürümü, host sürümü, backend sürümü, DB şeması uyumluluğu

Bu fazın amacı:
**“en yeni modül” yerine “uyumlu modül” mantığını kurmak**

Burası senin anlattığın nöbet örneği için temel faz.

## Hedef

* host sürümü
* backend API sürümü
* DB şema sürümü
* modül sürüm uyumluluğu
* katalog kararı

## Bu fazda yapılacaklar

### 1. `ModulUyumlulukManifesti` tasarlanmalı

Her modül için şu alanlar olmalı:

* modül adı
* modül sürümü
* min host sürümü
* max host sürümü
* min API sürümü
* min DB şema sürümü
* max DB şema sürümü

### 2. Backend tarafında katalog endpoint’i olmalı

Host açılışta şunu sormalı:

* bu kuruma ait modüller neler
* bu kullanıcı ne görebilir
* bu host sürümüne hangi modül sürümleri uygun
* bu DB şeması için hangi modül sürümü uygun

### 3. Lokal tarafta kurulu modül envanteri tutulmalı

Host şu bilgileri bilecek:

* lokalde hangi modül sürümü var
* aktif sürüm hangisi
* yeni sürüm gerek var mı
* uyumluluk engeli var mı

### 4. Uyuşmazlık ekranı tasarlanmalı

Örnek:

* “Nöbet modülü kurulu ama DB şeması eski”
* “Yönetici güncellemesi gerekli”
* “Bu host sürümüyle bu modül açılamaz”

## Faz 4 sonucu

Bu faz sonunda sistem artık kurumsal dağıtım mantığına yaklaşır.

---

# Faz 5 — Kurum lisansı, modül kataloğu ve yerel paket deposu

Bu fazın amacı:
**modülleri sadece kod olarak değil, ürün paketi olarak yönetmek**

Burada artık satış/lisans/kurulum mantığı girer.

## Hedef

* kurumun sahip olduğu modüller
* cihazda kurulu modüller
* kullanıcının kullanabildiği modüller
* üçlü ayrımı netleştirmek

## Bu fazda yapılacaklar

### 1. Şu ayrım kodda açık hale gelmeli

* `kurum_lisansli_moduller`
* `yerel_kurulu_moduller`
* `kullanici_yetkili_moduller`

### 2. Yerel modül deposu tasarlanmalı

Örnek dizin:

```text
moduller/
  arsiv/
    1.0.0/
    1.1.0/
  nobet/
    2.0.3/
```

### 3. Kurulum manifesti eklenmeli

Her modül paketi için:

* sürüm
* hash
* imza
* bağımlılıklar
* uyumluluk
* ikon/asset
* migration gereksinimi

### 4. Host kurulumu kullanıcı yetkisine göre değil, lisans + uyumluluğa göre yapmalı

Bu, önceki konuşmamızdaki önemli nokta.

Yani:

* kullanıcı yetki alınca modül görünür olabilir
* ama modülün cihaza kurulması asıl olarak kurum lisansı ve sürüm uyumluluğuyla belirlenmeli

## Faz 5 sonucu

Bu fazdan sonra sistem artık gerçek anlamda “ürünleştirilebilir modül platformu” olmaya başlar.

---

# Faz 6 — Dinamik modül yükleme (DLL / DYLIB / SO)

Bu fazın amacı:
**statik modül modelinden dinamik paket modeline geçmek**

Bunu özellikle Faz 1–5 tamamlanmadan önermem.

## Hedef

* modül yükleyici
* modül yaşam döngüsü
* sürümlü klasörden yükleme
* güvenli aktive etme

## Bu fazda yapılacaklar

### 1. `modul_sistemi` ikiye ayrılmalı

Ben burada tek crate yerine ayırmanı öneririm:

* `modul_sozlesmesi`
* `modul_yukleyici`
* `modul_katalogu`

### 2. ABI sınırı tanımlanmalı

Burada artık önceki uyarımız devreye girer:

**host ile plugin arasında ham Rust tipleri taşınmamalı**

Yani artık şunları crossing yapmıyorsun:

* `gpui::View`
* trait object
* generic struct
* host iç entity tipleri
* DB pool

Yerine:

* manifest çağrıları
* function table
* opaque handle
* serialize edilmiş mesaj

### 3. Yüklenen modül yerinde overwrite edilmemeli

Sürümlü klasör + aktif sürüm pointer mantığı kullanılmalı.

### 4. İlk dinamikleştirilecek modüller doğru seçilmeli

İlk adaylar:

* `arsiv`
* `nobet`
* `personel`

Çünkü bunlar büyük iş modülleri.

Ama:

* `yetki`
* `kullanici_yonetimi`
* `kurum`

başta statik de kalabilir. Çünkü bunlar çekirdeğe daha yakın.

## Faz 6 sonucu

Bu faz sonunda istediğin “sadece ilgili modülü güncelle” modeli mümkün olur.

---

# Faz 7 — Güncelleme yöneticisi ve bakım modu

Bu fazın amacı:
**modül güncellemesi, host güncellemesi ve DB migration sürecini birbirine bağlamak**

## Hedef

* normal kullanıcı açılışı
* yönetici bakım açılışı
* migration gereksinimi
* güncelleme akışı

## Bu fazda yapılacaklar

### 1. Açılış akışı ikiye ayrılmalı

#### Normal kullanıcı akışı

* giriş yap
* uyumluluğu kontrol et
* eksik modülü indir
* uyumsuz modülü açma
* migration gerekiyorsa bilgi ver

#### Yönetici/bakım akışı

* migration planı al
* yedek doğrulaması
* backend üzerinden güncelleme tetikle
* modül aktivasyonunu değiştir

### 2. Masaüstü istemci migration motoru olmamalı

Ben bunu özellikle ayrı söylüyorum.

Desktop uygulama:

* migration gerektiğini saptasın
* backend’e talep atsın
* sonucu izlesin

Ama normal kullanıcı masaüstü uygulaması gidip rastgele SQL migration çalıştırmasın.

### 3. Güncelleme karar motoru eklenmeli

Sistem şu kararı vermeli:

* host yeterli mi
* backend yeterli mi
* DB şeması yeterli mi
* modül sürümü uygun mu
* lisans var mı
* kullanıcı yetkisi var mı

Bu altı kontrol artık merkezi hale gelmeli.

## Faz 7 sonucu

Burada sistem artık profesyonel ürün yaşam döngüsüne ulaşır.

---

# Faz X — Uzun vadeli hedef mimari

Bu faz “sonraki yıllar” seviyesi.

## Hedefler

* 80–130 modül
* modül marketi/kataloğu
* tenant/kurum bazlı paketleme
* çevrimdışı kurulum paketleri
* modül bağımlılık grafiği
* telemetry / health
* rollback
* A/B modül geçişi
* modül başına lisans

## Bu fazda istenecek şeyler

* modül bağımlılık çözümleyicisi
* imzalı paket
* rollback stratejisi
* bozuk modül izolasyonu
* log/telemetry
* modül çökse host çökmesin yaklaşımı
* gerekirse bazı modülleri ayrı process çalışma modeline taşıma

Bu artık “uygulama” değil, küçük bir ürün platformu olur.

---

# Sana özel net yönlendirme

Bu projede en doğru sırayı tek cümlede söylersem:

**Önce statik modül platformunu kur, sonra oturum/yetkiyi yerleştir, sonra sürüm-uyumluluk kataloğunu kur, en son dinamik modül paketine geç.**

Şu anki kod yapısına göre doğrudan DLL modeline atlamak erken.

---

# Dosya bazlı ilk müdahale önerim

İlk iş olarak şu dosyalar hedef alınmalı:

### Hemen doldurulacaklar

* `kutuphaneler/modul_sistemi/src/lib.rs`
* `kutuphaneler/sunucu_istemcisi/src/lib.rs`
* `kutuphaneler/ortak_tipler/src/lib.rs`
* `kutuphaneler/calisma_sekmeleri/src/lib.rs`

### Davranışı büyütülecekler

* `kutuphaneler/sol_menu/src/lib.rs`
* `kutuphaneler/uygulama_kabugu/src/ana_panel.rs`

### Gerçek modüle çevrilecekler

* `moduller/arsiv`
* `moduller/kullanici_yonetimi`
* `moduller/kurum`
* `moduller/yetki`

### Sonraya bırakılacaklar

* gerçek dinamik yükleme
* modül indirme/kurma
* otomatik güncelleme
* DB şema uyumluluk motoru

---

# Başlangıç sırası

Ben olsam bu hafta şu sırayla ilerlerim:

1. `modul_sistemi` için statik trait + manifest + registry
2. `sol_menu`yu registry’den besleme
3. `calisma_sekmeleri` ile merkez alan açma
4. `arsiv` modülünü ilk örnek modül yapma
5. `sunucu_istemcisi` içine login + session iskeleti
6. `ortak_tipler` içine oturum/yetki DTO’ları
7. sonra Faz 2’ye geçiş
