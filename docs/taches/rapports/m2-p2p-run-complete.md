# m2-p2p-run-complete — une run gungeon complète en p2p réel

Session b2 (VPS Debian 13 x86-64, noyau 6.12, 8 cœurs, partagé avec l'orchestrateur : charge ambiante de 1,3 à 2,3 avant
mes lancements), branche `m2-p2p-run-complete` depuis `origin/main` `64ec961c`. Docker local (signaling `matchbox_server`),
**pas d'affichage**, **pas de root** : les cibles `make *_matchbox` (fenêtre) n'ont pas servi, les clients sont les
binaires `--profile headless` du jeu (`ALACOD_HEADLESS=1`), comme `scripts/p2p-stress.sh`. **La seconde machine (Mac
arm64) était indisponible : le point 2 (deux machines) n'est pas exécuté** (§4).

**État réel par sous-point (après la revue bloquante du 2026-10-17, §7)** : 1 (2 puis 4 clients, transition d'étage) **fait** ; 2 (deux machines)
**non fait** ; 3 : restart après la fin de run à 2 et à 4 clients **fait**, boucle hub → run → victoire → hub **faite**, déconnexion d'un pair
**faite**, retour au lobby après la fin de run **partiel, bloqué par D86** (cause établie, reproduit deux fois, non corrigé) ; 4 (réseau dégradé)
**fait** pour 120 ms / 20 ms / 1 %, déconnexion à 3 % de perte (D87) ; 5 (nightly) **fait**, sans la couverture du réseau dégradé par défaut
(`NIGHTLY_GUNGEON_NETEM` vide : « netem SKIPPED »).

## 1. Résultats

Tout est du transport WebRTC réel (matchbox), `ALACOD_INPUT_DELAY=0`, inputs scriptés qui changent à chaque frame utile
(jamais `ALACOD_INPUT=neutral`), journal du jeu pour le compteur de frames resimulées, `cmp` + `check-p2p-traces.py` pour
les traces. Graine de carte 10, séquence `etages_deux` (manifeste), `--seed 10`, `ALACOD_SKIP_HUB=1` sauf mention. Machine :
la même pour tous les clients (une seule machine : ce n'est PAS une preuve inter-plateformes).

| Point | Frames | Resimulées (par client) | Transition d'étage | Verdict |
|---|---|---|---|---|
| 1. 2 clients, stress SIGSTOP | 4 500 (trace 4 499 lignes) | 7 488 / 10 | f3725, les deux journaux | **OK** : 0 `DesyncDetected`, codes 0 et 0, traces identiques |
| 1. 2 clients, sans stress (témoin) | 4 500 | 221 / 212 | f3725 | OK (le transport seul rollbacke déjà) |
| 1. **4 clients**, stress | 4 500 | 6 651 / 7 / 6 614 / 6 704 | f3428, les quatre journaux | **OK** : traces des quatre identiques (3 `cmp` + `check-p2p-traces.py`) |
| 3. Hub → run en ligne (hub réel, porte f170, salle `…-r1`), 2 clients, stress | 3 000 après le hub | 7 614 / 1 | — (inputs aléatoires ; la boucle avec bots et victoire est en §7) | **OK** : traces identiques |
| 3. Restart (`RunRequest::Restart`, comme la touche R) **après la fin de run** | partie 1 : victoire f7138, trace écrite f7200, requête posée f7260 (`ALACOD_RESTART_AT_FRAME` + 60 frames de grâce), 7 199 lignes ; partie 2 : 1 499 lignes | 15 426 / 8 (deux parties) | f3725 (partie 1) | **OK** : traces identiques entre pairs pour chaque partie ; partie 2 ≠ partie 1 (graine dérivée) ; client local : partie 2 = partie 1 |
| 3. Retour au lobby (« Lobby », requête à f300, partie en cours : pas après la fin de run) | — | — | — | **BLOQUÉ** : la partie 2 ne démarre jamais quand les deux clients reviennent à plus de ~100 ms d'écart (D86, cause en §7) |
| 3. Déconnexion, 2 clients (client 1 tué, SIGKILL, à 40 s, ~f1000, étage 0) | 4 500 | survivant : 257 | — (le portail n'est franchi que par le joueur 1, mort) | **OK** : pas de panique, `Disconnected` 1,55 s après `NetworkInterrupted`, le survivant joue seul jusqu'à f4500, exit 0, trace 4 499 lignes |
| 3. Déconnexion, 4 clients (client 2 tué à 100 s, ~f3300, juste avant le portail) | 4 500 | 887 / 831 / — / 793 | f3429, **franchie par les survivants après le départ** | **OK** : trois survivants, traces identiques entre eux, exit 0, pas de panique |
| 4. RTT 120 ms, jitter 20 ms, perte 1 % (`delay 60ms 10ms loss 1%`, sans stress) | 4 500 | 2 301 / 2 412 | f3725 | **OK**, 142 s ; traces identiques ; 4 interruptions réseau, toutes reprises |
| 4. RTT 200 ms, perte 3 % (`delay 100ms 10ms loss 3%`) | le client 0 écrit sa trace (4 499 lignes), le client 1 sort en code 3 avant | 2 025 / — | — | **ÉCHEC attendu, hors cible** : 12 interruptions, déconnexion GGRS puis `DesyncDetected` frame 1920 (client 1 code 3) |

Les sorties complètes sont reprises en §3 ; le détail du réseau dégradé (matrice) en §3.4.

## 2. Ce qui a été construit

- **`scripts/p2p-lib.sh`** (nouveau, sourcé par les autres) : binaire headless, signaling (Docker, `MATCHBOX_URL` externe, ou
  processus local dans un espace de noms réseau), `NETEM` sans root.
- **`scripts/p2p-stress.sh`** généralisé : `CLIENTS` (N), `GAME_ARGS`, `EXPECT_FLOORS` (nombre de `floor_transition` **distincts** ;
  le journal répète la ligne à chaque resimulation de la frame), `STRESS=0`, `NETEM`, `MATCHBOX_URL` ; vérifie en plus les
  interruptions réseau non reprises et lance `check-p2p-traces.py`. Les anciens appels (`p2p-stress.sh zombies inputs 3000`)
  marchent tels quels (rejoués : « OK », 4 851 frames resimulées, 2 999 lignes identiques).
- **`scripts/p2p-restart.sh <jeu>`** généralisé : `CLIENTS`, `INPUTS`, `STRESS`, `GAME_ARGS`, `RESTART_KIND=restart|lobby|hub`, `LOCAL`
  (rejoué sur zombies : parties 1 et 2 identiques entre pairs, partie 2 ≠ 1, local identique). **`scripts/p2p-disconnect.sh`**
  (pair tué), **`scripts/p2p-machine.sh`** (un client par machine), **`scripts/p2p-gungeon-inputs.sh`** (inputs de bots figés).
- **Jeu** : `gungeon` accepte `--seed` et `ALACOD_SKIP_HUB=1` ; `ALACOD_INPUT=script:` rejoue les pistes `hub:` du scénario tant que
  la session est au hub (`ScriptedHubInputs`, test unitaire) ; la trace d'état est vidée à **toute** sortie d'`InGame` (sinon les
  frames du hub comptaient comme resimulées et se mêlaient à celles de la run : 7 614 / **1** mesuré avec le correctif, le
  client 1 n'aurait pas pu afficher 1) ; `ALACOD_RESTART_KIND=lobby|hub` ; cible `make gungeon_matchbox` (pour `make
  test_multiplayer TARGET=gungeon`, avec fenêtre : non exécutée ici).
- **`scripts/nightly.sh` étape C2** (§5), `docs/conventions.md` §57 « p2p réel de la run complète », D59 fermée, D85 à D87.

## 3. Commandes exactes et sorties

Les §3.1 à 3.4 ont été mesurés sur le binaire construit à 20:54 (commit `53c3ccb8` + l'ajout d'`ALACOD_RESTART_KIND`), avant
l'extraction de `scripted_tracks` (refactor sans effet de simulation) et du message de journal ; **tout a été rejoué sur le
binaire final** (§3.5). SHA de livraison en tête du Summary.
Variables communes : `COMPOSE_PROJECT_NAME=b2`, `MATCHBOX_PORT=3566` (le projet `b2h` / 3567 pour les essais en parallèle).

### 3.1 Inputs figés des bots (graine 10, hub ignoré)

```bash
scripts/p2p-gungeon-inputs.sh 2 10 5000 /tmp/b2-rec/seed_10.ron
#  graine 10, 5200 frames enregistrées, passages d'étage [3731], morts 0      (2 bots, ~100 s)
scripts/p2p-gungeon-inputs.sh 4 10 5000 /tmp/b2-rec/four_10.ron              #  passages d'étage [3434]  (4 bots, ~230 s)
ALLOW_SHORT=1 scripts/p2p-gungeon-inputs.sh 2 10 9000 /tmp/b2-rec/seed_10_full.ron   #  7143 frames, issue Victory f7142
```

Le passage d'étage est daté f3731 par `alacod-sim` (`floor_frames`) et **f3725** par le journal du jeu en p2p (f3434 contre f3428 à
4 joueurs) : l'écart constant de 5 à 6 frames est le **délai d'input** (le synctest d'`alacod-sim` applique l'input décidé à la frame f à
f+5, les clients p2p tournent à 0) ; élucidé en §7 (D85). Les enregistrements de ce § n'étaient pas décalés ; ceux de §7 le sont.

### 3.2 Point 1 : 2 puis 4 clients

```bash
ALACOD_SKIP_HUB=1 GAME_ARGS="--seed 10" EXPECT_FLOORS=1 OUT=/tmp/b2-p1-2c \
  scripts/p2p-stress.sh gungeon /tmp/b2-rec/seed_10.ron 4500
# jeu=gungeon frames=4500 clients=2 … durée=272s ; codes de sortie : 0 0 ; DesyncDetected : 0
# frames resimulées : 7488 10 ; transitions d'étage distinctes : 1 1 (f3725) ; cmp identiques (4499) ; p2p-stress : OK
CLIENTS=4 ALACOD_SKIP_HUB=1 GAME_ARGS="--seed 10" EXPECT_FLOORS=1 OUT=/tmp/b2-p1-4c \
  scripts/p2p-stress.sh gungeon /tmp/b2-rec/four_10.ron 4500
# clients=4 durée=213s ; codes 0 0 0 0 ; DesyncDetected : 0 ; resimulées : 6651 7 6614 6704 ; transitions 1 1 1 1 (f3428)
# cmp 0/1, 0/2, 0/3 identiques ; OK: 4 pairs, 4499 frames contiguës et identiques
```

Profil de rollback relevé par le jeu (client suspendu par le stress, donc bruyant) : sauvegarde 3,8 à 8,3 ms en moyenne (58 à
119 ms au pire), restauration 3,2 à 4,8 ms, frame de simulation 2,4 à 6,0 ms, instantané ~52 à 55 Kio (2 et 4 clients).
Ce ne sont pas des mesures de performance (machine partagée, processus suspendus) ; elles disent seulement que la simulation d'un
étage de gungeon coûte 2 à 3 ms par frame sans rollback et que la restauration reste à quelques ms.

**Hub → run** (porte de gungeon, inputs aléatoires, run sur graine dérivée) :
```bash
python3 scripts/p2p-stress-inputs.py 3000 --seed 11 --hub-door > /tmp/b2-rec/hub_random.ron
OUT=/tmp/b2-hub scripts/p2p-stress.sh gungeon /tmp/b2-rec/hub_random.ron 3000
# journal : « hub : sortie du joueur 0 confirmée (frame 170), lancement de la run », « start p2p connection … (salle …-r1) »
# durée=182s ; codes 0 0 ; DesyncDetected 0 ; resimulées 7614 1 ; cmp identiques (2999) ; OK
```

### 3.3 Point 3 : restart, lobby, déconnexion

```bash
RESTART_AT=7200 EXIT_AT=1500 STRESS=1 INPUTS=/tmp/b2-rec/seed_10_full.ron ALACOD_SKIP_HUB=1 GAME_ARGS="--seed 10" \
  OUT=/tmp/b2-restart scripts/p2p-restart.sh gungeon
# run summary f7138 outcome=Victory (client 0) ; restart en ligne : partie 2, salle …-r1 ; resimulées 15426 / 8
# p2p partie 1 : identiques (7199 lignes) ; partie 2 : identiques (1499 lignes) ; partie 2 ≠ partie 1 ; local : partie 2 = partie 1 (1499) ; OK
```
**Retour au lobby** : essai à f300 (partie en cours) : la partie 2 ne démarre jamais. Interprétation d'alors (un « pair fantôme » du serveur) **corrigée
en §7** : c'est une course sur la salle d'origine, reproduite deux fois **après la fin de run**, D86.

```bash
KILL_AFTER_S=40 ALACOD_SKIP_HUB=1 GAME_ARGS="--seed 10" OUT=/tmp/b2-disc2 scripts/p2p-disconnect.sh gungeon /tmp/b2-rec/seed_10.ron 4500
# client 1 tué (SIGKILL) après 40s ; code client 0 = 0 ; « NetworkInterrupted » 21:12:49,32 puis « Disconnected » 21:12:50,87 ;
# « Player 'Player 2' disconnected » (ERROR) ; trace écrite 4499 lignes ; DesyncDetected 0 ; invariants OK
CLIENTS=4 KILL_CLIENT=2 KILL_AFTER_S=100 ALACOD_SKIP_HUB=1 GAME_ARGS="--seed 10" OUT=/tmp/b2-disc4 \
  scripts/p2p-disconnect.sh gungeon /tmp/b2-rec/four_10.ron 4500
# codes 0 0 137 0 ; les trois survivants : « Disconnected » à 21:16:49,06 ; transition d'étage f3429 à 21:16:54 (APRÈS le départ,
# franchie par le joueur 0) ; 4499 lignes chacun, traces identiques entre eux ; DesyncDetected 0 ; invariants OK
```
Ce qui se passe : GGRS notifie l'interruption après 500 ms, déclare le pair déconnecté après 2 000 ms de silence au total (réglages par
défaut, le jeu ne les touche pas), la simulation du survivant se bloque ce temps-là puis reprend avec l'input du pair parti
neutre ; le personnage du pair reste dans le monde, immobile. **En headless**, `log_ggrs_events` n'arrête le processus que sur
`DesyncDetected` : le survivant termine sa partie (c'est ce que mesurent ces deux essais). **Avec fenêtre**, `ui::disconnected`
affiche l'écran « GAME DISCONNECTED » (code lu, non observé : pas d'affichage ici ; `docs/etat-engine.md` risque 3 décrit la
politique de session comme brutale par choix par défaut, dette inchangée).

### 3.4 Point 4 : réseau dégradé sans root

`tc netem` demande root ; ici `unshare -Urn` (espaces de noms utilisateur **et** réseau sans privilège, noyau 6.12) donne un `lo` où
`tc qdisc add dev lo root netem …` fonctionne ; WebRTC ne pose aucun candidat sur la boucle locale : une interface `dummy0`
(10.77.0.1/24) est ajoutée (le trafic vers sa propre adresse passe par `lo`, donc par netem) ; le signaling (`matchbox_server`
extrait de l'image Docker) tourne dans le même espace. Sans `dummy0`, le premier essai est resté sur « Waiting for players 1/2 »
jusqu'à la limite de 900 s. Le délai de netem s'applique à **chaque** sens (RTT = 2 × délai) ; `ping -c 40 -i 0.1` à l'intérieur le
mesure avant chaque run.

```bash
NETEM="delay 60ms 10ms loss 1%" STRESS=0 MIN_RESIMULATED=1 ALACOD_SKIP_HUB=1 GAME_ARGS="--seed 10" EXPECT_FLOORS=1 \
  OUT=/tmp/b2-p4-netem120 scripts/p2p-stress.sh gungeon /tmp/b2-rec/seed_10.ron 4500
# ping : rtt min/avg/max/mdev = 105,1/119,3/137,7/8,5 ms, 0 % de perte (40 paquets) ; durée=142s
# resimulées 2301 2412 ; transitions 1 1 (f3725) ; interruptions réseau reprises 4/4 ; DesyncDetected 0 ; cmp identiques ; OK
```

Matrice mesurée (2 clients, même run, 4 500 frames, sans stress de suspension, une exécution par cellule, deux cellules en
parallèle sur la machine : les durées sont indicatives). `delay` = par sens ; « interruptions » = `NetworkInterrupted` suivi de
`NetworkResumed`.

| RTT mesuré (ping) | Jitter | Perte (par sens) | Durée | Resimulées | Interruptions reprises | Verdict |
|---|---|---|---|---|---|---|
| 60 ms (60,1 à 65,3) | 0 | 0 | 117 s | 1 198 / 1 283 | 0 / 0 | OK |
| 120 ms (120,1 à 121,4) | 0 | 0 | 122 s | 2 363 / 2 312 | 0 / 0 | OK |
| 200 ms (200,1 à 208,7) | 0 | 0 | 171 s | 2 619 / 3 535 | 0 / 0 | OK |
| 60 ms | 0 | 1 % | 154 s | 1 244 / 1 248 | 0 / 0 | OK |
| **120 ms (105 à 138)** | **±20 ms** | **1 %** | 142 s | 2 301 / 2 412 | 4 / 4 | **OK** (point 4 de la fiche) |
| 200 ms (184 à 223) | ±20 ms | 1 % | 167 s | 3 075 / 3 587 | 4 / 4 | OK |
| 120 ms (102 à 136) | ±20 ms | 3 % | 189 s | 3 049 / 3 087 | 26 / 27 | **ÉCHEC du script** : une déconnexion GGRS en fin de partie (client 1, après la sortie du client 0), dernière ligne de trace (f4498) différente ; pas de `DesyncDetected` |
| 200 ms (187 à 215) | ±20 ms | 3 % | 143 s | 2 025 / — | 10 / 12 | **ÉCHEC** : déconnexion GGRS en cours de partie (frames ~2 000), puis `DesyncDetected` frame 1920, client 1 code 3 |

La « perte » est celle de chaque sens : un aller-retour perd ~2× plus (le ping mesure 7,5 % pour 3 %). Stalls : chaque interruption est
un silence de 500 ms (avant la notification) plus 0,56 à 0,9 s (après), soit 1,1 à 1,4 s au total pour un délai de déconnexion
de 2 s : **à 3 % de perte la marge est épuisée**. Le jeu ne règle pas ces délais et utilise un canal matchbox fiable (`ChannelConfig::reliable()`) ;
[INFERENCE, non vérifiée] une perte sur un canal fiable ordonné bloque ce qui suit jusqu'à la retransmission, d'où ces silences
alors que GGRS sait absorber des pertes tout seul. Dette **D87**. Le `DesyncDetected` du dernier cas suit la déconnexion
(chaque pair continue sans l'autre) : c'est la conséquence d'une session cassée, pas d'un écart de simulation ; je ne l'ai pas
démontré autrement (le client 0 a déconnecté le pair 7 s avant le client 1 dans ce run).
Profondeur et fréquence des rollbacks : le jeu ne journalise pas la profondeur d'un rollback ; seul le total de frames resimulées
est mesuré (la fenêtre de prédiction est de 12 frames, `MAX_PREDICTION_WINDOW`). Les 70 `WaitRecommendation` du client 0 sous stress
(0 pour le client 1) disent que le client qui devance se fait freiner par GGRS.

### 3.5 Rejeu sur le binaire final

| Essai | Frames | Resimulées | Verdict |
|---|---|---|---|
| 2 clients, stress, f3725 | 4 500 | 10 195 / 6 | OK, 376 s, 2 interruptions reprises, traces identiques |
| 4 clients, stress, f3428 | 4 500 | 6 840 / 2 / 7 207 / 7 013 | OK, 222 s, traces des quatre identiques |
| Hub → run (porte f170), 2 clients, stress | 3 000 | 4 049 / 1 | OK, 107 s, traces identiques |
| Restart après la victoire (f7200 → requête f7260), 2 clients, stress | 7 199 puis 1 499 | 10 588 / 1 (deux parties) | OK : traces identiques pour chaque partie, partie 2 ≠ 1, local identique |
| Déconnexion, 2 clients (SIGKILL à 40 s) | 4 500 | — | OK : survivant exit 0, trace 4 499 lignes |
| Déconnexion, 4 clients (SIGKILL à 100 s) | 4 500 | — | OK : trois survivants, traces identiques |
| Réseau 120 ms / ±20 ms / 1 % | 4 500 | 2 026 / 1 763 | OK, 201 s, 6 interruptions sur 6 reprises, traces identiques |

**Incident de charge, à connaître** : un premier rejeu a lancé ces essais en **trois voies parallèles** pendant que la compilation d'un autre
worker saturait la machine (charge 8 à 11 sur 8 cœurs) : 3 essais sur 7 ont échoué par la même cause, une déconnexion GGRS (silence de plus
de 2 s d'un pair affamé de CPU ou suspendu) — 4 clients : 4 `DesyncDetected` après la déconnexion ; hub : un client jamais sorti (tué à 900 s) ;
restart : traces de la partie 2 différentes à la ligne 214 après une déconnexion, `DesyncDetected` 0. Rejoués **un par un** (mon script de lancement, non commité,
attendait une charge < 3 avant chaque essai), ils passent. Ce ne sont pas des divergences de simulation (rien n'a divergé
avant la déconnexion, `p2p partie 1` identique), mais cela dit que **les preuves de ce rapport demandent une machine au calme** et que la
session p2p n'absorbe pas une pause de 2 s d'un pair (D87).
Un défaut de mon script a aussi été trouvé à ce rejeu : `p2p-restart.sh` (`set -e`) sortait en **code 1 même en cas de succès** (le `kill` du
nettoyage sur un processus déjà parti) ; corrigé (`|| true`), vérifié (code 0 sur un restart court). Les autres scripts n'ont pas `set -e`.

## 4. Ce qui attend la seconde machine (point 2, non exécuté)

Préparé, jamais lancé : `scripts/p2p-machine.sh <jeu> <script.ron> <frames> <index>` (un client par machine, `MATCHBOX_URL` et `LOBBY`
obligatoires, `CLIENTS=2`, `GAME_ARGS`, `ALACOD_SKIP_HUB`). Écrit `$OUT/p2p-<index>.trace` et imprime architecture, commit,
code de sortie, `DesyncDetected`, frames resimulées, transitions d'étage, nombre de lignes, sha256 et dernière ligne de la
trace ; la comparaison se fait en copiant une trace sur l'autre machine (`python3 scripts/check-p2p-traces.py --frames 4499 a.trace b.trace`).
```bash
# sur UNE machine : enregistrer les inputs, copier run.ron sur l'autre
scripts/p2p-gungeon-inputs.sh 2 10 5000 run.ron
# machine A (x86-64) et machine B (arm64), même commit, même signaling joignable des deux (ex. wss://allumette.bascanada.org) :
MATCHBOX_URL=wss://… LOBBY=m2-run-1 ALACOD_SKIP_HUB=1 GAME_ARGS="--seed 10" scripts/p2p-machine.sh gungeon run.ron 4500 0   # A
MATCHBOX_URL=wss://… LOBBY=m2-run-1 ALACOD_SKIP_HUB=1 GAME_ARGS="--seed 10" scripts/p2p-machine.sh gungeon run.ron 4500 1   # B
```
Les inputs doivent être **enregistrés sur une machine** : les bots sont déterministes, mais `alacod-sim` et le jeu n'ont pas à donner le
même enregistrement sur x86 et arm64 (c'est justement D73 qu'on teste, fausse alerte constatée par ailleurs). À vérifier en réel :
D73, la traversée NAT, le comportement sur un vrai RTT. Le script a été essayé sur cette machine en deux instances (signaling Docker local
`ws://127.0.0.1:3567`, salle `machine-test-1`, 4 500 frames, sans stress) : 943 / 617 frames resimulées, transition f3725, traces de
4 499 lignes de même sha256 (`d7c44542…c842`), `check-p2p-traces.py` OK ; jamais entre deux architectures ni sur un vrai réseau.

## 5. Nightly

`scripts/nightly.sh`, étape **C2** (après C, avant D), `NIGHTLY_P2P_GUNGEON=1` par défaut hors `--quick` : enregistre les inputs de
2 bots (`NIGHTLY_GUNGEON_SEED=10`), joue `p2p-stress.sh gungeon` 4 500 frames avec `EXPECT_FLOORS=1`, signaling du service Docker
`signaling` ; échoue le nightly si une condition de `p2p-stress.sh` échoue (traces, desync, resimulées < 1 000, aucun étage franchi) ;
`NIGHTLY_GUNGEON_NETEM="delay 60ms 10ms loss 1%"` ajoute la même run sur réseau dégradé. Le résumé reçoit commit, frames resimulées,
transitions. **Éprouvé hors du script complet** : le bloc extrait de `nightly.sh` (mêmes lignes, quatre fonctions d'enveloppe) a été
exécuté seul : 412 s, « PASSED », 8 162 / 3 frames resimulées, 1 / 1 transition d'étage ; le nightly complet (étapes A, B, D…)
n'a pas été lancé (heures), la variante `NIGHTLY_GUNGEON_NETEM` non plus (même `p2p-stress.sh`, essayé à la main en §3.4).

## 6. Vérifications faites, limites

- **Faites** (résultats réels) : voir §1 et §3 ; `cargo fmt --all -- --check` propre ; `make test_tools` 18 tests OK ; test unitaire
  `scripted_tracks_tests::hub_tracks_only_while_in_hub` ; rejeu des anciens appels (`p2p-stress.sh zombies`, `p2p-restart.sh`) :
  OK. `make test_scenarios_par SHARDS=3` sans BLESS sur `5163d9a7` (tout le Rust de la branche) : **497 scénarios verts**, 1 332 s, aucune `.trace` modifiée
  (`git status` propre hors mes fichiers) ; les commits suivants ne touchent que des scripts et de la documentation.
- **Hub avec les bots** : résolu en §7 (D85 fermée : délai d'input) ; les preuves des §3.2 à 3.4 à bots utilisent `ALACOD_SKIP_HUB=1`, celle de §7 passe par le hub.
- Les inputs des bots sont **figés** (enregistrement) : un changement de simulation les rend caducs ; `EXPECT_FLOORS` l'attrape.
  Les bots ne décident pas en ligne.
- Une machine seulement (x86-64) : aucune conclusion sur arm64, D73, la traversée NAT ni un vrai WAN. `netem` agit sur la boucle locale
  d'un espace de noms : pas de bande passante limitée, pas de réordonnancement, pas de perte en rafale.
- La détection d'étage est lue dans le journal (`floor_transition`), pas dans l'état : `FloorIndex` n'est pas exposé hors scénario.
- Pair tué = SIGKILL, pas la fermeture propre d'une fenêtre ni une coupure réseau.
- Retour au lobby : bloqué par D86 (§7) ; la preuve « Lobby après la fin de run » n'est donc pas faite.
- Pas de campagne : une exécution par cellule de la matrice, une graine, une run.
- Sans `MIN_RESIMULATED`, un client suspendu par le stress n'accumule presque pas de resimulation (10, 7, 1, 8) : seul le client qui
  devance en a ; le critère reste « le plus resimulé », comme dans la recette d'`outillage-p2p-stress`.

## 7. Revue bloquante du 2026-10-17 : corrections et essais ajoutés

Binaire reconstruit sur `origin/main` `d4162f18` fusionné. Charge : chaque essai p2p a attendu `uptime` < 3 avant de lancer ses clients
(`P2P_WAIT_CALM=3`, affichée par les scripts : « charge machine au lancement ») ; elle a été tenue **au lancement** (2,05 à 2,98), pas
**pendant** : l'autre worker (b6) compilait et la charge de fin d'essai est montée à 3,4–7,6 sur trois essais. Aucun n'a été déconnecté. Le disque
partagé est tombé à 0 octet libre pendant la session (b6 : 106 Go) : `scripts/target-sweep.sh` sur mon target a libéré 19 Go.

### 7.1 Corrections demandées
1. `scripts/p2p-machine.sh` exige maintenant `FRAMES-1` lignes contiguës (`check-p2p-traces.py --single`, test `SingleTraceCliTests`) et
   refuse une trace que le journal dit tronquée (`trace d'état tronquée`) ; un sha256 égal ne suffit plus.
2. `scripts/nightly.sh` C2 : sans `NIGHTLY_GUNGEON_NETEM`, le résumé affiche « gungeon netem : SKIPPED … réseau dégradé NON couvert », avertit
   dans le journal et la ligne finale de C2 énumère ce qui est couvert (« netem SKIPPED » ou le réglage joué).
3. D87 reformulée (GGRS déconnecte après plus de 2 s sans paquet reçu ; les reprises à 1,1–1,4 s ne prouvent pas un défaut du délai ;
   la dette est « mesurer le silence fatal et tester un délai plus long »).

### 7.2 Essais ajoutés (2 clients sauf mention ; SIGSTOP ; délai d'input 0 ; graine 10 ; frames resimulées du journal du jeu)

| Essai | Frames | Resimulées | Verdict |
|---|---|---|---|
| **4 clients jusqu'à la victoire puis restart** (`CLIENTS=4 RESTART_AT=6700 EXIT_AT=1500 EXPECT_VICTORY=1`, `four_10_full.ron`) | partie 1 : 6 699 lignes, **victoire f6374 chez les quatre** ; partie 2 : 1 499 | 10 071 / 28 / 9 969 / 10 609 | **OK** : traces des quatre identiques pour chaque partie (3 `cmp` par partie), partie 2 ≠ 1, 0 `DesyncDetected` |
| **Retour au lobby après la fin de run**, essai 1 (`RESTART_KIND=lobby RESTART_AT=7200`, victoire f7138, requête f7260) | partie 2 jamais démarrée | — | **ÉCHEC** (D86), écart 113 ms entre les deux requêtes |
| idem, essai 2 | partie 1 : 7 199 ; partie 2 : 1 499 | 461 / 414 | OK (écart 5 ms) : partie 2 = partie 1 (graine d'origine), traces identiques |
| idem, essai 3 | partie 2 jamais démarrée | — | **ÉCHEC** (D86), écart 329 ms |
| idem, essai 4 | partie 1 : 7 199 ; partie 2 : 1 499 | — | OK : traces identiques, partie 2 = partie 1 |
| **Boucle hub → run → victoire → hub** (`HUB=1`, `INPUTS=hub_full_s5.ron`, `RESTART_KIND=hub RESTART_AT=7800 EXIT_AT=120`, **sans** `ALACOD_SKIP_HUB`) | run : 7 799 lignes, **victoire f7461 chez les deux** (enregistré f7460) ; second hub : 119 | 6 640 / 5 | **OK** : porte f170, salle `…-r1`, traces identiques pour la run et pour le second hub, partie 2 ≠ 1 |
| même boucle, enregistrements **non décalés** (deux essais) | 7 799 | 8 561 / 9 ; 9 771 / 4 | traces identiques, **aucune victoire** (boss mort f3738 / f3868) : cause D85 ci-dessous |
| Recette du nightly (`p2p-gungeon-inputs.sh` décalé + `p2p-stress.sh`, 2 clients) | 4 500 | 4 088 / 4 | OK : franchissement d'étage **f3730** en ligne pour f3731 enregistré (avant le décalage : f3725), traces identiques |

La première trace de la boucle (le premier hub, 172 frames) n'est pas écrite : l'enregistreur est vidé à la sortie du hub, la run seule est
tracée ; l'entrée en run depuis le hub a ses preuves (§3.2, 7 614 / 1 resimulées, journal « hub : sortie du joueur 0 confirmée (frame 170) »).
La victoire est lue dans le journal du jeu (`run summary … outcome=Victory`), exigée par `EXPECT_VICTORY=1` pour chaque client.

### 7.3 D85 : cause trouvée, fermée
Les inputs des bots, rejoués après le hub, ne gagnaient pas. **Cause : le délai d'input.** Le synctest d'`alacod-sim` applique l'input décidé à
la frame f à la frame f+5 ; les clients p2p des preuves tournent à `ALACOD_INPUT_DELAY=0` (sinon plus de rollbacks forcés) : la run était décalée de
5 frames. C'est aussi l'écart f3731 / f3725 du franchissement d'étage (§3.1). Sans hub la run tolérait ce décalage ; avec la graine dérivée du hub
(`restart_seed`, §33) non. Correction : `scripts/p2p-gungeon-inputs.sh` décale les segments de `INPUT_SHIFT` (5) frames. Éliminés avant d'y
arriver (mesures) : la graine de run (deux enregistrements, avec et sans la dérivation `ALACOD_RUN_SEED_ONLINE_GAME=1`), le hub dans le runner
(`ALACOD_SIM_HUB=1` rejoue la même run que sans hub : 7 461 frames, étage f3627, victoire f7460), et l'état initial (dump complet
`ALACOD_STATE_TRACE_FULL=1` : les 99 premières frames d'une run en ligne après le hub et après un restart sont identiques). Les traces du runner
et d'une partie en ligne diffèrent dès la frame 0 **même sans hub** (checksums de natures différentes) : elles ne se comparent pas.

### 7.4 D86 : retour au lobby, cause établie, non corrigé
Reproduit **deux fois après la fin de run** (essais 1 et 3 ci-dessus), avec le signaling Docker (`<OUT>/signaling.log`, désormais sauvé par les scripts) :
- Essai 1 (`/tmp/b2r/lobby1`) : client 1 pose `ToLobby` à 23:53:20,636 et rouvre la salle d'origine à .683 (nouvel id `8f92…`) ; **à .757 il voit
  `8941…`, qui est l'ancien id du client 0** (encore dans la partie, requête à .749), et passe à `GameLoading` avec lui. Le serveur retire `8941…` à
  .793 (le client 0 ferme son socket de partie). Le client 0 (nouvel id `f35e…`) voit alors `8f92…` à .883 et construit sa session sur lui. Chacun a une
  session GGRS sur un identifiant que l'autre n'écoute pas : aucun événement `Synchronizing` après le retour, les deux journaux s'arrêtent à
  « Map loading complete, transitioning to GameStarting state ».
- Essai 3 (`/tmp/b2r/lobby3`) : même séquence, écart 329 ms (client 1 `.357`, client 0 `.686`, ancien id retiré à `.7407`).
- Passages (essais 2 et 4) : les deux requêtes à ~5 ms, les deux anciens sockets retirés (`00:06:27,133`) avant que les neufs ne se connectent (`.138`).

**Ce n'est pas un état non remis à zéro** (contrairement aux portes du hub) : c'est une course de protocole. Le retour au lobby rouvre la salle
**d'origine**, celle de la partie qui finit ; le pair qui revient le premier s'y connecte au **socket encore vivant** de l'autre. « Rejouer » l'évite
(salle `{lobby}-r{n}` neuve). **Conséquence pour un humain** : appuyer à quelques secondes d'écart ne l'évite pas, il l'aggrave (le premier revenu
rencontre forcément l'ancien socket du second). Correction non faite (choix de conception : salle `{lobby}-l{n}`, poignée de main de lobby, ou abandon
d'une session qui ne se synchronise pas) ; pistes en D86. **Le point 3 reste donc partiel** pour « retour au lobby en ligne après la fin de run ».

### 7.5 Aussi trouvé
- `p2p-restart.sh` (`set -e`) : sortait en code 1 à tort (corrigé au rejeu précédent) ; `set -e` + `grep` sans correspondance (`EXPECT_VICTORY`) l'aurait fait
  mourir en silence : corrigé.
- Les essais précédents (§3) n'avaient pas décalé les inputs ; leurs verdicts (traces identiques entre pairs) ne dépendent pas du décalage, mais la run
  rejouée n'était pas exactement celle des bots (victoires f7138 pour f7142, f6374 pour f6378).
