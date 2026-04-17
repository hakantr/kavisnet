use gpui::*;

/// Pencere gölgesi boyutu (CSD modunda). Resize hitbox ve box-shadow ikisi de
/// bu değerle hizalanır.
pub const GOLGE_BOYUTU: Pixels = px(10.0);

/// Fare konumunu incelcik bir resize şeridine göre değerlendirip hangi
/// kenarda olduğumuzu döner. Hiçbir kenarda değilse None.
pub fn golge_kenar_bul(
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
