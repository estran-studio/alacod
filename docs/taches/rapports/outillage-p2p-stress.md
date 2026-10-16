# outillage-p2p-stress : p2p à rollbacks réels, outillé

Branche `outillage-p2p-stress` (depuis `origin/main` 72ccb26f), worker b1, 2026-10-16.

## 1. Pourquoi

Les p2p « à deux clients » de M3-T6, M3-T4 et M3-T7 n'avaient aucun ou presque aucun rollback réel (compteur du journal
« frames resimulées après rollback » : 0 à 3) ; les lignes `ggrs{f=N}` en double prises pour des resimulations étaient des
lignes émises une fois par exemplaire (voir `m3-t2-effets-v3.md`, section p2p). Seule la recette de M3-T2 forçait des
rollbacks : `ALACOD_INPUT_DELAY=0`, entrées qui changent toutes les 1 à 3 frames, client 1 suspendu/repris (`SIGSTOP`/`SIGCONT`).

## 2. Fait

- **`scripts/p2p-stress.sh <jeu> <script-inputs.ron> [frames]`** : la recette de T2 en un script (signaling docker démarré et
  arrêté, compilation `headless`, binaire lancé directement pour que `SIGSTOP` atteigne le client et pas `cargo`, deux clients,
  `ALACOD_INPUT=script:`, `ALACOD_INPUT_DELAY=0`, suspension du client 1 12 ms toutes les 30 ms). Imprime les codes de sortie,
  `DesyncDetected`, **les frames resimulées lues dans le journal du jeu**, le `cmp` des traces et le nombre de lignes. Échoue
  sur : code de sortie non nul, `DesyncDetected`, traces différentes, ou moins de `MIN_RESIMULATED` (1000 par défaut) frames
  resimulées chez le client le plus resimulé.
- **`scripts/p2p-stress-inputs.py FRAMES [--seed] [--pool] [--hold Bouton:de:à]`** : génère le script d'inputs (deux joueurs, segments
  de 1 à 3 frames, graine fixe) ; `--hold` maintient un bouton (Interaction, Bomb, Fire…) sur une plage.
- **`docs/taches/README.md` §4** : cette recette remplace l'ancienne (sans stress), obligatoire quand la simulation, la session
  ou l'état rollback change ; elle dit aussi qu'un système propre à une situation demande un montage qui l'y met.

## 3. Correctif des dernières frames (point 2)

**Cause** : à `ALACOD_EXIT_AT_FRAME`, `write_trace_at_exit_frame` écrivait aussitôt les frames enregistrées ; le client en avance sur
son pair n'en avait que des versions prédites pour les ~8 dernières (T2 : 8 lignes différentes), qu'une resimulation aurait corrigées.

**Correctif** (`crates/game/src/state_trace.rs`) : `confirmation` (pure) compare la dernière frame à écrire à
`bevy_ggrs::ConfirmedFrameCount`. Tant qu'elle n'est pas confirmée, le système renvoie sans écrire (le jeu continue, les
inputs du pair arrivent, la confirmation suit) ; au bout de `CONFIRMATION_TIMEOUT` (10 s : pair parti, session perdue), il écrit
seulement les frames confirmées et avertit (`trace d'état tronquée…`). S'applique aussi à l'écriture de `-g1` du restart (D14).
Le client qui attend continue d'avancer : la trace s'arrête donc toujours à `exit_at_frame - 1` (`exit_at_frame - 2` inclus, comme
avant : la dernière frame n'a jamais été enregistrée à l'arrêt) ; sans ce plafond le client en avance écrivait une ligne de
plus que le pair (constaté au premier essai : 3000 lignes contre 2999, préfixe identique).
Test unitaire `attend_la_confirmation_des_dernieres_frames` (attente, écriture complète, expiration, frames au-delà de `end`).
Aucune trace de scénario ne dépend de ce chemin (le runner de scénarios n'écrit pas par lui) : voir les résultats du § 5.

## 4. Stress sur `main` (+ ce correctif), 3000 frames, trois montages temporaires non commités

Binaire `testbed`, `ALACOD_INPUT_DELAY=0`, `MATCHBOX_PORT=3556`, `COMPOSE_PROJECT_NAME=b1`, montages annulés par `git checkout`
(+ suppression des cartes `p2p_*.ldtk`). Chaque run : `p2p-stress.sh testbed <script> 3000`, « p2p-stress : OK », codes de sortie 0/0,
`DesyncDetected` 0.

| Montage | Frames resimulées (client 0 / client 1) | `cmp` des traces | Écarts |
|---|---|---|---|
| (a) familiers : système `GgrsSchedule` qui pose `compagnon_actif`/`essaim`/`orbital` aux pieds de chaque joueur toutes les 120 frames (dès f20, rotation), `Interaction` maintenue (inputs `--hold Interaction:1:3000`) | 4690 / 33 | identiques, 2999 lignes | aucun. Journal du client 0 : 244 `familiar_spawn`, 88 `familiar_lost`, 157 `familiar_shot` |
| (b) bombes + rocher + mur secret : carte `p2p_t4` (`salles_secret.ldtk` + couche `Rocks` : 4 rochers dont 1 à butin ; salle secrète à l'est), `bomb: Some((fuse_frames: 20, radius: "40.0", damage: "10.0", friendly_fire: Never))`, `starting_consumables: {"bomb": 30}`, `Bomb` toutes les 100 frames dès f25, `Right` maintenu f20–140 | 7333 / 2 | identiques, 2999 lignes | aucun. Journal : 121 `bomb_place`, 165 `explosion`, 6 `rock_broken`, 3 `rock_loot`, 1 `secret_wall_open` (f145) |
| (c) champions : carte `arena_variantes` avec 4 `elite_explosif` (variante `explosif`) et 4 `elite_divise` (variante `divise`) forcés par `CharacterSpawn`, `Fire` maintenu | 6715 / 0 | identiques, 2999 lignes | aucun. Journal : 31 `champion_death`, 45 `champion_split`, 15 `explosion`, 16 `powerup_drop` |

Écart au cahier des charges : (b) donne 30 bombes et non 8, pour qu'il en reste sur 3000 frames (huit auraient été posées dans les
800 premières). Les ~8 dernières lignes ne diffèrent plus (avant le correctif T2 en avait 8 de différentes ; le premier essai
du montage (a) donnait préfixe identique et une ligne de plus, corrigé par le plafond ci-dessus).

Un premier essai de (c) a échoué à l'ouverture (« RON invalide : game.ron ») : erreur de mon montage (`open(f,'w').write(open(f).read())`
tronque le fichier avant la lecture), pas du jeu.

## 5. Vérification

- `make test_scenarios_par SHARDS=3` sans BLESS : **482 scénarios, trois tranches ok**, 184 s, aucune trace bougée (`git status` : aucun `.trace` modifié).
- `make test_crates` : tous les paquets verts sauf `-p game --lib` au premier passage : un test de ce commit (`attend_la_confirmation…`, dernier cas)
  avait une erreur dans mes valeurs (frame 599 non confirmée à 597) ; corrigé, `cargo test -p game --lib` : **193 passés, 0 échec**. Le reste du
  workspace n'a pas été rejoué après ce correctif de test (il ne touche que ce module de test).
- `make test_tools` : 17 tests ok. `cargo fmt --all -- --check` : ok.
- Chemin local (synctest) du correctif : `testbed` headless, `ALACOD_INPUT=neutral`, `ALACOD_EXIT_AT_FRAME=300` : trace de **299 lignes**, aucun
  avertissement de troncature, sortie normale (la confirmation synctest suit de `check_distance` frames).
- p2p : voir le tableau du § 4 (trois montages, `p2p-stress.sh` OK).

## 6. Dettes, non fait

- Le p2p stress de zombies seul (`p2p-stress.sh zombies`) n'a pas été rejoué dans ce passage : la tâche demandait les trois montages testbed.
- Les montages (a) à (c) restent des modifications temporaires : le script n'en garde rien. Un montage permanent (carte et système de pose
  d'objets dans un binaire de test) serait une tâche à part.
- Le compteur « resimulées » est celui du client le plus resimulé : le client 1 (suspendu) n'en a presque pas, le client 0 les porte. Une
  variante qui suspend alternativement les deux clients doublerait la couverture ; non faite.

## 7. Passage de relecture (worker b4)

1. **Trace `-g1` du restart** (`state_trace.rs`) : `WriteFirst` plafonnait à `restart_at` : une ligne de plus si la simulation avançait pendant l'attente
   (600 au lieu de 599). Fonction commune `last_trace_line(at_frame) = at_frame - 1`, utilisée par `-g1` et par la trace de sortie ; test
   `plafond_des_lignes_independant_de_la_simulation` (trace enregistrée jusqu'à 602, tout confirmé : 599 lignes). `scripts/p2p-restart.sh` rejoué
   (docker) : p2p partie 1 et partie 2 identiques (599 lignes), local partie 2 = partie 1 (599).
2. **`p2p-stress-inputs.py`** : segments coupés aux bornes de chaque `--hold` ; maintien exact à la frame près (un maintien d'une frame ne disparaît plus).
   Test `scripts/tests/test_p2p_stress_inputs.py` (dans `make test_tools`) et `--selftest` : `Interaction` sur `[13,14[`, `Dash` sur `[40,77[`, 50 graines.
3. **`p2p-stress.sh`** : chaque trace doit faire `FRAMES-1` lignes (échec sinon, message dédié) ; une déconnexion GGRS (`disconnected`, `Disconnected`,
   `NetworkInterrupted`) dans le journal **avant** la ligne « trace d'état écrite » est un échec (après, le pair qui a fini et quitté est normal : voir
   `jjrs::log_ggrs_events`, qui ne traite le départ du pair comme perte qu'en cas de desync en headless).
4. **Rejeu du montage (b)** avec les scripts corrigés (montage temporaire, annulé : carte `rocks_static` + 2e `PlayerSpawn`, `bomb` 40 px/20 frames,
   30 bombes, `Right` f20–140, `Bomb` toutes les 100 frames dès f25 par `--hold`) : `p2p-stress.sh testbed … 3000` OK, codes 0/0, `DesyncDetected` 0,
   **9886 / 1 frames resimulées**, déconnexions 0, traces **identiques, 2999 lignes = FRAMES-1 contrôlées**. Journal client 0 : 187 `bomb_place`,
   270 `explosion`, 6 `rock_broken` (pas de `secret_wall_open` ni `rock_loot` : carte sans salle secrète, contrairement à l'ancien `p2p_t4`).
   Un premier essai a échoué sur « pas de point de spawn pour le joueur 1 » (carte à un seul spawn) : erreur de montage.
5. Vérifications : `make test_crates` vert (`game --lib` 194), `make test_scenarios_par SHARDS=3` sans BLESS : 482 scénarios, 3 tranches ok, aucune trace
   bougée ; `make test_tools` 18 ok ; fmt ok.
