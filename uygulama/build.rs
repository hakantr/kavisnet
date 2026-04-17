//! Yalnizca Windows hedefi icin .exe'ye uygulama ikonunu ve versiyon
//! metadata'sini gomer. Diger hedeflerde (macOS/Linux) script hicbir sey
//! yapmaz. build.rs host uzerinde derlendigi icin `target_os` yerine
//! `CARGO_CFG_TARGET_OS` env var'ina bakariz — cross-compile guvenli.
//!
//! Akis:
//! 1. `resimler/ikonlar/ikon_02_{boyut}.png` dosyalarini `ico` crate'i ile
//!    tek multi-resolution `.ico` olarak `OUT_DIR`'e yazar.
//! 2. `winresource::WindowsResource` bu `.ico`'yu resource ID 1 olarak exe'ye
//!    gomer. GPUI `gpui_windows::platform::load_icon()` `LoadImageW(module,
//!    PCWSTR(1 as _), IMAGE_ICON, ...)` cagrisiyla ayni ID'den okuyor; bu
//!    sayede pencere/taskbar/alt-tab hepsi dogru ikonu gosteriyor.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        windows_ikonunu_gom();
    }
}

fn windows_ikonunu_gom() {
    use ico::{IconDir, IconDirEntry, IconImage, ResourceType};
    use std::fs::File;
    use std::path::Path;

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR bulunamadi");
    let ico_yolu = Path::new(&out_dir).join("KavisNet.ico");

    let repo_kok = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("uygulama crate'inin ust dizini yok");
    let ikon_dizini = repo_kok.join("resimler/ikonlar");

    // Windows'un farkli baglamlarda (tray, title bar, alt-tab, start menu)
    // en uygun olani secebilmesi icin coklu boyut. 256 .ico formatinin
    // standart ust siniri; 512 icin PNG ayri verilmesi gerekir ve gerek yok.
    let boyutlar: [u32; 6] = [16, 32, 48, 64, 128, 256];

    let mut ico = IconDir::new(ResourceType::Icon);
    for boyut in boyutlar {
        let png_yolu = ikon_dizini.join(format!("ikon_02_{boyut}.png"));
        println!("cargo:rerun-if-changed={}", png_yolu.display());

        // Indexed/palette PNG'ler `ico::IconImage::read_png` tarafindan
        // kabul edilmiyor; `image` crate'i her varyanti RGBA8'e decode
        // ediyor, sonra ham byte'lari dogrudan `.from_rgba_data`'ya
        // veriyoruz.
        let rgba = image::open(&png_yolu)
            .unwrap_or_else(|e| panic!("PNG acilamadi ({}): {e}", png_yolu.display()))
            .to_rgba8();
        let (genislik, yukseklik) = rgba.dimensions();
        let icon_image = IconImage::from_rgba_data(genislik, yukseklik, rgba.into_raw());
        let entry = IconDirEntry::encode(&icon_image)
            .unwrap_or_else(|e| panic!("ico entry olusturulamadi ({boyut}): {e}"));
        ico.add_entry(entry);
    }

    let out_file =
        File::create(&ico_yolu).unwrap_or_else(|e| panic!(".ico yazilamadi ({}): {e}", ico_yolu.display()));
    ico.write(out_file).expect(".ico yazma hatasi");

    let mut res = winresource::WindowsResource::new();
    res.set_icon(ico_yolu.to_str().expect(".ico yolu UTF-8 degil"));
    res.set("FileDescription", "KavisNet");
    res.set("ProductName", "KavisNet");
    res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
    res.set("FileVersion", env!("CARGO_PKG_VERSION"));
    if let Err(e) = res.compile() {
        eprintln!("winresource derleme hatasi: {e}");
        std::process::exit(1);
    }
}
