# polissage-throne-jouable — D66/D67 et campagne chiffrée de throne (S2-7)

Branche `polissage-throne-jouable`, base `00dc83ce`, VPS ovh-inner.

## D66 / D67

- **Fait / vérifié, aucun code ajouté** : tout était déjà livré (revue-sons-visuels-m1, dette-d66-vue-directe, M2-T19).
  - D66 : `marker` de `games/throne/assets/characters/tourelle.ron` (teinte `(0.55, 0.75, 1.0)`, échelle 1,25, sans asset) ;
    `visuals::face_stationary_enemies` oriente un stationnaire vers sa cible (présentation, `flip_x` seul).
  - D67 : `pattern_shape` (`crates/game/src/feedback_log.rs`) dessine `Ring` en rayons courts dans les directions de tir,
    test `forme_ring_rayons_courts_dans_les_directions_de_tir` ; `draw_telegraphs` (`feedback.rs:887`) ne trace plus le cercle de portée.
- **Non vérifié (VPS sans rendu, D94)** : l'aspect à l'écran. Dettes renvoyées dans `dettes.md` avec « confirmation visuelle Mac ».
- Sprite propre à la tourelle : seul vrai remède durable, du pack throne (S2-10).

### À filmer sur le Mac (S2-8)

`make videos SCENARIO=enemy_tourelle_still,enemy_roi_rat_still` (scénarios générés throne, 600 frames) : la tourelle (teinte bleutée,
plus grande) tourne vers le joueur ; à chaque salve (120 frames) huit rayons courts autour d'elle, pas de grand cercle rouge.
`make play_scenario SCENARIO=enemy_tourelle_moving` avec `--follow 0` pour comparer avec un zombie qui avance.

## Campagne « avant » (aucun changement de contenu)

Commande (binaire `--profile headless --no-default-features`, commit `00dc83ce`) :
`alacod-sim --game throne --bots N --profiles prudent[,prudent] --floors run --seeds 1..100 --until-floor 3 --max-frames 15000 --json …`
JSON : `polissage-throne-jouable-avant-1bot.json`, `…-avant-2bots.json` (même dossier).

| | 1 bot | 2 bots |
|---|---|---|
| étage 1 franchi | 100 % | 100 % |
| étage 2 franchi | 89 % | 97 % |
| run complète (étage 3 franchi) | 41 % | 86 % |
| morts | 49 (48 à l'étage 3, 1 à l'étage 2) | 1 (étage 3) |
| soft-locks | 10 (étage 2) | 13 (3 à l'étage 2, 10 à l'étage 3) |
| desync | 0 | 0 |
| durée médiane (frames) / max | 4 259 / 6 412 | 4 120 / 5 557 |
| dégâts subis médiane | 123,5 | 69 |

Le JSON ne donne pas de cause de mort plus fine que l'étage : la cause se lit au rejeu (`--save-scenario`, voir `m1-analyse-200-throne.md`).

**Verdict** : aucun étage sous 20 % ; l'étage 1 à 100 % est l'introduction, pas un mur de difficulté ; le palier le plus dur est
l'étage 3 en solo (41 %). Pas de réglage de contenu, donc pas de campagne « après », aucune trace bougée (seuils de travail, William tranche).
Point à décider par William : les soft-locks (10–13 %) sont un défaut du bot ou de la carte, pas de l'équilibrage ; non traités ici.

## make check

Vert (code 0, `env -u CI make check`). Un écart de formatage déjà sur main (`crates/content/tests/lint_fixtures.rs`, format seul) corrigé au passage. Dernières lignes :

```
     |     ^^^^^^^^^^^
games/isaac : aucune erreur (24 personnages, 3 armes, 2 armes de corps à corps, 0 vagues, 3 cartes)
✓ All checks passed
exit=0
```
