# m1-d51-dispersion-ignoree — `FiringModeConfig.spread` n'est jamais appliqué (D51, bug moteur)

Lire d'abord `docs/taches/README.md` (agent **local**, worktree de b1, branche
`m1-d51-dispersion-ignoree` créée depuis `origin/main`, même target que m1-v3-bots-armes qui
attend). Voie V1a combat, ½ j de code, le reste en mesure. Compilation après le **feu vert** d'orch,
`CARGO_BUILD_JOBS=2`, un dump à la fois.

## Constat (b1, 2026-10-05, vérifié par l'orchestrateur)

`crates/combat/src/weapons/mod.rs`, branche « tir simple » (toutes les armes sauf le fusil à
pompe, ~l. 1404) :

```rust
let offset_from_center = random_fixed_val.saturating_sub(fixed_math::FIXED_HALF);
let pellet_angle_fixed = offset_from_center.saturating_mul(fixed_math::FIXED_ONE);
```

L'angle de chaque balle vaut `(rand − 0,5) × 1` : **toutes les armes simples dispersent à ±0,5 rad
(±29°)**, quel que soit `spread` de leur `FiringModeConfig` (lu nulle part hors `Hash`/`Debug`).
Revolver (0,0005), laser et lance-lames (0), disque (0,05) tirent donc comme la mitraillette
(0,15). À 300 px, ±0,5 rad couvre ±160 px : ~8 % des balles touchent un ennemi de rayon 12. C'est
l'essentiel des 66 à 81 % de balles perdues mesurées sur les défaites de `throne`. Seul le fusil à
pompe applique son `spread_angle`.

## Décisions (fixées ici)

1. **Correctif** : `offset_from_center.saturating_mul(weapon_mode_config.spread)` (le champ de la
   config du mode de tir en cours), rien d'autre dans la simulation. `spread` = 0 → tir exactement
   dans la direction visée. Test unitaire : avec `spread` 0 la direction est `aim_dir` ; avec
   `spread` s l'angle est dans [−s/2, s/2] ; le nombre de tirages RNG est inchangé (un par balle :
   les flux `RngStreams` ne bougent pas, seule la direction change).
2. **Lint** : `spread` ≥ 0 et ≤ π, fixture ; doc du champ dans conventions §16 (projectiles) : unité
   radians, demi-largeur = `spread / 2`, et une ligne au §29 (armes de `throne`) rappelant que les
   valeurs de contenu n'avaient jamais été appliquées avant D51.
3. **Traces** : **toutes** celles où une arme simple tire bougent, zombies (`clone_*`,
   `equilibrage_*`, `avant_poste_demo`, `bots_four_mixed`…), testbed et throne compris. Preuve
   du §10 par **catégorie** : pour trois scénarios (un par jeu), la première ligne qui diffère est
   la frame du premier tir simple (relevé `BulletCount`/trace), et un scénario sans tir simple
   (par exemple un scénario de surfaces, de statut par contact, ou `explode_wall`) reste identique :
   liste exacte des traces inchangées. Rien de béni par l'agent.
4. **Mesure d'équilibrage avant/après**, même binaire de base `main`, dans le rapport :
   - zombies : 20 graines `avant_poste`, 4 `acheteur`, `--until-wave 5` (vague 5, frames, kills,
     morts) ; **attendu** : 20/20 toujours, frames en baisse ;
   - throne : témoins 1..20 à 2 et 4 bots (étage 3, défaites, soft-locks, desync) ; graines 103,
     25, 131 : balles perdues au 3e étage (méthode du digest, un dump à la fois) ;
   - `bench_bullets`/`bench_horde` inchangés dans leur budget (le correctif ne coûte rien).
   Sims **une à la fois** tant que b0 joue `test_map` (mémoire : deux runs tués le 2026-10-05).
5. **Pas de rééquilibrage de contenu** dans cette tâche : si les armes deviennent trop fortes
   (zombies vague 5 beaucoup plus tôt, throne défaites proches de 0), le noter avec les chiffres ;
   le réglage des `spread` et des dégâts est une tâche de contenu à part (après D50 et l'arroseur),
   décidée avec William.
6. **Hors périmètre** : le fusil à pompe (déjà correct), les patterns ennemis (`Pattern` a sa
   propre dispersion, §20 : vérifier en une phrase qu'elle est bien appliquée, sinon dette), les
   bots (m1-v3-bots-armes reprend après le merge).

## Livrer

Suite complète (liste des traces qui bougent et de celles qui ne bougent pas), crates, `make
lint` des trois jeux, fmt, scripts, `make gen` ×3 (les scénarios générés bougent aussi : dire
combien), exemples ; rapport `docs/taches/rapports/m1-d51-dispersion-ignoree.md` avec la mesure de la
décision 4 ; purge du target. Merger `origin/main` juste avant. `git push -u origin
m1-d51-dispersion-ignoree`, puis `SendMessage` à `orch` : `LIVRÉ m1-d51-dispersion-ignoree <sha> :
<traces qui bougent / inchangées, zombies avant → après, throne avant → après>`. Ne merge pas, ne
bénis pas.
