# X-6e « Chef en deux phases » — sur le VPS (coordinateur alacod, run `run_9effecfcae39`, sam 21 nov 2026)

Objectif studio O2 au service d'O3. Jalon **S5 dim 13 déc** (« sergent en deux phases »), parti en avance. Simulation : checklist GGRS de `CLAUDE.md` (fait foi). Tu ne merges pas.
Preuve qui fait foi : ligne **X-6e** du § 6 de `docs/taches/plan-1837-cli.md` (`8778a267`). § 127, D223, rapport `docs/taches/rapports/x-6e-chef-deux-phases.md`.

## Le besoin (design, lecture seule)
`/home/debian/workspace/1867_lore/design/archetypes-ennemis.md` (« Prologue — mini-boss : chef de groupe ») et `design/production/sergent-des-gardes.md` (brouillon : télégraphe
des ordres, durée de l'ouverture, nombre et fréquence des renforts **ouverts** → données réglables, valeurs du testbed neutres). Phase 1 : le chef commande un mur de soldats qui
tirent des **salves coordonnées** à son ordre (geste/cri télégraphié) ; on tue les soldats pour l'atteindre ; aucun renfort ; il a son arme (cône de mitraille). À la **moitié de
sa vie** : geste de colère, il **lâche son arme** et en prend une autre (paire de pistolets) ; phase 2 : **ruée**, **décharge** de plusieurs projectiles à courte portée calée
sur les départs du clip (`moments`), **coup au contact**, **ouverture** après la décharge (rechargement) pour la hache, **renforts** de temps en temps.
**Rien de 1837 dans l'engine** : noms neutres (chef, ordre, phase), personnages de testbed ; décris au rapport ce que 1837 posera en données (sergent, tromblon, pistolets, poses).

## Ordre
1. Inventaire de l'existant avant d'écrire : phases de boss (§ 37, `BossPhase`, attente `BossPhase`), ruée `Charge` (§ 119), tireur posté et cycle (`RangedAttackState`, § 110/122),
   `moments` (§ 116), apparitions/invocations (vagues, `spawn`, M3), changement d'arme d'un ennemi, `ModelPlayback`/poses (§ 122), attentes T-3b (§ 123), Grab (X-6d, branche
   `wquintal/x-6d-agrippeur`, en cours : ne touche pas ses fichiers sans nécessité, signale tout conflit prévisible par `status`). Réutilise ; pas de second mécanisme.
2. Scénarios et attentes d'abord (rouges), puis les blocs. Liste minimale (preuve du § 6) : salve à l'ordre (les soldats tirent ensemble) ; tuer les soldats ouvre l'accès ;
   passage de phase au seuil ; changement d'arme ; ruée et décharge ; ouverture exploitable à la mêlée ; renforts à la fréquence donnée ; aucun renfort en phase 1 ; coop.
   Chaque attente nouvelle **échoue sur une régression provoquée** (journal).
3. Vidéo des scénarios principaux (`make videos SCENARIO=…`, `xvfb-run -a`) ; un scénario court `rapide` si le budget de `check_integration` le permet.
4. `make check_sim_boundary`, `make schedule_order_diff BASE=origin/main`, contrôle rapide découpé (sur mon feu vert), rapport, ligne LIVRÉ.

## Où, règles
- Worktree `/home/debian/orca/workspaces/alacod/x-6e-chef`, branche `wquintal/x-6e-chef`. **Ta cible** : `CARGO_TARGET_DIR=/home/debian/orca/workspaces/alacod/target-p1e`
  (chaude, laissée par P-1e). L'autre worker engine (X-6d) compile dans `target-partage` : n'y touche pas.
- Préfixe `env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1`. Un seul cargo ; jamais de profil debug ; cargo seulement si `df -h /` ≥ 20 Go libres, arrêt de
  TON cargo sous 15 Go et `status` ; `scripts/target-sweep.sh $CARGO_TARGET_DIR` après chaque cargo  ;
  rien de lourd entre 03:45 et 05:00 UTC (suite de nuit).
- Deux contrôles rapides ne tiennent pas ensemble sur le disque : **avant ton contrôle rapide découpé, `ask` « contrôle rapide ? »** et attends mon feu vert. Scripts :
  `~/orca/runs/alpha/logs/p-1d-check-rapide-decoupe.sh` et `p-1d-test-crates-decoupe.sh` sous ton nom (journal `logs/x-6e-chef-check-rapide-<sha>.log`), puis purge des
  binaires de tests d'intégration et sweep.
- Aucune trace existante modifiée ; ne `BLESS` que les traces neuves de ton lot. Lis `CLAUDE.md`, `~/orca/runs/alpha/commun.md` (écrit pour le Mac ; ce fichier-ci prime).

## Communication
Commite et pousse à chaque étape. Carte `orca worktree set --worktree active --comment …`. `status` à chaque étape (rouges, vert, E prêt) et au moins une fois par jour ;
aperçu à mi-parcours (voie de plus de deux jours) : une vidéo ou un journal de scénario. 429 : `status` tout de suite. Question bloquante : `ask` avec ton choix par défaut,
et avance. Fin : rapport, LIVRÉ, aucun processus, un seul `worker_done`.
