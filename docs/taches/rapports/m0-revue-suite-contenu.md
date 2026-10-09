# m0-revue-suite-contenu — S5, option B

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
- William a choisi B : `base_enemies=3`, `enemies_per_wave=2`, plafond 4 vivants,
  intervalle 120 frames ; mesure après terminée (20/20 V5) ; 186 scénarios verts après bless. Aucune intervention dans M2.
- D29 confirmé : les multiplicateurs de santé **et** de dégâts des vagues sont calculés
  et tracés, mais ne sont pas lus pour appliquer la santé ou les attaques des zombies.
- Mesure avant S5 : terminée, quatre acheteurs, graines 1..20, carte `avant_poste`,
  arrêt à l'entrée en vague 5, plafond 20 000 frames ; quatre lots indépendants de cinq.
- Partie solo enregistrée (5277 frames), retour de ressenti de William attendu.
  P2p et bench : non vérifiés ici ; pas de livraison distante demandée à ce stade.

Commande de chaque lot (bornes inclusives) :

```sh
APP_VERSION=x target/headless/alacod-sim --game zombies --bots 4 \
  --profiles acheteur,acheteur,acheteur,acheteur --seeds 1..5 \
  --until-wave 5 --max-frames 20000 --json target/metrics/m0-revue-suite-contenu/s5-avant-lot1.json
```

Autres lots : `6..10`, `11..15`, `16..20`. Les durées CPU sous concurrence ne constituent
pas un benchmark. Les dégâts sont la somme des baisses de santé observées, comme le relevé
global du runner (arrondi par vague), pas une somme brute des événements de dégâts.

Propositions (B choisie et appliquée ; variance conservée à 0..2, lots d'un zombie, types inchangés) :

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


## S5 — option B choisie par William

Changement de gameplay voulu : `base_enemies` 6 → 3, `enemies_per_wave` 4 → 2,
`max_concurrent_enemies` 8 → 4, `spawn_interval_frames` 60 → 120. Aucune autre donnée
jouée n'est changée : variance, types, santé, vitesse et pauses conservés.

Même protocole qu'avant, 20 graines : 20/20 V5, 0 mort, dégât, mise à terre, desync,
soft-lock ou failure ; les quatre processus terminent avec le code 0. Entrée V5 moyenne
6422,8 → 6454,05 frames, plage après 6108–7136. Kills moyens 51,7 → 28,4 (−45,1 %) ;
portes ouvertes moyennes 11,4 → 9,4. La cadence plus lente compense la diminution du
nombre d'ennemis en durée ; ces chiffres ne prouvent pas le ressenti solo. William doit
encore jouer B. La V5 est atteinte, son combat n'est pas mesuré.

Le code final du relevé est utilisé ici : les timestamps n'ont pas été corrigés après
coup. Vérification : préparation 180 frames pour toutes les V1–V4, sommes de dégâts et
de morts cohérentes avec le runner. JSON `s5-apres.json`, lots bruts et `comparer.py`
dans `target/metrics/m0-revue-suite-contenu/`.

| Vague | Avant : frames (s) | B : frames (s) | B : min–max frames | Morts avant / B | Dégâts avant / B |
|---|---|---|---|---|---|
| 1 | 456.1 (7.60 s) | 472.1 (7.87 s) | 294–666 | 0 / 0 | 0 / 0 |
| 2 | 688.4 (11.47 s) | 758.1 (12.63 s) | 560–833 | 0 / 0 | 0 / 0 |
| 3 | 996.0 (16.60 s) | 994.6 (16.58 s) | 784–1561 | 0 / 0 | 0 / 0 |
| 4 | 1161.2 (19.35 s) | 1108.2 (18.47 s) | 991–1319 | 0 / 0 | 0 / 0 |
| 5 | Arrêt à l’entrée | Arrêt à l’entrée | — | 0 / 0 | 0 / 0 |

| Graine | B : V1 combat | V2 | V3 | V4 | Entrée V5 | Kills | Portes | Dégâts | Morts |
|---|---|---|---|---|---|---|---|---|---|
| 1 | 354 | 784 | 959 | 1014 | 6232 | 27 | 8 | 0 | 0 |
| 2 | 325 | 822 | 1026 | 1064 | 6358 | 28 | 8 | 0 | 0 |
| 3 | 462 | 799 | 916 | 1116 | 6414 | 29 | 10 | 0 | 0 |
| 4 | 342 | 770 | 1061 | 1194 | 6488 | 29 | 10 | 0 | 0 |
| 5 | 564 | 792 | 950 | 1025 | 6452 | 28 | 10 | 0 | 0 |
| 6 | 454 | 819 | 891 | 991 | 6276 | 28 | 10 | 0 | 0 |
| 7 | 666 | 774 | 1561 | 1014 | 7136 | 30 | 10 | 0 | 0 |
| 8 | 325 | 690 | 784 | 1319 | 6239 | 26 | 10 | 0 | 0 |
| 9 | 462 | 826 | 951 | 1067 | 6427 | 28 | 6 | 0 | 0 |
| 10 | 435 | 802 | 893 | 1268 | 6519 | 30 | 10 | 0 | 0 |
| 11 | 538 | 677 | 918 | 1265 | 6519 | 30 | 6 | 0 | 0 |
| 12 | 480 | 833 | 1055 | 1047 | 6536 | 29 | 12 | 0 | 0 |
| 13 | 589 | 716 | 908 | 1046 | 6380 | 28 | 8 | 0 | 0 |
| 14 | 621 | 560 | 1406 | 1020 | 6728 | 27 | 8 | 0 | 0 |
| 15 | 294 | 808 | 834 | 1051 | 6108 | 26 | 8 | 0 | 0 |
| 16 | 467 | 798 | 800 | 1017 | 6203 | 27 | 10 | 0 | 0 |
| 17 | 531 | 662 | 943 | 1104 | 6361 | 29 | 12 | 0 | 0 |
| 18 | 599 | 819 | 1045 | 1178 | 6762 | 31 | 12 | 0 | 0 |
| 19 | 344 | 581 | 1076 | 1124 | 6246 | 27 | 8 | 0 | 0 |
| 20 | 589 | 830 | 916 | 1241 | 6697 | 31 | 12 | 0 | 0 |


## Preuve des traces contre main

Référence : checkout détaché `../alacod-main-s5` de `origin/main` (`ae87ae0`), target
partagé, binaire de test compilé depuis ce checkout. Dumps un scénario par appel :
`idle`, `two_players_shooting`, `points_on_kill`, `downed_all_lose`. Même compilation
et mêmes dumps côté branche ; `scripts/trace-diff.py` compare les états détaillés.

- Avant isolation de la scène historique de `idle`, `idle` et `two_players_shooting` :
  première différence f0, uniquement `WaveState`
  (7 ennemis préparés → 4 avec la même variance 1 : effet de `base_enemies` 6 → 3).
- `two_players_shooting --ignore WaveState` : tout identique jusqu'à f239 ; première
  différence f240, deuxième zombie et son arme présents sur main, pas encore sur B,
  avec les avancées du flux RNG `waves` et du compteur d'ids correspondantes.
  Conséquence exacte de l'intervalle 60 → 120, pas de modification du moteur.
- Les baisses de nombre modifient ensuite les apparitions, kills, points, RNG et les
  transitions ; le plafond de 4 espace les apparitions quand quatre zombies vivent.
- Les tests de mécanique `points_on_kill`, `remote_first_fight`, `downed_all_lose`,
  `two_players_idle`, `run_lose_summary`, `idle`, `shoot_around`, `window_repair`
  reposent sur une visée enregistrée ou une
  mise à terre à une frame fixée. Leur scène historique est maintenant explicite :
  `wave_overrides` rétablit les quatre valeurs anciennes, **assertions conservées**.
  Les vagues de partie réelle et les 20 acheteurs utilisent B.
- Rejeu final avec ces fixtures : `points_on_kill` identique à main sur **599 frames**,
  `downed_all_lose` identique sur **1699 frames**, aucune exclusion au trace-diff,
  attentes/invariants/synctest verts (codes 0). Ces deux traces ne nécessitent pas de bless.

Lint : `APP_VERSION=x make lint`, code 0 ; zombies, testbed et throne sans erreur.
Vérification complète avec bless : 186 scénarios verts, code 0 (voir bilan ci-dessous). Aucun changement de simulation
Rust ; seule la config S5 et les conditions explicites de huit tests sont modifiées.


Partie solo B lancée pour William via le binaire avec rendu déjà compilé,
`ALACOD_RECORD=…/revue_s5_b_2026-10-09.ron`. Capture écrite à la fermeture, 5277 frames.
V1 ≈ 14,4 s (861 frames), V2 ≈ 22,7 s (1359 frames), V3 commencée ; transitions du
log, resimulations dédupliquées. Ce ne sont pas des parties à inputs identiques à la
revue antérieure (~22 / ~30 s). William valide ensuite B le 2026-10-09 : « ça marche très bien, plus équilibré ».


Premier passage complet : 186 scénarios joués ; code 101 (38 références de traces
modifiées, toutes dès la ligne 1, et 10 attentes de mécanique à calendrier historique
en échec dans `downed_all_lose`, `idle`, `shoot_around`, `window_repair`). Aucune erreur
RON, aucun invariant ni desync signalé. Les huit fixtures historiques ci-dessus
explicitent maintenant les quatre paramètres ; **aucune assertion affaiblie ni retirée**.
`idle`, `shoot_around`, `window_repair` rejoués séparément après cette isolation :
attentes, invariants, synctest et comparaison aux anciennes traces passent, codes 0.
`points_on_kill` et `downed_all_lose` avaient déjà passé avec trace détaillée identique
à main. Le bless complet qui suit doit conserver les huit références historiques et
ne mettre à jour que les cas héritant des paramètres S5.

Formatage et contrôles statiques repassés : fmt OK, forbidden 4 avertissements
préexistants, rollback registration OK. Dumps temporaires supprimés après preuve
(10 fichiers, 5,67 Gio) ; checkout de référence retiré, logs de diff conservés.
L'enregistrement solo garde explicitement les quatre valeurs B dans `wave_overrides`
(inputs inchangés), pour conserver les vagues de cet essai lors d'un futur réglage.


## Bilan de vérification S5

`BLESS=1 make test_scenarios TEST_VERSION=x` : **code 0**, 186 scénarios avec attentes,
invariants et synctest verts ; 3 tests Rust réussis, 0 échec, 7 ignorés ; 780,83 s.
`TEST_VERSION=x` conserve la version de compilation utilisée pour les mesures et les
preuves (`APP_VERSION=x`), sans recompilation inutile ; ce champ est de présentation.

**34 traces modifiées, 152 inchangées.** La liste correspond exactement aux 38 différences
initiales, moins les quatre scènes (`idle`, `shoot_around`, `window_repair`,
`downed_all_lose`) qui avaient encore leurs valeurs implicites lors du diagnostic.
Les trois clones et les huit scènes historiques gardent leurs traces. Tous les tests
de testbed et throne restent inchangés. Les références des six ennemis zombies générés
changent car ces scénarios héritent aussi de la préparation de vagues du jeu.
Les scènes de mécanique ont été explicitement isolées, pas affaiblies : mêmes inputs,
frames et assertions. La vraie partie et la mesure à quatre acheteurs utilisent B.

Les 34 fichiers de traces sont consignés dans le commit de bless séparé du réglage,
avec justification « changement de gameplay voulu : S5 B » et cette preuve.

### Non fait / restant

- Ressenti de William : B validée le 2026-10-09 après une nouvelle partie (« plus équilibré »).
- S4, S6, S7, S2/S8, R10 et décisions R2/R3/R6/R8 : non traités dans cette passe S5.
- Livraison : tests complets des dix crates et génération des trois jeux non relancés
  ici (aucune arme modifiée) ; p2p et bench strict au calme non vérifiés ici, conformément
  au prompt. Aucun push de livraison.
- Dette D29 conservée ; pas de modification du moteur de M2, du RNG ou des dégâts.


Comparaison finale aux nouvelles références (sans bless) : `two_players_shooting`,
`bots_four_mixed`, `enemy_zombie_full_moving` passés, codes 0 ; attentes, invariants,
synctest et hashes comparés aux références vertes. Commande par cas :
`ALACOD_SCENARIO=<cas> APP_VERSION=x cargo test -p scenario --profile headless --test scenarios scenarios -- --nocapture`.
Formatage final et `git diff --check` : OK. Liste des 34 références contrôlée exactement
contre l'inventaire des différences, en excluant les quatre scènes isolées après
leur premier passage. Aucun fichier de code ni de contenu supplémentaire dans le bless.

## S2/S8 — Le Relais, bâtiment à défendre (2026-10-09)

Direction approuvée par William : petit bâtiment avec zombies à l’extérieur,
fenêtres barricadées et portes intérieures. `maps/le_relais_prototype.ldtk`
contient quatre orientations d’un bâtiment de cinq pièces, huit fenêtres,
huit spawners extérieurs et six portes. Toutes les pièces et la cour sont
dans un seul niveau LDtk par partie ; l’assemblage variable des pièces, les
objectifs interactifs et le renouvellement de la graine restent à construire.
Script reproductible : `scripts/construire-le-relais.py`.

Le premier prototype était bloqué par des spawns derrière des portes fermées.
Le nouveau laisse les zombies circuler dehors jusqu’aux fenêtres de l’accueil,
même portes fermées. Aucun code moteur changé. Vérifications effectuées :
lint zombies, dégagement des départs, flood-fill de corps 20 × 20 px sur grille
8 px, génération sur 20 graines (quatre plans, cinq occurrences chacun), puis
20 parties de quatre acheteurs jusqu’à l’entrée en V5 : **20/20**, 0 mort,
0 mise à terre, 0 desync, 0 failure d’invariant, 134 dégâts cumulés. Médiane
V5 f6940 ; graine 11 f18713 après une V3 de 12754 frames, à revoir humainement
pour sa stagnation prolongée. Aucun arrêt pour soft-lock rapporté.

Relevés par graine et limites : `docs/captures/le-relais/README.md`. Assets
existants, textures provisoires ; registre des assets mis à jour. Manifeste
conservé sur avant_poste. Pas de bless ni de nouvelle suite historique complète
pour cette passe. Pas de push ; validation humaine du bâtiment encore attendue.
