# m4-t0a-contrats-larmes — contrats des larmes et de l'empilement de projectiles (M4, vague 0)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/taches.md` §8.1 (M4 en tâches),
`docs/conventions.md` §4 (checklist d'un vocabulaire), §9 (stats et modificateurs), §16 (projectiles
composables), §20 et §41 (patterns, projectiles v2), §53 (stats lues, `stat_reader`), et le code
`crates/combat/src/projectile.rs` (`ProjectileModifier`, `resolve_modifiers`, `Projectile`),
`crates/combat/src/weapons/mod.rs` (`MagBulletConfig`, `weapon_rollback_system`, lecture des stats au
tir), `crates/sim_core/src/stats.rs` (`StatId`, `stat_reader`), `crates/items/src/lib.rs` (`ItemDef`).
Branche `m4-t0a-contrats-larmes` depuis `origin/main`. Estimation : 2 à 3 j. Plan :
`docs/plan-engine.md` §5 B5 (v3), ligne M4 du tableau §6. Section de conventions réservée : **§60**.

## But

Poser les contrats dont dépendent M4-T1 (larmes v3), M4-T6 (familiers qui tirent), M4-T11a (contenu de
larmes) et M4-T15 (bench), avec un chemin minimal de bout en bout dans le testbed. **Aucune trace
existante ne bouge.**

## Contexte (état de `main`)

- Les modificateurs de projectile (`Bounce`, `Pierce`, `Size`, `Lifetime`, `Homing`, `Gravity`) ne viennent
  que du mode de tir de l'arme (`FiringModeConfig::projectile`) ou de sa table `projectiles`. Un objet ne
  peut rien ajouter aux tirs de son porteur.
- `resolve_modifiers` écrase : un modificateur répété garde la dernière valeur, et le lint le refuse
  (« modificateur X répété »). Isaac est l'inverse : la valeur d'un objet tient à ce qu'il se combine.
- Stats lues au tir : `Damage`, `FireRate`, `Range`, `ReloadSpeed`. La vitesse de la balle est fixée par
  `BulletType.speed` et sa taille par `Size` de l'arme ; `Luck` n'a aucun lecteur (refusée par le lint).
- Pas de tir sans munitions : `Magless` est une limite totale, son « rechargement » ne remplit rien.
- Constat de lecture (à signaler au rapport, ne pas corriger ici) : `ReloadSpeed` multiplie la **durée** du
  rechargement (`weapons/mod.rs`, `reload_time_seconds.saturating_mul(...)`) ; l'objet gungeon
  `chargeur_rapide` (`Pct 0.3`, « 30 % plus rapide ») ralentit donc le rechargement.

## À faire

1. **Munitions infinies** : variante `MagBulletConfig::Infinite` **en fin d'enum**. Le mode tire tant que
   la cadence le permet, jamais de rechargement, rien à afficher en réserve (le HUD affiche `∞` ou rien :
   T16). Lint : `Infinite` avec `ammo_type` est refusé (pas de réserve à consommer).
2. **Stats de larmes** : `StatId::ShotSpeed` (multiplie la vitesse de la balle au tir) et
   `StatId::ShotSize` (multiplie le rayon de collision et le sprite, comme `Size`), en fin d'enum (après
   `Custom`, pour garder l'ordre et le hash des stats existantes), ajoutées à `stat_reader`. Base 1 pour tout
   personnage ; sans modificateur, produit exact par 1 : aucune valeur ne change.
3. **`Luck` lue** : `Luck` entre dans `stat_reader` avec un lecteur de contrat (`effects::chance_with_luck`,
   fonction pure : chance finale = `clamp(base + luck × par_point, 0, 1)`, en `Fixed`) ; T1 et T2 s'en servent
   pour les chances d'effet. Rien ne l'appelle encore dans la simulation : aucune trace ne bouge.
4. **Modificateurs portés par un objet** : champ optionnel `ItemDef.projectile: Option<ProjectileMods>`
   (`modifiers: [ProjectileModifier]`, `on_hit: [Action]`, `on_expire: [ExpireAction]`), absent = v1 exacte.
   Même champ sur `SynergyDef` (une synergie peut changer les larmes). Hors hash quand absent. Le tir **ne le
   lit pas encore** (c'est T1).
5. **Composition** : fonction pure `combat::projectile::compose(base: &[ProjectileModifier], ajouts:
   &[&[ProjectileModifier]]) -> ResolvedModifiers` et sa table de règles, documentée en §60. Proposition par
   défaut, **à valider par William (Q3)** :

   | Modificateur | Règle | Pourquoi |
   |---|---|---|
   | `Bounce(n)`, `Pierce(n)` | addition des `n` | deux objets de rebond rebondissent plus |
   | `Size(f)` | produit | comme une stat multiplicative |
   | `Lifetime(n)` | maximum | une larme vit au moins aussi longtemps |
   | `Homing(f)` | somme plafonnée à 1 | le guidage se renforce sans dépasser le virage complet |
   | `Gravity(g)` | somme | |
   | `Spectral`, `Orbit`, `Split` (contrats, point 6) | présence (`Split` : addition des branches) | |

   Ordre déterministe : modificateurs de l'arme, puis ceux des passifs dans l'ordre de ramassage, puis de
   l'actif, puis des synergies par id ; un passif en double compte deux fois. `resolve_modifiers` existant
   reste tel quel pour l'arme seule (aucun appel ne change) ; le lint garde son refus des doublons **dans
   une même liste** (une arme), pas entre sources.
6. **Nouveaux modificateurs (contrats)**, en fin d'enum `ProjectileModifier` :
   - `Spectral` : traverse les murs et les obstacles (pas les personnages) ;
   - `Split { count, spread, child }` : à l'impact sur un personnage, `count` projectiles `child` (id de la
     table de l'arme, ou projectile par défaut du porteur si absent : Q3) en éventail ;
   - `Orbit { radius, frames_per_turn }` : tourne autour du tireur au lieu d'avancer.
   Le lint les accepte dans le RON (formes, plages : `count > 0`, `radius > 0`) ; leur exécution est T1 :
   d'ici là, une arme qui les porte est refusée par le lint (`Unsupported`, comme les contrats de M1).
7. **Kinds** : déclarer les nouveaux `projectile_modifier` dans `CombatPlugin` (`register_kinds`), et
   ajouter au passage `Fan`, `Spiral`, `Burst` qui manquent aux `pattern` déclarés (constat de lecture).
8. **Bout en bout minimal (testbed)** : un personnage d'essai `larme` (`Infinite`, `Automatic`) et un objet
   `lentille` (`ShotSpeed Pct 0.5`, `ShotSize Mul 1.5`). Scénarios : `tears_infinite` (300 frames de tir sans
   rechargement, `Ammo` constant), `tears_shot_speed` (la balle atteint la cible plus tôt avec l'objet :
   `HitsAtLeast` à une frame où, sans l'objet, elle n'est pas encore arrivée). Unitaires de `compose` : chaque
   règle, l'ordre des sources, l'identité sans ajout (`compose(base, []) == resolve_modifiers(base)`).

## Règles

- Variantes en fin d'enum (`MagBulletConfig`, `StatId`, `ProjectileModifier`) ; attention au piège de
  `#[derive(Hash)]` sur un enum (`CLAUDE.md`, checklist : une deuxième variante fait hacher le
  discriminant) : vérifier qu'aucun enum touché n'avait une seule variante.
- Champs optionnels d'`ItemDef`/`SynergyDef` hors hash quand absents (ce sont des définitions, pas des
  composants rollback : le dire au rapport si c'est le cas).
- Pas de `f32` (les règles en `Fixed`, `Homing` plafonné en `Fixed`), `order_iter!`, `BTreeMap`.
- `make check_sim_boundary` : `combat` est en dette D69, ne pas l'aggraver (aucune dépendance de présentation
  nouvelle).

## Hors périmètre

Exécution des modificateurs d'objets au tir, `Spectral`/`Split`/`Orbit` en jeu, statuts `Poison` et `Fear`,
chance d'effet en jeu (tout cela : M4-T1) ; contenu `isaac` (T0f, T11a) ; correction de `ReloadSpeed`
(dette à numéroter par l'orchestrateur).

## Vérification

README §4 complet (`make test_scenarios` sans `BLESS`, `make test_crates`, `make lint` ×4, fmt,
`check-forbidden`, `check-rollback-registration`, `make gen` ×4, `make check_sim_boundary`). Rapport
`docs/taches/rapports/m4-t0a-contrats-larmes.md` (§7 du README), conventions §60, liste des attentes de
`CLAUDE.md` inchangée (aucune attente nouvelle) ou mise à jour si la tâche en ajoute.

## Atteignabilité

Contrat : aucune partie ordinaire ne le rencontre encore. T1 l'exécute, T11a le met dans les pools de
`isaac` (salle de trésor).

## Session cloud

Tâche prévue pour une session Claude Code cloud. Ignorer les consignes `orca` du README (§1). Pas de
Docker : le p2p est rejoué par l'orchestrateur, le dire au rapport. Pousser la branche régulièrement
(commits WIP acceptés ; le conteneur redémarre environ toutes les deux heures). Voie rapide du README §6 :
fusionner `origin/main` et rejouer la suite juste avant le commit `LIVRÉ m4-t0a-contrats-larmes`, qui
contient le rapport.
