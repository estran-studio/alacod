//! Repli derrière un couvert (X-6c, `docs/conventions.md` §120) : géométrie **pure** du behavior
//! `TakeCover`, en `Fixed` et en entiers (aucun `f32`, aucun RNG). `rules::behavior_select_system`
//! l'appelle au moment du déclenchement ; l'attente de scénario `CoverSight` (`scenario`) rejoue le
//! même test de segment pour mesurer la ligne de vue.
//!
//! Un couvert (`sim_core::cover::Cover`, X-6a2) arrête les balles sur son collider ; il **ne figure pas**
//! dans `FlowFieldCache::wall_cells`, donc `line_of_sight` ne le voit pas : la ligne de vue coupée est
//! ici le segment cible → point qui traverse la boîte du collider ([`segment_touches_box`]).
//!
//! Choix du point abrité ([`find_shelter`]) : huit points autour de chaque couvert (les quatre côtés
//! et les quatre coins, où le corps du tireur est à `BODY_GAP` du bord, ordre fixe) ; retenus s'ils sont à `radius` de l'ennemi, si
//! le segment depuis la cible traverse ce couvert, si ni le trajet de l'ennemi ni son **corps** à
//! l'arrivée (demi-étendues et décalage de son collider, [`Body`]) ne chevauchent un couvert, et si le
//! terrain est libre (`free` : murs sous le corps). Le plus proche l'emporte ; sont à égalité tous les points à
//! moins de `DISTANCE_TOLERANCE` (un seizième d'unité : le tireur dérive de quelques bits) de la plus
//! courte distance, départagés par le plus petit `GgrsNetId` du couvert, puis le plus petit `x`, puis le
//! plus petit `y` : le résultat ne dépend pas de l'ordre de `covers`. Déjà à l'abri d'un couvert : il
//! reste où il est.

use bevy_fixed::fixed_math::{Fixed, FixedVec2, FIXED_HALF};
use combat::grid::Aabb;

/// Boîte d'un couvert (collider positionné) et son `GgrsNetId`.
#[derive(Clone, Copy, Debug)]
pub struct CoverBox {
    pub net_id: usize,
    pub aabb: Aabb,
}

/// Deux points sont « à égale distance » du tireur à un seizième d'unité près.
const DISTANCE_TOLERANCE: Fixed = Fixed::from_bits(1 << 12);

/// Directions des huit points autour d'un couvert (ordre fixe : E, NE, N, NO, O, SO, S, SE).
const RING: [(i32, i32); 8] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];

/// Le segment `a`→`b` touche la boîte (bords compris). Exact, en entiers (les bits des `Fixed`, `i128`) :
/// les boîtes englobantes se recouvrent, et les quatre coins de la boîte ne sont pas tous du même côté
/// de la droite `ab` (produits croisés, aucune division).
pub fn segment_touches_box(a: FixedVec2, b: FixedVec2, aabb: &Aabb) -> bool {
    let bits = |value: Fixed| i128::from(value.to_bits());
    let (ax, ay, bx, by) = (bits(a.x), bits(a.y), bits(b.x), bits(b.y));
    let (min_x, min_y) = (bits(aabb.min.x), bits(aabb.min.y));
    let (max_x, max_y) = (bits(aabb.max.x), bits(aabb.max.y));
    if ax.max(bx) < min_x || ax.min(bx) > max_x || ay.max(by) < min_y || ay.min(by) > max_y {
        return false;
    }
    let (dx, dy) = (bx - ax, by - ay);
    let side = |cx: i128, cy: i128| (cx - ax) * dy - (cy - ay) * dx;
    let sides = [
        side(min_x, min_y),
        side(max_x, min_y),
        side(min_x, max_y),
        side(max_x, max_y),
    ];
    !(sides.iter().all(|s| *s > 0) || sides.iter().all(|s| *s < 0))
}

/// Un couvert (le plus petit `GgrsNetId` si plusieurs) coupe le segment `from`→`to`.
pub fn cutting_cover(from: FixedVec2, to: FixedVec2, covers: &[CoverBox]) -> Option<usize> {
    covers
        .iter()
        .filter(|cover| segment_touches_box(from, to, &cover.aabb))
        .map(|cover| cover.net_id)
        .min()
}

/// Point abrité choisi et couvert qui l'abrite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shelter {
    pub point: FixedVec2,
    pub cover: usize,
}

/// Jeu laissé entre le corps du tireur et le bord du couvert à l'arrivée (unités).
pub const BODY_GAP: i32 = 1;

/// Corps du tireur : demi-étendues et décalage de son collider par rapport à sa position (le collider
/// d'un personnage est décalé, ex. `offset.y = -6`).
#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub half: FixedVec2,
    pub offset: FixedVec2,
}

impl Body {
    /// Boîte du corps quand la position du tireur est `point`.
    pub fn aabb_at(&self, point: FixedVec2) -> Aabb {
        let center = point + self.offset;
        Aabb {
            min: center - self.half,
            max: center + self.half,
        }
    }
}

/// Les deux boîtes se chevauchent sur une surface (se toucher par un bord ne compte pas).
pub fn boxes_overlap(a: &Aabb, b: &Aabb) -> bool {
    a.min.x < b.max.x && a.max.x > b.min.x && a.min.y < b.max.y && a.max.y > b.min.y
}

/// Voir le module. `free(from, body, point)` : le terrain est libre sur le segment `from`→`point`
/// (murs) et sous la boîte `body` du tireur arrivé en `point`.
pub fn find_shelter(
    position: FixedVec2,
    target: FixedVec2,
    radius: Fixed,
    body: Body,
    covers: &[CoverBox],
    free: impl Fn(FixedVec2, &Aabb, FixedVec2) -> bool,
) -> Option<Shelter> {
    if let Some(cover) = cutting_cover(target, position + body.offset, covers) {
        return Some(Shelter {
            point: position,
            cover,
        });
    }
    // Deux passes, indépendantes de l'ordre de `covers` : la plus courte distance valide, puis, parmi
    // les points à moins de `DISTANCE_TOLERANCE` de celle-ci, la plus petite clé (`GgrsNetId`, `x`, `y`).
    let each_candidate = |visit: &mut dyn FnMut(Fixed, usize, FixedVec2)| {
        for cover in covers {
            let center = FixedVec2::new(
                (cover.aabb.min.x + cover.aabb.max.x) * FIXED_HALF,
                (cover.aabb.min.y + cover.aabb.max.y) * FIXED_HALF,
            );
            let gap = Fixed::from_num(BODY_GAP);
            let half_w = (cover.aabb.max.x - cover.aabb.min.x) * FIXED_HALF + body.half.x + gap;
            let half_h = (cover.aabb.max.y - cover.aabb.min.y) * FIXED_HALF + body.half.y + gap;
            for (sx, sy) in RING {
                // Le **corps** (et non la position) touche presque le couvert : on retire le décalage.
                let point = FixedVec2::new(
                    center.x + half_w * Fixed::from_num(sx),
                    center.y + half_h * Fixed::from_num(sy),
                ) - body.offset;
                let distance = position.distance(&point);
                // Ligne de vue : vers le **centre du corps** (là où les balles frappent), pas vers la
                // position (décalée de `offset`), et encore coupée si le tireur s'arrête à
                // `ARRIVAL_SLACK` du point (tolérance d'arrivée de `rules`).
                let eye = point + body.offset;
                let slack = Fixed::from_num(super::rules::RETURN_ARRIVE);
                let hidden = |at: FixedVec2| segment_touches_box(target, at, &cover.aabb);
                if distance > radius
                    || !hidden(eye)
                    || !hidden(eye + FixedVec2::new(slack, Fixed::ZERO))
                    || !hidden(eye - FixedVec2::new(slack, Fixed::ZERO))
                    || !hidden(eye + FixedVec2::new(Fixed::ZERO, slack))
                    || !hidden(eye - FixedVec2::new(Fixed::ZERO, slack))
                {
                    continue;
                }
                let body_box = body.aabb_at(point);
                // Ni le trajet ni le corps d'arrivée ne traversent un couvert (la rangée de caisses
                // voisine du couvert choisi compte), et le terrain est libre.
                if covers.iter().all(|other| {
                    !segment_touches_box(position, point, &other.aabb)
                        && !boxes_overlap(&body_box, &other.aabb)
                }) && free(position, &body_box, point)
                {
                    visit(distance, cover.net_id, point);
                }
            }
        }
    };
    let mut nearest: Option<Fixed> = None;
    each_candidate(&mut |distance, _, _| {
        nearest = Some(nearest.map_or(distance, |current| current.min(distance)));
    });
    let nearest = nearest?;
    let mut best: Option<(usize, Fixed, Fixed)> = None;
    each_candidate(&mut |distance, cover, point| {
        if distance <= nearest + DISTANCE_TOLERANCE {
            let key = (cover, point.x, point.y);
            if best.is_none_or(|current| key < current) {
                best = Some(key);
            }
        }
    });
    best.map(|(cover, x, y)| Shelter {
        point: FixedVec2::new(x, y),
        cover,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f32, y: f32) -> FixedVec2 {
        FixedVec2::new(Fixed::from_num(x), Fixed::from_num(y))
    }

    fn crate_at(net_id: usize, x: f32, y: f32) -> CoverBox {
        CoverBox {
            net_id,
            aabb: Aabb {
                min: v(x - 10.0, y - 10.0),
                max: v(x + 10.0, y + 10.0),
            },
        }
    }

    fn open(_: FixedVec2, _: &Aabb, _: FixedVec2) -> bool {
        true
    }

    /// Corps de 20 × 20 sans décalage.
    fn body() -> Body {
        Body {
            half: v(10.0, 10.0),
            offset: v(0.0, 0.0),
        }
    }

    #[test]
    fn segment_crossing_a_box() {
        let aabb = crate_at(1, 0.0, 0.0).aabb;
        assert!(segment_touches_box(v(-30.0, 0.0), v(30.0, 0.0), &aabb));
        assert!(segment_touches_box(v(-30.0, -30.0), v(30.0, 30.0), &aabb));
        // Un bord touché compte ; un segment qui passe à côté ou s'arrête avant, non.
        assert!(segment_touches_box(v(-30.0, 10.0), v(30.0, 10.0), &aabb));
        assert!(!segment_touches_box(v(-30.0, 11.0), v(30.0, 11.0), &aabb));
        assert!(!segment_touches_box(v(-30.0, 0.0), v(-11.0, 0.0), &aabb));
        // Boîte englobante qui recouvre, mais droite qui rase le coin : pas de contact.
        assert!(!segment_touches_box(v(-30.0, 55.0), v(55.0, -30.0), &aabb));
    }

    #[test]
    fn shelter_is_behind_the_cover_and_reachable() {
        let cover = crate_at(7, 100.0, 0.0);
        let target = v(0.0, 0.0);
        let shooter = v(130.0, 30.0);
        let found =
            find_shelter(shooter, target, Fixed::from_num(80), body(), &[cover], open).unwrap();
        assert_eq!(found.cover, 7);
        assert!(segment_touches_box(target, found.point, &cover.aabb));
        assert!(!segment_touches_box(shooter, found.point, &cover.aabb));
        assert!(found.point.distance(&shooter) <= Fixed::from_num(80));
    }

    #[test]
    fn no_cover_in_range_gives_no_shelter() {
        let cover = crate_at(7, 100.0, 0.0);
        assert!(find_shelter(
            v(130.0, 30.0),
            v(0.0, 0.0),
            Fixed::from_num(5),
            body(),
            &[cover],
            open,
        )
        .is_none());
        assert!(find_shelter(
            v(130.0, 30.0),
            v(0.0, 0.0),
            Fixed::from_num(80),
            body(),
            &[],
            open,
        )
        .is_none());
        // Terrain bloqué partout : rien.
        assert!(find_shelter(
            v(130.0, 30.0),
            v(0.0, 0.0),
            Fixed::from_num(80),
            body(),
            &[cover],
            |_, _, _| false,
        )
        .is_none());
    }

    #[test]
    fn already_hidden_stays_put() {
        let cover = crate_at(3, 100.0, 0.0);
        let hidden = v(125.0, 0.0);
        let found = find_shelter(
            hidden,
            v(0.0, 0.0),
            Fixed::from_num(80),
            body(),
            &[cover],
            open,
        )
        .unwrap();
        assert_eq!(found.point, hidden);
        assert_eq!(found.cover, 3);
    }

    /// Deux couverts symétriques à égale distance : le plus petit `GgrsNetId` l'emporte, quel que soit
    /// l'ordre de la liste.
    #[test]
    fn equal_distance_picks_the_smaller_net_id() {
        let target = v(0.0, 0.0);
        let shooter = v(130.0, 0.0);
        let north = crate_at(9, 100.0, 40.0);
        let south = crate_at(4, 100.0, -40.0);
        let pick = |covers: &[CoverBox]| {
            find_shelter(shooter, target, Fixed::from_num(120), body(), covers, open).unwrap()
        };
        let a = pick(&[north, south]);
        let b = pick(&[south, north]);
        assert_eq!(a, b);
        assert_eq!(a.cover, 4);
        // Le tireur dérive de quelques bits (0,0002 unité) : l'égalité tient, le plus petit id l'emporte.
        let drifted = find_shelter(
            v(130.0, 0.0002),
            target,
            Fixed::from_num(120),
            body(),
            &[north, south],
            open,
        )
        .unwrap();
        assert_eq!(drifted.cover, 4);
    }

    /// Un point abrité dont le corps chevauche une caisse voisine (rangée) est écarté ; seul celui du
    /// bout de la rangée reste.
    #[test]
    fn a_neighbouring_cover_blocks_the_points_it_overlaps() {
        let target = v(0.0, 0.0);
        let shooter = v(100.0, 36.0);
        let near = crate_at(1, 80.0, 0.0);
        let row_end = crate_at(2, 100.0, 0.0);
        let found = find_shelter(
            shooter,
            target,
            Fixed::from_num(80),
            body(),
            &[near, row_end],
            open,
        )
        .unwrap();
        for cover in [near, row_end] {
            assert!(!boxes_overlap(&body().aabb_at(found.point), &cover.aabb));
        }
        assert_eq!(found.cover, 2);
        assert!(found.point.x > Fixed::from_num(110));
    }

    /// Le décalage du collider compte : avec `offset.y = -6` la position est 6 unités plus bas que le
    /// centre du corps, et le corps (non la position) touche le couvert à `BODY_GAP`.
    #[test]
    fn the_collider_offset_moves_the_point_not_the_body() {
        let cover = crate_at(5, 100.0, 0.0);
        let shifted = Body {
            half: v(10.0, 10.0),
            offset: v(0.0, -6.0),
        };
        let found = find_shelter(
            v(130.0, 30.0),
            v(0.0, 0.0),
            Fixed::from_num(80),
            shifted,
            &[cover],
            open,
        )
        .unwrap();
        let body_box = shifted.aabb_at(found.point);
        // Corps collé au bord est du couvert (x = 110) à un jeu de 1, centré en y sur le couvert.
        assert_eq!(body_box.min.x, Fixed::from_num(111));
        assert_eq!((body_box.min.y + body_box.max.y) * FIXED_HALF, Fixed::ZERO);
        assert_eq!(found.point.y, Fixed::from_num(6));
    }

    /// Un mur sous le corps d'arrivée (pas seulement sous la position) écarte le point.
    #[test]
    fn a_wall_under_the_arrival_body_rejects_the_point() {
        let cover = crate_at(5, 100.0, 0.0);
        // Mur plein à partir de x = 115 : le point est à x = 121 (corps 111..131) → rejeté ;
        // même si la position (121) est elle-même hors du mur à x < 125 d'une case de 16.
        let wall_from = Fixed::from_num(115);
        let found = find_shelter(
            v(130.0, 30.0),
            v(0.0, 0.0),
            Fixed::from_num(80),
            body(),
            &[cover],
            |_, body_box, _| body_box.max.x <= wall_from,
        );
        assert!(found.is_none());
    }
}
