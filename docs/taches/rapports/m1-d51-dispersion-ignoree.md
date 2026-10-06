# Rapport — m1-d51-dispersion-ignoree (`FiringModeConfig.spread` jamais appliqué, D51)

Branche partie de `origin/main` f198354.

## État en cours

SUITE_D51

## Correctif

- `crates/combat/src/weapons/mod.rs` : la branche « tir simple » (tout mode sauf `Shotgun`)
  calcule la direction par `single_shot_direction(aim_dir, random, weapon_config.spread)` :
  angle `(random − ½) × spread`, donc dans `[−spread/2, spread/2]` ; avant, `(random − ½) × 1`
  (±0,5 rad pour toute arme). `spread` = 0 rend `aim_dir` tel quel (`FixedMat2::from_angle(0)`
  n'est pas l'identité exacte en `Fixed` : cos ≈ 1,00002). Un seul tirage RNG par balle (flux
  `weapons`), tiré avant comme avant : les flux ne bougent pas, seule la direction change.
- Tests : `weapons::tests::dispersion_nulle_tir_exact` (spread 0 : direction visée exacte, quel
  que soit le tirage), `dispersion_dans_la_demi_largeur` (angle dans `[−s/2, s/2]`, borne
  `−s/2` au tirage 0).
- Lint : `0 <= spread <= π` par mode (`lint_weapons`, champ `spread` lu dans le schéma miroir,
  0 si absent) ; fixture `weapon_spread_out_of_range` (spread 4).
- Doc : conventions §16 (unité radians, pleine largeur, formule, état avant D51), §29 (valeurs
  des armes de `throne` appliquées seulement depuis D51).
- Patterns ennemis (§20) : leur dispersion est bien appliquée (`emitter.rs`, `fan_angles` de
  `projectile.rs` multiplient par `spread`) ; pas de dette.

### Contenu : aucune conversion

Toutes les valeurs de `spread` des modes de tir sont en radians, cohérentes avec le seul
champ appliqué jusque-là (`spread_angle` du fusil : 0,4) ; aucune n'est > π :

- `throne` : mitraillette 0,15, disque 0,05, revolver 0,0005, les autres 0.
- `zombies` et `testbed` (`ZombieShooter/Sprites/Character/weapons.ron`) : mitraillette 0,15,
  son mode `rafale` 1, une arme à 0,01, les autres 0. `rafale` à 1 = ±0,5 rad, exactement
  l'ancien comportement : l'auteur l'a probablement calé sur ce qu'il voyait (le bug) ; il ne
  change pas.

## Mesure avant/après

« Avant » = `main` ddfc190 (code identique à f198354), « après » = cette branche ; mêmes
commandes, une sim à la fois.

| | avant | après |
|---|---|---|
| zombies `avant_poste`, 4 `acheteur`, 20 graines, vague 5 atteinte | 20/20 | 20/20 |
| zombies : frames moyennes à la vague 5 | 7078 | 7031 |
| zombies : kills (total) / morts / desync | 1040 / 0 / 0 | 1049 / 0 / 0 |
| throne 2 bots, témoins 1..20 : étage 3 | 16/20 | **20/20** |
| throne 2 bots : défaites | 4 (11, 12, 17, 20) | **0** |
| throne 2 bots : soft-locks / desync | 0 / 0 | 0 / 0 |
| throne 4 bots, témoins 1..20 : étage 3 / défaites / desync | 20/20 / 0 / 0 | 20/20 / 0 / 0 |

Balles perdues au troisième étage (méthode du digest de b0 : trace détaillée, un rejeu à la
fois, dump supprimé ; dégâts infligés / (munitions consommées × 8), mitraillette seule dans les
deux cas) :

| graine | avant | après |
|---|---|---|
| 25 | 66 % (défaite, étage 3 à f2819) | 62 % (étage 3 franchi, f3257) |
| 103 | 77 % (fin f5119) | 56 % (fin f3271) |
| 131 | 81 % (fin f5786) | 54 % (fin f2848) |

**À noter (décision 5 de la fiche, sans rééquilibrage)** : sur `throne`, les défaites des
témoins à 2 bots tombent de 4 à **0** et le troisième étage se finit bien plus vite ; sur
`zombies`, presque aucun effet (les acheteurs tirent surtout à la mitraillette, 0,15). La
mitraillette perd encore environ la moitié de ses balles (sa propre dispersion et les cibles
mobiles) : c'est le terrain de m1-v3-bots-armes (score de précision sur la vraie dispersion).
Le réglage des `spread` et des dégâts reste une tâche de contenu à part.

## Traces

TRACES_D51
