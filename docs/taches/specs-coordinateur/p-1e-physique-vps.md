# P-1e « Physique du labo » — sur le VPS (coordinateur alacod, run `run_9effecfcae39`, ven 20 nov 2026)

Objectif studio O2 au service d'O3. Jalon **S2 dim 22 nov**. **En parallèle de M-7b** (décision de William du 20 nov : « toujours au moins 2 en parallèle »). Simulation (déplacement, projectiles) : checklist GGRS de `CLAUDE.md`. Tu ne merges pas.
Preuve qui fait foi : ligne **P-1e** du § 6 de `docs/taches/plan-1837-cli.md` (`8f093d2f`). § 125, D221, rapport `docs/taches/rapports/p-1e-physique-labo.md`.

## Le verdict (William, ven 20, 1837 `f2ac3bf`, natif, dans la salle)
« On est assez proche du labo, **notre physique est juste plus lente et lourde**. » Écart de parité. Décision du 6 nov : même comportement que le labo, **les défauts
des bancs font foi**. Méthode du § 6 : la mesure d'abord, puis un contrôle qui tient les nombres, puis la correction ; **aucune question de diagnostic à William**.

## Ordre
1. **Mesure, tableau chiffré banc / jeu** (avant toute correction) des grandeurs du Frère et des projectiles : vitesse de marche, vitesse de course (si le banc en a une),
   accélération, freinage (distance et temps d'arrêt), temps pour atteindre la vitesse, demi-tour (temps et distance pour inverser), vitesse des balles, recul (poussée et
   durée), plus tout réglage de déplacement qui diffère (inertie, frottement, dash/pas). Unités communes (m/s, m/s², s ; donne aussi u/frame).
   - **Jeu** : 1837 `origin/main` (copie jetable détachée `git -C /home/debian/orca/1837 worktree add --detach /home/debian/orca/workspaces/1837/p-1e-jetable origin/main`,
     `[patch]` vers ton worktree), build **release**, `xvfb-run -a`, **entrées réelles** (clavier/manette/curseur par le vrai chemin d'input, ou `ALACOD_INPUT=script:`),
     positions relevées par `alacod/state` (remote) ou trace d'état, frame par frame.
   - **Banc** : `/home/debian/workspace/1867_lore/site/labo/` (lecture seule, ni pull ni écriture ; version récente par `git fetch -q` + `git show origin/main:<chemin>`) :
     `armes/armes.js`, `armes/allures.js`, `gigue/*.js` et leurs **défauts** ; lis les constantes **et** mesure dans la page (chromium Playwright
     `~/.cache/ms-playwright/chromium-1248` sous `xvfb-run -a`, page servie en statique, une session ponctuelle, pas de boucle d'automatisation).
   - **À vérifier, pas à supposer** : le pas de temps (60 Hz fixe du jeu contre le `dt` du banc : `requestAnimationFrame`, plafond de `dt`, sous-pas), l'échelle mètre/unité
     des deux côtés (`units_per_meter` du jeu, 1,15 et 48 du banc cités par P-1d…), la conversion des défauts du banc vers les RON de 1837, l'intégration (ordre vitesse/position,
     frottement par frame vs par seconde).
   - Les deux écarts du Frère de P-1d entrent au tableau et sont **alignés sur le banc par défaut** : (a) geste du tir joué à la durée du clip au lieu de `fTir` (le rechargement
     n'entre qu'à 22 % à l'écran), (b) premier pas de gigue sur `gigue-02`. Détail : `docs/taches/rapports/p-1d-parite-observee.md` § 4.
   `status` « mesure : … » avec le tableau dès qu'il est rempli (fichier `logs/p-1e-tableau-avant.md`).
2. **Contrôle** : scénarios + attentes qui tiennent ces nombres (tolérances écrites) et **échouent sur 1837 `f2ac3bf`** ; testbed générique pour ce qui est de l'engine,
   et une forme jouable dans `alacod test` de 1837 pour ce qui est de la donnée de 1837 (tag `rapide` si court). Journal rouge `logs/p-1e-controle-rouge-<sha>.log`.
3. **Correction** selon la cause : **donnée de 1837** → décrite précisément (fichier, champ, ancienne → nouvelle valeur) dans le rapport et un `status` (le coordinateur 1837
   l'applique ; tu ne commites rien dans 1837) ; **engine** → corrigé ici, test rouge/vert, `make schedule_order_diff BASE=origin/main` si l'ordre bouge ; une trace existante
   qui bouge : dis laquelle et pourquoi (BLESS seulement justifié par écrit, et seulement si la correction d'engine change voulu la physique du testbed).
   Journal vert `logs/p-1e-controle-vert-<sha>.log`.
4. **Preuve** : tableau avant/après (`logs/p-1e-tableau.md` : banc, jeu avant, jeu après, écart) ; **capture côte à côte banc / jeu de 20 à 40 s** (mêmes gestes : marche,
   arrêt, demi-tour, tir, recul), vidéo sous `logs/p-1e-captures/`. Contrôle rapide découpé ; rapport ; ligne LIVRÉ.

## Où, règles
- Worktree `/home/debian/orca/workspaces/alacod/p-1e-physique`, branche `wquintal/p-1e-physique`. **Ta cible à toi** : `CARGO_TARGET_DIR=/home/debian/orca/workspaces/alacod/target-p1e`
  pour toutes tes commandes, y compris le release de 1837 (aucun autre dossier cible ; `target-partage` est à M-7b et au coordinateur 1837, n'y touche pas).
- Disque partagé avec un autre worker engine (M-7b) : un `make check_rapide`/contrôle découpé peut monter à ~39 Go de cible ; **deux en même temps ne tiennent pas**.
  Avant ton contrôle rapide découpé : `ask` « contrôle rapide ? » et attends mon feu vert (je les sérialise) ; après : purge de tes binaires de tests d'intégration et sweep.
  Supprime `target-p1e/release` dès que la mesure et la capture au vrai jeu sont faites.
- Préfixe `env -u CI CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 ALACOD_MUTE=1`. Un seul cargo ; jamais de profil debug ; cargo seulement si `df -h /` ≥ 20 Go libres, arrêt de TON cargo (par PID)
  sous 15 Go et `status` ; `scripts/target-sweep.sh $CARGO_TARGET_DIR` après chaque cargo ; rien de lourd entre 03:45 et 05:00 UTC (suite de nuit à 04:00).
- Contrôle rapide **découpé** : `~/orca/runs/alpha/logs/p-1d-check-rapide-decoupe.sh` et `p-1d-test-crates-decoupe.sh` sous ton nom
  (journal `logs/p-1e-physique-check-rapide-<sha>.log`), puis purge des binaires de tests d'intégration.
- Rien de 1837 dans l'engine (noms neutres, testbed). Lis `CLAUDE.md`, `~/orca/runs/alpha/commun.md` (écrit pour le Mac ; ce fichier-ci prime), les rapports P-1 et P-1d.

## Communication
Commite et pousse à chaque étape. Carte `orca worktree set --worktree active --comment …`. `status` à chaque étape (mesure, rouge, cause, vert, E prêt). 429 : `status`
tout de suite. Question bloquante : `ask` avec ton choix par défaut, et avance. Fin : rapport, LIVRÉ, copie jetable retirée, aucun processus, un seul `worker_done`.
