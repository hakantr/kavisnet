//! Ortak ikon havuzu. SVG varlıkları GPUI `AssetSource` üzerinden
//! sunulur; render tarafı `svg().path("ikonlar/…svg")` çağrısıyla erişir.
//!
//! GPUI SVG'yi monokrom alpha mask olarak çizdiği için SVG'deki
//! `stroke`/`fill` renkleri önemsiz — `text_color` ile verilen renk maske
//! üzerine basılır. Bu sayede aynı dosya koyu/açık temada tek kaynak olur.

use gpui::{AssetSource, Result, SharedString};
use std::borrow::Cow;

/// Bütün gömülü SVG'lerin yol → bayt eşlemesi. Yeni ikon eklenince
/// buraya bir satır eklemek yeter.
const SVG_VARLIKLARI: &[(&str, &[u8])] = &[
    (
        "ikonlar/kontrol_kucult.svg",
        include_bytes!("svgler/kontrol_kucult.svg"),
    ),
    (
        "ikonlar/kontrol_buyut.svg",
        include_bytes!("svgler/kontrol_buyut.svg"),
    ),
    (
        "ikonlar/kontrol_restore.svg",
        include_bytes!("svgler/kontrol_restore.svg"),
    ),
    (
        "ikonlar/kontrol_kapat.svg",
        include_bytes!("svgler/kontrol_kapat.svg"),
    ),
];

/// Zed'in asset kaynağıyla aynı rolü oynayan struct. `Application`
/// fabrikasında `.with_assets(VarlikKaynagi)` ile kaydedilir.
pub struct VarlikKaynagi;

impl AssetSource for VarlikKaynagi {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        for (yol, veri) in SVG_VARLIKLARI {
            if *yol == path {
                return Ok(Some(Cow::Borrowed(*veri)));
            }
        }
        Ok(None)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(SVG_VARLIKLARI
            .iter()
            .filter_map(|(yol, _)| yol.strip_prefix(path).map(|_| SharedString::from(*yol)))
            .collect())
    }
}
