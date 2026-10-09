# m0-revue-suite-contenu — préparation S5

Session de William, 2026-10-09 ; base `ae87ae0`, branche `m0-revue-suite-contenu`.

- §0 : branche existante conservée, `git fetch origin` effectué ; compilation de `zombies`
  avec rendu et profil `headless` réussie (4 min 39 s, compilation groupée avec `alacod-sim`).
- §1 : prompt, `CLAUDE.md`, README des tâches, carnet, dettes et conventions utiles lus.
- Instrumentation : `alacod-sim` rapporte, hors simulation dans `Last`, la préparation,
  le début des apparitions, la fin, les baisses de santé, les mises à terre et les morts
  par vague. Une frame déjà observée est ignorée.
- Compilation séparée d'`alacod-sim` sans la feature de rendu des tilemaps : réussie,
  59,14 s. Test du binaire : 1 réussi, 0 échec.
- `cargo fmt --all -- --check` : réussi ; contrôles forbidden : 4 avertissements
  préexistants ; contrôle rollback registration : OK.
- Aucun réglage de gameplay appliqué, aucune trace bénie, aucune intervention dans M2.
- D29 confirmé : les multiplicateurs de santé **et** de dégâts des vagues sont calculés
  et tracés, mais ne sont pas lus pour appliquer la santé ou les attaques des zombies.
- Mesure avant S5 : terminée, quatre acheteurs, graines 1..20, carte `avant_poste`,
  arrêt à l'entrée en vague 5, plafond 20 000 frames ; quatre lots indépendants de cinq.
- Prochaine étape : choix chiffré par William, puis mesure après
  et partie solo. Suite complète, p2p et bench : non vérifiés ici à ce stade.

Commande de chaque lot (bornes inclusives) :

```sh
APP_VERSION=x target/headless/alacod-sim --game zombies --bots 4 \
  --profiles acheteur,acheteur,acheteur,acheteur --seeds 1..5 \
  --until-wave 5 --max-frames 20000 --json target/metrics/m0-revue-suite-contenu/s5-avant-lot1.json
```

Autres lots : `6..10`, `11..15`, `16..20`. Les durées CPU sous concurrence ne constituent
pas un benchmark. Les dégâts sont la somme des baisses de santé observées, comme le relevé
global du runner (arrondi par vague), pas une somme brute des événements de dégâts.

Propositions, non appliquées (variance conservée à 0..2, lots d'un zombie, types inchangés) :

| Réglage | Actuel | A : allègement modéré | B : départ doux, recommandé |
|---|---|---|---|
| `base_enemies` | 6 | 4 | 3 |
| `enemies_per_wave` | 4 | 3 | 2 |
| V1 | 6–8 | 4–6 | 3–5 |
| V2 | 10–12 | 7–9 | 5–7 |
| V3 | 14–16 | 10–12 | 7–9 |
| V4 | 18–20 | 13–15 | 9–11 |
| V5 | 22–24 | 16–18 | 11–13 |
| `spawn_interval_frames` | 60 (1 s) | 90 (1,5 s) | 120 (2 s) |
| `max_concurrent_enemies` | 8 | 6 | 4 |

Préparation : 180 frames ; délai après le dernier kill : 600 frames, suivies de la
préparation (environ 13 s au total). Ces deux valeurs restent inchangées dans A et B.
Les zombies des vagues 1–3 gardent 50 PV et la répartition 80 % `zombie_full` (80 px/s),
20 % `zombie_1` (120 px/s). Les multiplicateurs D29 restent inchangés, sans leur prêter
d'effet. Les deux options affectent aussi les vagues ultérieures et tous les nombres de
joueurs ; moins de kills peut retarder les achats. Il faudra mesurer la variante choisie,
puis la faire jouer en solo avant de valider S5.


## Mesure de référence S5

20/20 graines atteignent la vague 5 ; 0 mort, 0 mise à terre, 0 dégât encaissé,
0 désynchronisation, 0 soft-lock, 0 failure d'attente/invariant. Les quatre processus
se sont terminés avec le code 0. Entrée V5 entre les frames 6193 et 6847.
Les quatre acheteurs n'indiquent donc pas la difficulté ressentie en solo ; la revue
humaine (V1 ~22 s, V2 ~30 s) reste le motif de S5.

Contrôle de l'observateur : graine 1 contre le binaire construit depuis le fichier
`alacod-sim.rs` de la base `ae87ae0` (source temporaire retirée) : toutes les métriques
de jeu JSON identiques, en excluant `waves` et les mesures de temps CPU. Frame 6800,
50 kills, 12 portes ouvertes ; santé/décès et état final des quatre joueurs identiques.
Ce contrôle compare les métriques, pas une preuve exhaustive d'identité des traces.

Horloge : `start`, `spawning`, `cleared` utilisent le compteur du runner **après update**.
Les premiers lots lisaient `WaveState.wave_start_frame` pour `spawning`, soit la frame
simulée (une unité avant le compteur après update). Le JSON consolidé a été corrigé
explicitement par `spawning += 1` ; les quatre JSON bruts restent conservés. Le code
final lit directement `FrameCount`, comme les deux autres timestamps. Pour chaque
graine et chaque vague 1–4 : `spawning - start = 180` vérifié. Les sommes par vague
correspondent aux relevés globaux de dégâts et de morts.

Résultats bruts et consolidés : `target/metrics/m0-revue-suite-contenu/` ; script de
consolidation `resumer.py`, référence sans sonde `s5-reference-seed1.json`, résultat
consolidé `s5-avant.json`. Les tableaux ci-dessous gardent les chiffres dans le dépôt.

Combat = `cleared - spawning` ; ajouter 180 frames (3 s) pour compter depuis le
changement de numéro de vague, préparation comprise. La V5 est seulement atteinte :
son combat n'a pas été joué. Dégâts cumulés sur les quatre joueurs et les vingt graines.

| Vague | Combat : frames moyennes (min–max) | Secondes moyennes | Dégâts cumulés (20 graines) | À terre | Morts |
|---|---|---|---|---|---|
| 1 | 456.1 (351–546) | 7.60 | 0 | 0 | 0 |
| 2 | 688.4 (564–1149) | 11.47 | 0 | 0 | 0 |
| 3 | 996.0 (848–1547) | 16.60 | 0 | 0 | 0 |
| 4 | 1161.2 (1065–1277) | 19.35 | 0 | 0 | 0 |
| 5 | Arrêt à l’entrée ; combat non mesuré | — | 0 | 0 | 0 |

| Graine | V1 combat | V2 combat | V3 combat | V4 combat | Entrée V5 | Dégâts | Morts |
|---|---|---|---|---|---|---|---|
| 1 | 399 | 580 | 1547 | 1153 | 6800 | 0 | 0 |
| 2 | 383 | 668 | 859 | 1277 | 6308 | 0 | 0 |
| 3 | 440 | 674 | 945 | 1065 | 6245 | 0 | 0 |
| 4 | 395 | 693 | 863 | 1150 | 6222 | 0 | 0 |
| 5 | 422 | 775 | 930 | 1243 | 6491 | 0 | 0 |
| 6 | 451 | 624 | 917 | 1172 | 6285 | 0 | 0 |
| 7 | 506 | 715 | 911 | 1159 | 6412 | 0 | 0 |
| 8 | 402 | 564 | 1533 | 1197 | 6817 | 0 | 0 |
| 9 | 475 | 673 | 848 | 1076 | 6193 | 0 | 0 |
| 10 | 419 | 778 | 1014 | 1121 | 6453 | 0 | 0 |
| 11 | 521 | 629 | 1005 | 1238 | 6514 | 0 | 0 |
| 12 | 504 | 630 | 934 | 1165 | 6354 | 0 | 0 |
| 13 | 507 | 786 | 928 | 1123 | 6465 | 0 | 0 |
| 14 | 536 | 1149 | 913 | 1128 | 6847 | 0 | 0 |
| 15 | 351 | 646 | 867 | 1248 | 6233 | 0 | 0 |
| 16 | 454 | 604 | 1144 | 1134 | 6457 | 0 | 0 |
| 17 | 521 | 607 | 966 | 1096 | 6311 | 0 | 0 |
| 18 | 531 | 665 | 875 | 1074 | 6266 | 0 | 0 |
| 19 | 360 | 705 | 1034 | 1254 | 6474 | 0 | 0 |
| 20 | 546 | 603 | 888 | 1151 | 6309 | 0 | 0 |

Validation finale de l'horloge : recompilation réussie ; rejeu graine 1 à quatre acheteurs
jusqu'à V2, code 0, sans failure/desync ; V1 strictement identique au JSON consolidé
(`start=1`, `spawning=181`, `cleared=580`, combat 399 frames), entrée V2 f1180.
`cargo fmt --all -- --check` et `git diff --check` passent après la correction.
