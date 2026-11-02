# Polissage des clones — fin de la semaine 1 (28 oct – 1er nov 2026)

Rapport de fin de voie du coordinateur (run d'orchestration `run_b9bd92cceed4`). **État : semaine 1 livrée, sauf la recette de S1-3.**
Consigne de William (dim 1) : on termine ce qui est en cours, rien de nouveau avant sa revue ; les lots suivants sont la matière de la semaine 2.
Chaque merge : `merge --no-ff` dans `polissage-integration`, `make check` vert (Mac ; `test_tools` sur le VPS), aucune trace d'un autre jeu
modifiée, puis `main` en `--ff-only` et poussé. `main` = `polissage-integration` à la date de ce rapport.

## Par tâche

| Tâche | Voie | Issue | Preuve (rapport, commit de merge) |
|---|---|---|---|
| Lot 0 `online` honnête | b2 | **livré** : `releases.json` `online: false` (zombies, throne) jusqu'à la recette S1-3 | journal `LIVRÉ polissage-online-faux` ; `caa001ac` |
| S1-1 manette, menus, aide | b0 | **livré** : manette dans Chrome (sticks, RT à seuil, pause/inventaire, « Touches ») ; 2 min de manette physique par William ; E4 complet **non tenu** | `polissage-manette.md` ; `ebab067a` |
| S1-4 textes en français | b2, b1 | **livré** : `ui/texts.rs`, salon/réseau/pause/fin de partie traduits ; police posée (accents), 13 + 13 captures avant/après | `polissage-textes-fr.md` ; `a9c41166`, `412d438c`, `a2ade254` |
| S1-5 perf wasm | b0 | **livré** : compteur FPS (`?perf=1`, F8) ; Chrome release : 60 fps de simulation à 1 et 2 joueurs locaux ; 120 Hz et 2 onglets en ligne non mesurés | `polissage-perf-web.md` ; `418d6e9e` |
| S1-6 feel | b0 | **livré** : interpolation d'affichage (images figées 8–21 % → 1 %), flash de bouche, recul, éclats ; « encore rigide ? » à juger par William (E4) | `polissage-feel.md` ; `6c650c10` |
| S1-7 sons du moteur | b2 | **livré** : 28 clés de sons, volume global hors rollback ; audio navigateur à écouter par William (build `e4-sons`) | `polissage-sons-evenements.md` ; `19ede770` |
| S1-8 zombies visuel | b1 | **livré** : pack TheLazyStone par surcharge locale (rien de payant commité), portes doubles, fenêtres, pose d'attaque, distributeurs CC0 | `polissage-zombies-visuel.md` ; `a7a7a49b`, `0edef129` |
| S1-9 zombies sons | b1 | **livré** : armes Snake's, fenêtres Kenney, `feedback.local.ron` (Sonniss hors dépôt) ; écoute de William **non faite** | `polissage-zombies-sons.md` ; `f16f1930`, `4cea5046` |
| S1-10 zombies cartes + armes | b3, b2 | **livré** : `avant_poste` 5 → 29 spawners, 6 → 19 fenêtres ; armes à rôles distincts ; parties bloquées 14 → 1 (solo), 5 → 0 (duo), 0 desync ; 54 traces zombies réécrites (preuve § 5) ; 15 min de William **non faites** | `polissage-zombies-map-equilibrage.md` ; `0c39eb1d` |
| D176 guidage autour des fenêtres | b4 | **livré** (trouvé par S1-10) : zombies qui gelaient près d'une fenêtre ; 9 traces zombies réécrites (preuve § 5) | `polissage-nav-fenetres.md` ; `79538914` |
| S2-7 throne jouable | b3 | **livré** : D66/D67 vérifiées, campagne avant (100 graines × 1 et 2 bots), aucun réglage | `polissage-throne-jouable.md` ; `2ff27132` |
| S2-8 lisibilité coop throne | b3 | **livré** : D66/D67 fermées (vidéo), couleurs par joueur, légende F7, halos, pointeur | `polissage-lisibilite-coop.md` ; `6616909c` |
| S2-9 budgets de perf | b0 | **livré** (réduit à zombies et throne) : pire rollback 12,7 / 6,1 ms ; 96 scénarios sans budget attendent trois passes au calme sur le VPS | `polissage-perf-budgets.md` ; `dec185c9` |
| S2-10 throne visuel | b1 | **livré** : Aymen B + Ittai par surcharge locale ; restent portail, 7 armes au sol, armes portées | `polissage-throne-visuel.md` ; `197ee8a6` |
| S2-11 throne sons | b1 | **livré** : 12 armes en 4 familles, clés `shot_<arme>`/`reload_<arme>`/`mutation` ; écoute de William **non faite** | `polissage-throne-sons.md` ; `facb7c92` |
| S-0b dépôt privé d'assets | b3 | **livré** : `estran-studio/alacod-assets` (privé, 208 fichiers), `scripts/assets-prives.sh`, CI gardée par le secret **et** `vars.ALACOD_WEB_PRIVATE_ASSETS` (non posée : le site reste en repli public) | `polissage-assets-prives.md` ; `62ff6365` |
| E4 préparé | b0 | **préparé, non tenu** : protocole, builds `e4-manette`/`e4-feel`/`e4-sons` sur `127.0.0.1:8777` | `plan-polissage-notes.md` § E4 du 30 ; `b170c026` |
| **S1-3 online par Allumette** | b2-allumette | **préparé, non recetté** : D116 fermée, fin de session sans écran figé, écran figé avant partie corrigé, correctif du ticket vérifié dans Chrome (hors prod), D165/D166 ; `online` reste `false` | `polissage-reseau-amis.md` § 4 ; branche `polissage-reseau-amis` `8297b2ea` (poussée, non mergée ; worktree Mac `b2-allumette-mac` gardé pour la recette) |
| S2-12 throne cavernes | b3 | **arrêté** par la consigne du 1er nov : audit et cause des soft-locks faits (faux positif du détecteur : choix de mutation non pris par les bots) ; corrections en cours **non vérifiées** | branche `polissage-throne-map` `d8c53061` (WIP, non mergée) |

## Ce qui attend William (revue)

E4 manette (30 min) ; recette deux réseaux + zombies à 2 (heure à donner : deux Chrome sur deux réseaux, dont un en partage de connexion,
VPN coupé, accès alumette) ; écoute des sons zombies et throne (5 min) ; armes de zombies (15 min, `ALACOD_MUTE=1 make zombies GARGS='--seed 42'`).
Questions ouvertes : confirmation de TheLazyStone pour un build web public ; runner `vps-alacod-builder` hors ligne ; portail et armes de throne
en CC0 ; date et montant de l'achat d'Aymen B ; `make test_tools` rouge sur le Mac seulement (`scripts/claude-say.sh` sous bash 3.2).

## Semaine 2 (rien de lancé sans décision de William)

S1-3 recette et merge ; S2-12 (reprendre `polissage-throne-map`) ; équilibrage throne ; vidéos de 60 s ; S1-11 session d'amis ; budgets VPS des
96 scénarios ; dettes D170–D175, D177.
