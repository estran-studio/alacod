# m3-t0d-contrats-familiers-champions — familiers, orbitaux et champions v2 (M3, vague 0)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md` (« Numérotation des entités »), `docs/taches.md` §8.1,
`docs/conventions.md` §4, §8 (équipes), §20 (émetteurs), §22 (behaviors), §25 (variantes et élites),
§36 (objets), §41 (blank : effacement de balles), §49 (formations), et le code
`crates/behaviors/src/lib.rs` (`Behavior`, `select`), `crates/combat/src/emitter.rs`,
`crates/game/src/character/variant.rs` (`VariantsConfig`, `VariantDef`, tirage local),
`crates/game/src/character/enemy/create.rs`, `crates/game/src/items.rs` (ramassage), `crates/items/src/lib.rs`.
Branche `m3-t0d-contrats-familiers-champions` depuis `origin/main`. Estimation : 2 j. Section de conventions
réservée : **§63**.

## But

Poser les contrats dont dépendent M3-T6 (familiers et orbitaux) et M3-T7 (champions et élites v2). **Aucune
trace existante ne bouge.**

## Contexte (état de `main`)

- Aucun familier, orbital ni compagnon. Briques réutilisables : un `Emitter` sur toute entité qui a
  `GgrsNetId`, `FixedTransform3D` et `Team` ; `Team::Allies` ; `Formation { Arc }` (placement autour d'un
  point) ; l'effacement de balles du blank.
- L'IA ennemie ne cible que `With<Player>` : un familier n'est ni visé ni attaqué. C'est ce qu'Isaac fait
  pour la plupart des familiers ; ne pas changer ce ciblage ici (allié qui combat : D5, hors M3).
- Une variante (§25) porte des modificateurs, des tags et un skin ; tirage local de graine
  `fnv1a("variants") ^ run_seed ^ net_id`, indépendant du moment. Un champion est une variante taguée
  `champion`, sans rien de plus : ni comportement, ni action à la mort, ni butin.

## À faire

1. **Familier = personnage** de l'équipe `Players` (ou `Allies`, à justifier) décrit dans `characters/`
   comme les autres, sans `ai` de poursuite, avec des behaviors nouveaux, en **fin d'enum** `Behavior` :
   - `Follow { gap }` : suit son porteur à `gap` unités, en file derrière les autres familiers du même
     porteur (rang = ordre d'apparition) ;
   - `Orbit { radius, frames_per_turn }` : tourne autour du porteur, phase = rang × 2π / nombre d'orbitaux ;
   - `MirrorShot { weapon, every }` : tire quand le porteur tire, dans sa direction de visée (`pan_x/pan_y`
     de l'input du porteur), au plus une fois tous les `every` frames ; arme du familier ou larmes du
     porteur : **Q13** (proposition par défaut : l'arme du familier, sans les modificateurs du porteur) ;
   - `BlockBullets` : une balle hostile qui touche le familier est détruite (`despawn_rollback`, comme le
     blank) ; le familier est immunisé (`Defenses`).
   Types, sérialisation, lint (porteur requis : ces behaviors ne sont valides que sur un personnage cité
   par un objet `familiar`) ; exécution en T6. `select` les ignore tant que T6 n'est pas livrée.
2. **Lien au porteur** : composant rollback neutre `FamiliarOf { owner: GgrsNetId, rank: u32 }`. Le familier
   est créé et détruit par les objets (point 3) ; son `GgrsNetId` vient de `id_factory` au ramassage, dans
   l'ordre déterministe des ramassages de la frame.
3. **Champ d'objet** : `ItemDef.familiar: Option<FamiliarRef { character: String, per_copy: u32 }>` (défaut
   1 par exemplaire), hors hash quand absent. Contrat : forme, lint (personnage connu, qui a au moins un
   behavior de familier) ; création et destruction en T6.
4. **Champions v2** : champs optionnels de `VariantDef`, hors hash et sans effet quand absents :
   - `on_death: [Action]` (actions d'`effects`, dont `Explode` de M3-T0c et une action nouvelle
     `SpawnCharacters { id, count }` en fin d'enum pour la division, ou réutiliser `SpawnEnemies` du boss si
     son RNG convient : justifier) ;
   - `behaviors: Option<Vec<BehaviorRule>>` (remplace la liste du personnage) ;
   - `loot: Option<String>` (table de butin lâchée à la mort, tirée par la graine de butin de §48 avec la clé
     de contenu `champion:<net_id>`) ;
   - `tint: Option<String>` (couleur `#rrggbb`, présentation seule) ;
   - `chance` accepte une expression (`NumOrExpr`, §18 : `players`, `floor`) en plus d'un nombre, valeur
     identique quand c'est un nombre.
   Le composant `Variant(nom)` ne change pas (hash identique). Lint : action refusée à la mort (`OnUse`…),
   table inconnue, couleur invalide. Exécution en T7.
5. **Kinds** : déclarer les nouveaux behaviors (`register_kinds`) ; attentes `FamiliarCount(handle, n)` et
   `EnemyVariant` (existe) ; mettre à jour la liste de `CLAUDE.md` (ligne M3).
6. **Bout en bout minimal (testbed)** : un objet `compagnon` qui cite un personnage `compagnon` à
   `Follow` + `MirrorShot` ; contrat seulement : le ramassage ne crée rien encore, le lint accepte le RON.
   Un scénario de lint (fixture valide et deux invalides) suffit ; pas de scénario de jeu.

## Règles

Aucune trace existante ne bouge : variantes en fin d'enum, champs optionnels hors hash, `FamiliarOf` neutre.
`Behavior` a déjà plusieurs variantes (pas de piège de discriminant). Pas de `f32` (phase de l'orbite par
`fixed_math`), `order_iter!`, `BTreeMap`. `behaviors` reste dans la frontière `SIM`.

## Hors périmètre

Création et comportement des familiers (M3-T6), champions en jeu (M3-T7), visuels (M3-T17), alliés qui
combattent (D5, hors M3).

## Vérification

README §4 complet ; rapport `docs/taches/rapports/m3-t0d-contrats-familiers-champions.md`, conventions §63.

## Atteignabilité

Contrats : rien n'est rencontré en partie ordinaire. T6 et T7 l'exécutent ; T11b met les familiers dans les
pools de `isaac`, T11c les champions sur ses ennemis.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Ignorer les consignes `orca` du README (§1). Pas de
Docker : le dire au rapport. Pousser la branche régulièrement. Voie rapide du README §6 : fusionner
`origin/main` et rejouer la suite juste avant le commit `LIVRÉ m3-t0d-contrats-familiers-champions`, qui
contient le rapport.
