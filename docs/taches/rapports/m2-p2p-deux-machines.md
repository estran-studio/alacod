# m2-p2p-deux-machines — p2p entre deux machines (x86-64 contre arm64) et mesure de D87

Worker b2, 2026-10-18, branche `m2-p2p-deux-machines` (depuis `origin/main` `7ee084e8`). Deux machines : le VPS Debian (x86-64, Haswell 8 cœurs, noyau
6.12, `rustc` 1.101.0-nightly du 2026-10-17, binaire **statique**, partagé avec b6 : charge 3,7 à 12 selon les essais) et le Mac de William
(Apple M1 Pro, 8 cœurs, macOS 27.0.1, `rustc` 1.96.0-nightly du 2026-03-26, Bevy en **bibliothèque dynamique** `game/dev-dylib`, partagé avec
d'autres workers : charge 8,6 à 85 selon les essais, 3 à 4 processus de simulation en parallèle). Deux compilateurs et deux éditions de liens différents
sur le même commit : c'est voulu par l'état des postes, et plus sévère pour la preuve.

## 1. Résultat

| Point demandé | État |
|---|---|
| 1. gungeon graine 10, étage franchi, 4 500 frames, x86-64 contre arm64 | **fait** : traces identiques (`check-p2p-traces.py --frames 4499`, `sha256`), 0 `DesyncDetected`, frames resimulées > 0 des deux côtés (essai `gung-e`), architecture affichée (§3) |
| 1. isaac (inputs actifs, étage franchi) | **fait** (graine du manifeste : D108) : étage franchi f2922, mêmes critères |
| 1. essai court à 4 clients (2 par machine) | **fait** : 4 000 frames, étage f3369, quatre traces identiques |
| 1. écart x86 / arm64 | **aucun** : D73 confirmée fausse alerte en réel (9 parties, 20 clients) ; pas de première frame divergente à chercher |
| 2. D87 : mesure (silences, journal horodaté, charge), matrice netem, `disconnect_timeout` 5 s, canal non fiable | **faite** (§5) : 31 essais ; **décision : `disconnect_timeout` 2 s → 5 s**, canal inchangé ; D87 fermée |
| 3. `docs/taches.md` ligne 7, `docs/etat-engine.md` | **faits** |

Trois constats hors demande, en dettes : **D108** (`isaac` ignore `--seed`), **D109** (connexion WebRTC jamais établie sous 3 % de perte, le lobby attend sans fin),
et l'asymétrie de resimulation (§4.2).

## 2. Montage

- **Signaling.** Le serveur public `wss://allumette.bascanada.org` (ancien nom, déjà noté mort dans `docs/plan-site-public-beta.md:74`) **ne résout plus** (NXDOMAIN depuis le VPS et depuis le Mac) ; le défaut réel du Makefile, `wss://allumette.estran.studio`, résout (158.69.192.54, HTTP 200 sur `/health`) mais **je ne l'ai pas utilisé pour les essais** (voir §7) :
  `matchbox_server 0.0.0.0:3536` (binaire extrait de l'image Docker du compose, `target/matchbox_server`) lancé sur le VPS, `MATCHBOX_URL=ws://148.113.207.199:3536`
  des deux côtés (adresse publique du VPS ; le Mac est sur `192.168.8.0/24`, derrière un NAT, ping VPS ↔ Mac 26,7 / 33,0 / 40,5 ms, 0 % de perte sur 20 paquets).
  ICE : STUN Google par défaut du jeu (`stun.l.google.com`, résolu des deux côtés) ; la connexion WebRTC s'établit à chaque essai.
- **Commit.** `7ee084e8` des deux côtés pour tous les essais du §3 (VPS : arbre propre au build ; Mac : `git worktree add --detach ~/orca/p2p-verif 7ee084e8`, target APFS cloné
  `cp -c -R ~/orca/target/headless ~/orca/p2p-target/headless`, `CARGO_BUILD_JOBS=4`, un seul client à la fois sauf l'essai à 4). Seul `scripts/p2p-machine.sh`
  a été copié dans le worktree du Mac (options `STRESS`, `MIN_RESIMULATED`, §6). Reprise sur le commit livré : §3.4.
- **Inputs.** gungeon : `scripts/p2p-gungeon-inputs.sh 2 10 5000 run.ron` (bots `explorateur`, `etages_deux`, étage franchi à la frame 3 503 de l'enregistrement,
  décalage +5 pour un client à délai 0) copié sur le Mac ; à 4 joueurs `p2p-gungeon-inputs.sh 4 10 5000` (étage 3 370). Les deux clients tournent à `ALACOD_INPUT_DELAY=0`,
  `ALACOD_SKIP_HUB=1`, `--seed 10`.
- **Commande par machine** (exemple gungeon) : `MATCHBOX_URL=ws://148.113.207.199:3536 LOBBY=gung-e ALACOD_SKIP_HUB=1 GAME_ARGS="--seed 10" MIN_RESIMULATED=1 OUT=… scripts/p2p-machine.sh gungeon run.ron 4500 <0|1>`
  (client 0 sur le VPS et 1 sur le Mac ; échangés pour `gung-b`), plus `STRESS=1 SUSPEND_MS=80 RUN_MS=40` sur le Mac pour `gung-e`. Les traces du Mac sont recopiées sur le VPS
  puis `python3 scripts/check-p2p-traces.py --frames 4499 p2p-0.trace p2p-1.trace`.

## 3. Deux machines : un tableau par essai

Colonnes : VPS = Linux x86-64, Mac = Darwin arm64 (lignes `p2p-machine : Linux x86_64` / `Darwin arm64` du script). « Resimulées » = `frames resimulées après rollback` du journal
du jeu, VPS / Mac. Charge = `loadavg` 1 min au lancement, VPS / Mac. Tous : code de sortie 0 des deux côtés, `DesyncDetected` 0, 4 499 lignes contiguës (599 pour `smoke1`, 3 999 pour l'essai à 4),
aucun « trace tronquée ».

### 3.1 gungeon (graine 10, étage franchi à la frame 3 502)

| Essai | Frames | Réglage | Resimulées VPS / Mac | Charge VPS / Mac | Empreinte `sha256` de la trace (identique des deux côtés) | Verdict |
|---|---|---|---|---|---|---|
| `smoke1` | 600 | VPS client 0, Mac client 1 | 1 226 / 39 | 3,7 / 74,5 | `75f11614…79223ec` | traces identiques |
| `gung-a` | 4 500 | idem | 6 328 / 704 | 4,4 / 84,3 | `0b094c2dfd2c1905…a9018aa` | **OK** (étage f3502) |
| `gung-b` | 4 500 | **rôles échangés** (Mac client 0), `STRESS=1` des deux côtés (12 / 30 ms) | **0** / 25 099 | 4,0 / 10,2 | `0b094c2d…` | traces identiques ; critère « resimulées > 0 » **non tenu** côté VPS (le script le dit : ÉCHEC) |
| `gung-d` | 4 500 | `STRESS=1` du Mac seul, 60 / 100 ms | **0** / 17 627 | 9,9 / 9,4 | `0b094c2d…` | idem |
| `gung-e` | 4 500 | `STRESS=1` du Mac seul, **80 / 40 ms** | **23** / 9 580 | 9,4 / 8,6 | `0b094c2dfd2c1905…a9018aa` | **OK** : critères tenus des deux côtés |

La même empreinte `0b094c2d…` sort de **chaque** essai, quels que soient les rôles, le stress ou la machine, et aussi des essais à un seul VPS sous netem (§5 : perte de 1 et 3 %,
RTT 120 et 200 ms, canal fiable ou non, délai de déconnexion de 2 à 8 s) : 44 traces sur 53 sont identiques (les 53 sont complètes, 4 499 lignes) ; les 9 autres sont exactement celles des clients qui ont déconnecté leur pair (7 sessions perdues en cours de partie,
2 fins de partie), où chaque pair a continué seul. Un essai (`gung-c`) a échoué au lancement pour une erreur de
mon script d'enchaînement (variable d'environnement vide devenue commande) : sans effet sur la preuve, non compté.

### 3.2 isaac

Les bots `explorateur` d'`alacod-sim` enregistrent des inputs (`--game isaac --bots 2 --floors sous_sol --until-floor 2`, +5 de décalage). **`isaac` ignore `--seed`** (D108) : la graine de la partie
est toujours celle du manifeste (123456). Un premier enregistrement à la graine 5 n'a donc pas reproduit la partie (défaite f3130, aucun étage franchi, vérifié en local
x86 : même défaite) ; il reste utile (inputs actifs, 11 093 frames resimulées, traces identiques entre architectures) mais ne prouve pas l'étage. Les essais utiles sont enregistrés à 123456
(étage franchi f2923 à l'enregistrement, f2922 en ligne).

| Essai | Frames | Réglage | Resimulées VPS / Mac | Charge VPS / Mac | Empreinte | Verdict |
|---|---|---|---|---|---|---|
| `isa-a` (graine 5, écarté pour l'étage) | 4 500 | sans stress | 1 727 / 11 093 | 5,8 / 44,5 | `fe0f166c…` | traces identiques, **pas d'étage** (défaite f3130 des deux côtés) |
| `isa-b` (graine 123456) | 4 500 | sans stress | **0** / 15 135 | 4,4 / 11,3 | `a0f0e1d2265e4a9f…49b1` | traces identiques, étage f2922 ; resimulées VPS 0 |
| `isa-c` (graine 123456) | 4 500 | `STRESS=1` du Mac, **120 / 30 ms** | **227** / 3 238 | 12,1 / 9,6 | `a0f0e1d2265e4a9f358cf5b0489f219b0529c7a75c8948ad04f5f893043249b1` | **OK** : étage `floor_transition from=0 to=1` f2922 des deux côtés, critères tenus |

### 3.3 Quatre clients (deux par machine)

`CLIENTS=4` : clients 0 et 1 sur le VPS, 2 et 3 sur le Mac, 4 000 frames (étage `floor_transition` f3369 pour tous), sans stress.

| Client | Machine | Resimulées | `sha256` de la trace (3 999 lignes) |
|---|---|---|---|
| 0 | VPS x86-64 | 298 | `26f9c45771ee04551294810d1866efbe2ca525d5d7a298e8fb58e2ebce5826b8` |
| 1 | VPS x86-64 | 245 | idem |
| 2 | Mac arm64 | 29 820 | idem |
| 3 | Mac arm64 | 29 792 | idem |

`check-p2p-traces.py` : « OK: 4 pairs, 3999 frames contiguës et identiques » ; charge VPS 6,6 / Mac 9,4 ; 0 `DesyncDetected`, codes de sortie 0.

### 3.4 Sur le commit livré

Reprise sur le commit de la branche livrée (`4a1f1220` : code identique à la livraison, délai de déconnexion de 5 s par défaut, aucune variable d'essai), **des deux côtés** (worktree du Mac mis à ce commit, binaires reconstruits `gungeon` et `isaac`) ; signaling `matchbox_server` du VPS ; charge VPS 1,9 à 2,2, Mac 13,7 à 13,8 (2 clients) et 8,7 (4 clients).

| Essai | Frames | Réglage | Resimulées | Empreinte `sha256` (identique chez tous) | Verdict |
|---|---|---|---|---|---|
| gungeon graine 10 | 4 500 | `STRESS=1 SUSPEND_MS=80 RUN_MS=40` du Mac, `MIN_RESIMULATED=1` | VPS 80 / Mac 12 396 | `0b094c2dfd2c1905…a9018aa` | **OK**, étage f3502 |
| isaac graine 123456 | 4 500 | `STRESS=1 SUSPEND_MS=120 RUN_MS=30` du Mac, `MIN_RESIMULATED=1` | VPS 740 / Mac 1 936 | `a0f0e1d2265e4a9f…49b1` | **OK**, étage f2922 |
| gungeon, 4 clients (VPS 0 et 1, Mac 2 et 3) | 4 000 | sans stress | 143 / 166 / 41 501 / 41 504 | `26f9c45771ee04551294810d1866efbe2ca525d5d7a298e8fb58e2ebce5826b8` | **OK**, étage f3369 |

Les empreintes sont celles du §3.1 à §3.3 : le changement de délai de déconnexion ne touche pas la simulation.

## 4. Constats

### 4.1 D73 : aucun écart x86 / arm64

Aucune première frame divergente à chercher : 9 parties (20 clients) identiques, y compris les 29 800 frames resimulées des clients du Mac de l'essai à 4 (25 099 au plus dans un essai à 2 sous stress). La simulation est
bit à bit la même sur x86-64 (statique, `rustc` 2026-10) et arm64 (dynamique, `rustc` 2026-03), avec un hash de checksum qui ne dépend pas de l'architecture. Limites : natif seulement (pas de wasm),
une seule paire de machines, une graine par jeu.

### 4.2 Qui resimule

Sur les 8 parties à deux clients, le Mac a toujours resimulé (39 à 25 099 frames) et le VPS 0 frame dans 3 d'entre elles (`gung-b`, `gung-d`, `isa-b`). [INFERENCE] Le client qui avance le plus vite
prédit les inputs de l'autre et resimule ; le plus en retard les a déjà reçus. Pour exiger « resimulées > 0 des deux côtés », suspendre la machine qui resimule déjà longuement : `STRESS=1 SUSPEND_MS=80 RUN_MS=40`
sur le Mac (`gung-e`) ou `120 / 30` (`isa-c`). Ce n'est pas un défaut : `MIN_RESIMULATED` le dit au lieu de laisser passer une preuve sans rollback.

### 4.3 `isaac --seed` (D108) et connexion jamais établie (D109)

D108 : voir §3.2. D109 : sur 24 démarrages à 3 % de perte (31 essais, dont 7 à 1 %), un (`d87b-1`, RTT 200 ms) n'a jamais établi la connexion WebRTC : les deux clients restent à « Waiting for players: 1/2 connected » jusqu'au
garde-fou de 900 s ; `wait_for_players` n'a aucun délai.

## 5. D87 : silences fatals sous perte de paquets

### 5.1 Méthode

`scripts/p2p-d87.sh <script> 4500 <dossier>` : pour chaque cellule réseau et chaque réglage de session, `scripts/p2p-stress.sh` (2 clients, `STRESS=0`, `tc netem` sans root dans `unshare -Urn`, gungeon
graine 10, `run.ron` de §2), plus un échantillon du **taux d'occupation du CPU** toutes les 2 s (`/proc/stat`) ; `scripts/p2p-silences.py` lit les journaux horodatés (microsecondes) des deux clients :
un silence repris vaut 500 ms (délai de notification GGRS) + l'écart `NetworkInterrupted` → `NetworkResumed` ; une déconnexion est classée **en cours de partie** ou **fin de partie** (un autre client avait déjà
quitté : l'attente de 2 à 8 s avant la fin est alors normale). Cellules (delay = par sens, RTT = 2 ×) : A RTT 120 ± 20 ms perte 1 %, B RTT 120 ± 20 ms perte 3 %, C RTT 200 ± 20 ms perte 3 %.
Réglages : `timeout2000` (défaut de GGRS avant D87, canal fiable : étiqueté `default` dans les fichiers bruts de la mesure, le code n'avait alors pas d'autre défaut), `timeout5000` / `timeout8000` (`ALACOD_DISCONNECT_TIMEOUT_MS`, canal fiable), `unreliable` (`ALACOD_P2P_CHANNEL=unreliable`, 2 s). 31 essais ; charge CPU de la machine 28 à 77 % en moyenne par essai
(b6 en parallèle : conditions réelles d'un poste partagé, et donc dites).

### 5.2 Résultats

| Cellule | Réglage | Essais (valides) | Interruptions | Silences < 1 s / 1–1,5 s / 1,5–2,5 s / 2,5–4 s | Silence max repris | Session perdue en cours de partie | Fin de partie (pair parti) | CPU moyen |
|---|---|---|---|---|---|---|---|---|
| A  120 ms ±20, 1 % | timeout2000 | 3 (3) | 8 | 1 / 6 / 0 / 0 | 1174 ms | **0 / 3** | 1 | 76 % |
| A  120 ms ±20, 1 % | timeout5000 | 2 (2) | 2 | 0 / 2 / 0 / 0 | 1153 ms | **0 / 2** | 0 | 56 % |
| A  120 ms ±20, 1 % | unreliable | 2 (2) | 3 | 0 / 3 / 0 / 0 | 1150 ms | **0 / 2** | 0 | 34 % |
| B  120 ms ±20, 3 % | timeout2000 | 3 (3) | 46 | 8 / 36 / 0 / 0 | 1252 ms | **2 / 3** | 0 | 48 % |
| B  120 ms ±20, 3 % | timeout5000 | 2 (2) | 56 | 6 / 48 / 0 / 2 | 3161 ms | **0 / 2** | 0 | 74 % |
| B  120 ms ±20, 3 % | unreliable | 2 (2) | 24 | 4 / 18 / 0 / 0 | 1273 ms | **1 / 2** | 0 | 48 % |
| C  200 ms ±20, 3 % | timeout2000 | 5 (5) | 100 | 26 / 71 / 1 / 0 | 1670 ms | **2 / 5** | 0 | 44 % |
| C  200 ms ±20, 3 % | timeout5000 | 6 (5) | 124 | 43 / 77 / 0 / 3 | 3283 ms | **1 / 5** | 0 | 62 % |
| C  200 ms ±20, 3 % | unreliable | 2 (2) | 48 | 13 / 34 / 0 / 0 | 1382 ms | **1 / 2** | 0 | 67 % |
| C  200 ms ±20, 3 % | timeout8000 | 4 (4) | 119 | 35 / 80 / 0 / 3 | 3432 ms | **0 / 4** | 1 | 40 % |

Un essai de la cellule C n'a jamais établi la connexion (§4.3) ; il n'est pas compté « valide ». Essais un par un (liste complète, `d87-N` = première série, `d87b-N` = seconde) :

| Essai | Cellule | Réglage | Interruptions (c0 / c1) | Silence max reprise (ms) | Déconnexion | CPU moyen | Resimulées |
|---|---|---|---|---|---|---|---|
| d87-1 | A (120 ms ±20, 1 %) | timeout2000 | 1/2 | 1174 | aucune | 76 % | 2648/2906 |
| d87-2 | A (120 ms ±20, 1 %) | timeout2000 | 0/2 | 1123 | fin de partie (pair parti) | 76 % | 2528/2774 |
| d87-3 | A (120 ms ±20, 1 %) | timeout2000 | 2/1 | 1163 | aucune | 77 % | 2745/2650 |
| d87-4 | A (120 ms ±20, 1 %) | timeout5000 | 1/0 | 1115 | aucune | 67 % | 2908/2972 |
| d87-5 | A (120 ms ±20, 1 %) | timeout5000 | 0/1 | 1153 | aucune | 46 % | 3097/3012 |
| d87-6 | A (120 ms ±20, 1 %) | unreliable | 1/0 | 1125 | aucune | 34 % | 3130/3119 |
| d87-7 | A (120 ms ±20, 1 %) | unreliable | 1/1 | 1150 | aucune | 33 % | 3222/3316 |
| d87-8 | B (120 ms ±20, 3 %) | timeout2000 | 5/5 | 1192 | **session perdue en cours de partie** | 28 % | 2382 |
| d87-9 | B (120 ms ±20, 3 %) | timeout2000 | 13/13 | 1214 | aucune | 44 % | 4736/4695 |
| d87-10 | B (120 ms ±20, 3 %) | timeout2000 | 6/4 | 1252 | **session perdue en cours de partie** | 71 % | 1736 |
| d87-11 | B (120 ms ±20, 3 %) | timeout5000 | 14/17 | 3161 | aucune | 77 % | 3319/3386 |
| d87-12 | B (120 ms ±20, 3 %) | timeout5000 | 13/12 | 1491 | aucune | 71 % | 3262/3248 |
| d87-13 | B (120 ms ±20, 3 %) | unreliable | 6/9 | 1273 | aucune | 51 % | 2163/2535 |
| d87-14 | B (120 ms ±20, 3 %) | unreliable | 5/4 | 1190 | **session perdue en cours de partie** | 46 % | 812 |
| d87-15 | C (200 ms ±20, 3 %) | timeout2000 | 8/13 | 1290 | aucune | 52 % | 3896/3783 |
| d87-16 | C (200 ms ±20, 3 %) | timeout2000 | 8/7 | 1375 | **session perdue en cours de partie** | 50 % | 2721 |
| d87-17 | C (200 ms ±20, 3 %) | timeout2000 | 14/14 | 1333 | aucune | 51 % | 3600/3992 |
| d87-18 | C (200 ms ±20, 3 %) | timeout5000 | 2/1 | 1280 | **session perdue en cours de partie** | 61 % | 409 |
| d87-19 | C (200 ms ±20, 3 %) | timeout5000 | 9/15 | 3283 | aucune | 67 % | 2983/3232 |
| d87-20 | C (200 ms ±20, 3 %) | unreliable | 11/16 | 1319 | aucune | 67 % | 2910/3305 |
| d87-21 | C (200 ms ±20, 3 %) | unreliable | 10/11 | 1382 | **session perdue en cours de partie** | 67 % | 2884 |
| d87b-1 | C (200 ms ±20, 3 %) | timeout5000 | — | — | **connexion jamais établie** (900 s, D109) | 65 % | — |
| d87b-2 | C (200 ms ±20, 3 %) | timeout5000 | 19/15 | 1374 | aucune | 57 % | 3648/3766 |
| d87b-3 | C (200 ms ±20, 3 %) | timeout5000 | 20/19 | 3072 | aucune | 75 % | 3184/3727 |
| d87b-4 | C (200 ms ±20, 3 %) | timeout5000 | 13/11 | 1292 | aucune | 52 % | 3440/3816 |
| d87b-5 | C (200 ms ±20, 3 %) | timeout8000 | 14/18 | 3305 | aucune | 49 % | 4094/3408 |
| d87b-6 | C (200 ms ±20, 3 %) | timeout8000 | 19/16 | 1320 | aucune | 39 % | 3291/3170 |
| d87b-7 | C (200 ms ±20, 3 %) | timeout8000 | 12/10 | 1324 | aucune | 38 % | 3747/3830 |
| d87b-8 | C (200 ms ±20, 3 %) | timeout8000 | 12/18 | 3432 | fin de partie (pair parti) | 36 % | 4454/5128 |
| d87b-9 | C (200 ms ±20, 3 %) | timeout2000 | 3/2 | 1298 | **session perdue en cours de partie** | 28 % | 1689 |
| d87b-10 | C (200 ms ±20, 3 %) | timeout2000 | 16/15 | 1670 | aucune | 37 % | 4373/4549 |

### 5.3 Lecture

1. **Les silences ont trois modes.** Sur 520 silences repris : 136 de 0,5 à 1 s (dont 110 de 0,5 à 0,7 s), 375 de 1,0 à 1,5 s (1,1 à 1,4 s pour l'essentiel), **8 de 3,0 à 3,4 s** (uniquement quand le délai de déconnexion
   le permettait), un seul entre les deux (1 670 ms). `webrtc-sctp` 0.17.2 fixe `RTO_MIN` = 1 000 ms (`rtx_timer.rs:10`) et `RTO_INITIAL` = 3 000 ms : [INFERENCE, rien n'instrumente SCTP] les silences de 0,5 à 0,7 s seraient une
   retransmission rapide, ceux de 1,0 à 1,5 s une retransmission après `RTO_MIN`, ceux de 3 s une deuxième retransmission après 2 s de plus (1 + 2 = 3 s). Une session GGRS de 2 s ne tient pas ce troisième mode.
2. **La charge CPU de ce poste ne l'explique pas.** Des sessions sont perdues à 28 % de CPU (`d87-8`, `d87b-9`) et tenues à 77 % (`d87-11`) ; les silences d'un client ne coïncident qu'en partie avec ceux de l'autre (5 interruptions sur 13
   de `d87-9` démarrent à moins de 0,5 s d'une interruption de l'autre client) : plutôt des silences d'un **sens** de transmission que des pauses de la machine. Une charge externe extrême (8 à 11 sur 8 cœurs, m2-p2p-run-complete §3.5)
   déconnectait déjà des pairs sans netem : non rejouée ici, elle reste une cause possible.
3. **Le canal non fiable ne les supprime pas.** Mêmes silences de 1,1 à 1,4 s à 1 % comme à 3 %, et 2 sessions perdues sur 4 à 3 % : [INFERENCE] l'arrêt est dans l'association SCTP elle-même (cwnd, minuteries), pas dans l'ordre de livraison du canal.
4. **Le délai de déconnexion règle le problème, en grande partie.** À 3 % de perte par sens (B et C ensemble) : session perdue en cours de partie **4 fois sur 8 à 2 s, 2 sur 4 non fiable, 1 sur 7 à 5 s, 0 sur 4 à 8 s** ; à 1 % : 0 sur 7 tous réglages.
   Deux essais sur 31 (`d87-2` à 2 s, `d87b-8` à 8 s) finissent autrement : le client 1 attend la fin du délai un pair qui a fini et quitté, puis valide ses dernières frames avec les inputs par défaut du pair parti ;
   les dernières lignes de sa trace (lignes 4 496 et 4 498) diffèrent et `p2p-stress.sh` les compte en échec. Une course de fin de partie, pas une perte de session ; indépendante du délai (aucun cas sur les 7 essais à 5 s). Le seul échec à 5 s (`d87-18`, 5 096 ms de silence, frame ~300) est un silence de plus de 5 s ([INFERENCE] : trois retransmissions de suite, non instrumenté).
5. **Coût d'un délai plus long.** Un pair réellement parti est constaté après 5 s au lieu de 2 s (`p2p-disconnect.sh`, SIGKILL du client 1 après 40 s, délai 5 s : 4,56 s entre `NetworkInterrupted` et `Disconnected` (silence total 5,06 s), survivant sans panique ni `DesyncDetected`, 4 499 lignes de trace) ; pendant ce temps la partie attend la fin de la fenêtre de prédiction, rien d'autre. 8 s serait la limite d'un tel système mais retarde d'autant
   chaque départ réel (la fin de partie d'`d87b-8` attend 8 s) : non retenu avec 4 essais.
6. **Le `DesyncDetected` suit la déconnexion** : dans 6 des 7 sessions perdues en cours de partie, le pair qui n'a pas déconnecté reçoit `DesyncDetected` 0,08 à 0,18 s après le `Disconnected` de l'autre
   (`d87-8` : frame 1970 ; `d87-10` : 2070 ; `d87-16` : 3070 ; `d87-18` : 320 ; `d87-21` : 3900 ; `d87b-9` : 1570) ; dans la septième (`d87-14`), les deux pairs se sont déconnectés à 7 s d'écart et le desync (frame 1550) tombe
   0,02 s après le second. Aucun `DesyncDetected` sans déconnexion préalable sur les 31 essais, et la première ligne différente de `d87-2` (fin de partie) est la 4 496 : [INFERENCE] une conséquence de la session coupée, pas un écart de simulation.

### 5.4 Décision

`p2p::DISCONNECT_TIMEOUT` = **5 s** (`SessionBuilder::with_disconnect_timeout`, défaut de GGRS : 2 s). Le canal reste fiable. Les deux variables d'essai (`ALACOD_DISCONNECT_TIMEOUT_MS`, `ALACOD_P2P_CHANNEL`) restent
pour rejouer la matrice. Justification : 4/8 → 1/7 sessions perdues à 3 % de perte par sens, sans effet mesuré à 1 %, pour 3 s de plus avant de constater un vrai départ.
**Limite de la comparaison** : 4/8 contre 1/7 n'est **pas significatif** (test exact de Fisher bilatéral, p ≈ 0,28) ; ce n'est pas lui qui fonde la décision. Le fondement est le **contrefactuel** : avec 5 s, trois essais ont repris après plus de 2 s de silence
(`d87-11` 3 161 ms, `d87-19` 3 283 ms, `d87b-3` 3 072 ms) et le même silence aurait déconnecté la session à 2 s ; avec `d87-18` (le seul à avoir quand même perdu la session à 5 s, 5 096 ms, que 2 s aurait aussi perdue), cela fait **4 essais sur 7
à 5 s** qui auraient été perdus à 2 s. Un silence de 3,0 à 3,4 s dépasse 2 s par construction : le délai de 2 s ne peut pas tenir ce mode, 5 s le tient (8 occurrences sur 8 reprises, hors `d87-18`).
Limites : échantillons de 2 à 6 essais par cellule, une exécution par essai, perte uniforme de netem (pas de pertes en rafale réelles), une seule machine partagée ; D109 reste.

## 6. Changements de code et de scripts

- `crates/game/src/jjrs/p2p.rs` : `DISCONNECT_TIMEOUT` (5 s) appliqué à la session (`system_after_map_loaded`), `ALACOD_DISCONNECT_TIMEOUT_MS`, `ALACOD_P2P_CHANNEL`. Session seulement : aucun état rollback, aucune trace de scénario.
- `scripts/p2p-machine.sh` : `STRESS` / `SUSPEND_MS` / `RUN_MS`, `MIN_RESIMULATED`.
- `scripts/p2p-d87.sh`, `scripts/p2p-silences.py` (nouveaux).
- Docs : `CLAUDE.md`, `docs/conventions.md` §57, `docs/etat-engine.md` (contrat de sortie points 3, 4, 5), `docs/taches.md` (ligne 7), `docs/taches/m2-p2p-run-complete.md`, `docs/taches/dettes.md`
  (D73 confirmée, D87 fermée, D108 et D109 ouvertes).

## 7. Vérification

Sur l'arbre livré (`4a1f1220` plus les documents) :

- `make test_scenarios_par SHARDS=3` sans BLESS : **649 scénarios, 3 tranches sur 3 vertes, 1 286 s**, aucun `.trace` modifié (`git status` propre hors documents). Une première passe (même code, commit `beecb129` posé pendant la
  passe) a vu ses trois tranches vertes mais l'étape de fusion des métriques échouer, le nom du dossier de métriques ayant changé avec le commit : rejouée sans toucher à l'arbre.
- `make test_crates` : **1 228 tests passés, 0 échec, 14 ignorés** (80 suites).
- `make test_tools` : 25 tests, OK. `cargo fmt --all -- --check` : propre.
- Fumée du changement de délai de déconnexion : `p2p-disconnect.sh` (SIGKILL du client 1 après 40 s, `disc2`) : survivant code 0, 4 499 lignes, 0 `DesyncDetected`, `NetworkInterrupted` puis `Disconnected` 4,56 s plus tard ; les trois essais du §3.4 sur le commit livré.
- Relecture indépendante (deux relecteurs, code et chiffres) : 13 constats chiffres et textes, 8 sur les scripts ; corrigés : classement fin de partie / en cours de partie de `p2p-silences.py` et pairs multiples, variante `timeout2000`
  de `p2p-d87.sh` (le défaut du jeu vaut maintenant 5 s), inférences SCTP marquées, D109 sur 24 démarrages, sources de stress, étage des essais, horloge de `disc2`, phrases d'état du dépôt.
- Aucun test permanent ajouté : le seul changement de comportement est une constante de session (5 s) dont l'effet n'est observable que sur le réseau ; il est prouvé par les essais ci-dessus.

## 8. Limites

Une seule paire de machines (VPS et un Mac M1) et un RTT de 27 à 41 ms ; natif seulement ; un jeu de graines (10 pour gungeon, 123456 pour isaac) ; le Mac était chargé par d'autres workers
pendant tous les essais (charge 9 à 85) et n'a pas déconnecté une seule fois à RTT réel, ce qui montre aussi que la session tient une machine chargée ; la revue humaine (contrat de sortie, point 7)
reste à faire ; D109 et D108 ouvertes.
