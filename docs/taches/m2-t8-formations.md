# m2-t8-formations — formations simples : escouades en arc, tir alterné (M2, V1c, chantier D1 v2)

Lire d'abord `docs/taches/README.md` (machine `orca`, §1 : `make sweep`, `make test_crates`,
signaling par session), `CLAUDE.md` (déterminisme, flow field, tie-break par `GgrsNetId`),
`docs/conventions.md` §22 (behaviors composables : `Chase`, `KeepDistance`, `Strafe`, `Shoot`…),
§25 (variantes), §28 (personnages placés), §35 (salles), §41 (patterns v2). Branche
`m2-t8-formations` depuis `main`. Estimation : 2 j. Section de conventions réservée : **§49**.

## Contexte

Chaque ennemi décide seul (règles de `behaviors`, flow field partagé). Les ennemis à distance de
`gungeon` (`bullet_kin`) s'agglutinent sur la même ligne et tirent ensemble : les salves se
superposent et la salle se lit mal. Une formation simple les répartit et alterne leurs tirs.

## À faire

1. **Escouade** : regroupement en donnée (`squad: Some((id, role))` sur un ennemi placé ou par
   règle de spawn de salle) ; l'escouade est une donnée dérivée de l'état (membres vivants de même
   id dans la même salle, triés par `GgrsNetId`), pas un nouvel état si possible ; sinon composant
   rollback **neutre**.
2. **Règle `Formation(Arc(radius, spread))`** (variante en fin d'enum) : chaque membre vise une
   place sur un arc centré sur la cible (joueur le plus proche par le chemin, tie-break `GgrsNetId`),
   places attribuées par rang de `GgrsNetId` (sans réattribution tant que l'effectif ne change
   pas) ; se déplace via le flow field jusqu'à sa place, puis `Strafe` léger autour.
3. **Tir alterné** : `Shoot` d'un membre ne part qu'à son tour (`slot = (frame / period) mod
   effectif == rang`, état dérivé, aucun curseur) ; option `volley` pour garder le tir groupé.
4. **Lint** : rayon ≤ 0, `spread` hors ]0, 2π], `period` 0, formation sans `Shoot` ni déplacement →
   refusés ; fixtures.
5. **Contenu `gungeon`** : une escouade de 3 `bullet_kin` dans un gabarit de salle de combat
   (si la grammaire reste valide sur 1 000 graines), un ennemi `bullet_kin_squad` dédié plutôt que
   modifier `bullet_kin` (traces gungeon existantes intactes).
6. **Scénarios** : `formation_arc` (3 membres, `EnemyDistance` ≈ rayon, écarts angulaires),
   `formation_member_dies` (réattribution des places à l'effectif 2), `formation_alternate_fire`
   (`BulletCount` : un seul tireur par période), `formation_two_players` (cible la plus proche,
   tie-break).

## Règles

Aucune trace existante ne doit bouger (nouveaux types en variante **neutre**, variantes en fin
d'enum, `bullet_kin` inchangé) ; toute trace qui bouge : preuve §5 (`ALACOD_DUMP_TRACE` main /
branche + `trace-diff.py`), cause dans le commit, avant tout bless. Pas de `f32` (angles en
`Fixed`, table trigonométrique existante), `order_iter!`, BTreeMap. `cargo fmt --all -- --check`
avant de livrer.

## Hors périmètre

Formations complexes (lignes, encerclement dynamique), communication entre escouades, IA de boss
(M2-T7).

## Vérification

README §4 (`make test_scenarios`, `make test_crates`, lint ×4, fmt, scripts, `make gen` ×4) ;
p2p à deux clients sur `gungeon` (port 3536, `MATCHBOX_PORT`) ; rapport
`docs/taches/rapports/m2-t8-formations.md`, conventions §49. Livrer par
`LIVRÉ m2-t8-formations <sha>`.
