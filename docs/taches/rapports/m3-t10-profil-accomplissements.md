# Rapport m3-t10-profil-accomplissements (b6)

Conventions : `docs/conventions.md` §75. Branche `m3-t10-profil-accomplissements`, fusionnée avec `origin/main` `f18f5945`
(M3-T11a, T11b, T11c, T16, lisibilité gungeon) ; conflits triviaux (`content/src/ui.rs` et `hud_model.rs` : sources HUD `achievements` + `legend`, `hearts`, `coins`, `item_banner` ; `CLAUDE.md` ;
`isaac/game.ron` : dossiers `unlocks`/`achievements` + `statuses`/`synergies`). Après la seconde fusion (T11c : gardien à trois phases), `achievement_unlocks` passe à 2 190 frames (victoire vers f2110, comme `grid_post_boss`).

## 1. Fait

| Point de la fiche | Livré |
|---|---|
| 1. Évaluation hors simulation | `game::profile::write_achievements_at_run_end` (`Update`, jamais `GgrsSchedule`) : à `Ended`, une fois par run, une fois la **fin confirmée** (`summary_is_final`, `ConfirmedFrameCount`) ; un abandon est définitif tout de suite. `grant_achievements` : `meta::achieved` sur chaque accomplissement du jeu (`meta::Achievements`, table du registre), les nouveaux ajoutent id + `unlocks` au profil de chaque joueur de la session (`LocalPlayers`), écriture atomique. `PROFILE_VERSION` 2 : `Profile::achievements` (`#[serde(default)]`), test de migration v1 → v2. |
| Faits de run | `RunHistory { bosses_defeated, items_held }` écrit par la simulation (`boss_defeated_system` : nom du personnage de `GgrsNetId`, seulement tant que la run se joue ; `record_held_items` : union des inventaires, dernière valeur gardée après la mort). **Rollback sans checksum** (nouveau `rollback_and_trace_resource_no_checksum`) : suit les rollbacks (une prédiction ratée ne laisse rien) et ne bouge aucun checksum ni aucune trace ; la simulation ne le lit jamais. Étages, temps, issue (défaite comprise) : déjà dans `RunSummary`. Remis à zéro à la relance. |
| 2. Entrée dans la run suivante | `RunUnlocks` (§51), sans autre chemin. Le marchand du hub et `alacod-profile unlock` ne vendent pas un déblocage qu'un accomplissement accorde (`Registry::achievement_unlocks`). En ligne : règle de l'hôte (§75 : chaque pair écrit ses joueurs locaux, la run suivante lit le profil de l'hôte). |
| Présentation | Source HUD `achievements` (accomplissements de cette fin de run, joueur affiché) ; l'écran de fin les liste sous le résumé (`AchievementsEarned`, hors rollback). `AchievementDef::name` (optionnel) : nom affiché. |
| 3. Contenu `isaac` | Quatre accomplissements (tableau ci-dessous). Les objets débloqués sont des objets **neufs** (`medaille`, `couronne`, `bague`) ajoutés aux pools `boss` et `tresor` **et** cachés par un déblocage `Item` : profil vide, un pool dont les candidats d'une entrée sont tous cachés ne compte pas l'entrée, le tirage est celui d'avant. Le lint d'`isaac` perd l'avertissement « aucun accomplissement dont le déblocage donne un objet… » (26 → 25 sur la fusion). |
| Contenu `gungeon` | Quatre accomplissements dont les déblocages sont des entrées `Loot` (aucun objet retiré d'un pool). |
| 4. Scénarios et tests | `achievement_unlocks` (isaac), `achievement_not_met` (isaac, défaite), `crates/scenario/tests/achievements.rs` (deux runs enchaînées, `RunUnlocks` puis pools à graine égale), tests unitaires `meta` (migration, accord unique, refus d'écraser un profil illisible/trop récent), `game::profile` (faits, confirmation, noms, objets tenus). Attentes : `ProfileHas` gagne `achievement`, `lacks_unlocks`, `lacks_achievements`. |

### Accomplissements de contenu

| Jeu | Accomplissement | Condition | Déblocage | Ce qui entre |
|---|---|---|---|---|
| isaac | `gardien_vaincu` | `BossDefeated("gardien")` | `medaille_libre` | `medaille` (+10 % dégâts) dans le pool `boss` |
| isaac | `run_finie` | `Victory` | `couronne_libre` | `couronne` (+15 % portée) dans `tresor` |
| isaac | `second_etage` | `FloorAtLeast(1)` | `bague_libre` | `bague` (+10 % cadence) dans `tresor` |
| isaac | `semelles_aux_pieds` | `ItemHeld("semelles_lestees")` | `bombes_en_reserve` | la bombe (poids 3) dans le pool `salle` (`Loot`) |
| gungeon | `gatling_vaincu` | `BossDefeated("gatling")` | `butin_gatling` | `parchemin` dans `coffre_etage1` et `coffre_etage2` |
| gungeon | `gungeon_termine` | `Victory` | `butin_victoire` | `sablier` dans `coffre_etage2` |
| gungeon | `second_etage` | `FloorAtLeast(1)` | `butin_second_etage` | `trophee` dans `coffre_etage2` |
| gungeon | `bottes_aux_pieds` | `ItemHeld("bottes")` | `butin_bottes` | `cartouchiere` dans `coffre_etage1` et `coffre_etage2` |

## 2. Décisions et écarts

- **Rollback sans checksum plutôt que « hors rollback » ou « neutre »** : l'historique est écrit dans `GgrsSchedule` ; hors rollback, une
  prédiction ratée (boss « vaincu » dans une frame resimulée autrement) accorderait à tort un accomplissement écrit sur disque ;
  neutre, la ressource bougerait dès qu'un boss meurt (`gungeon_boss`, `isaac_*` : traces). Contrepartie : un desync de cet historique
  ne serait pas détecté, sans effet sur la simulation.
- **Objets neufs plutôt que retirer des objets existants** : retirer `oeil_vif`, `lunette_longue`… d'un pool décalait les tirages de
  tous les scénarios `isaac` (pool à épuisement tiré à la génération, `PoolState` dès la frame 0 : 44 traces). La fiche dit « un objet
  retiré du pool de base » : c'est bien le cas (déblocage `Item`), l'objet n'existait simplement pas avant.
- **Q10 (transformation)** : pas de réponse, donc pas d'accomplissement « tenir une transformation » ; `semelles_aux_pieds` (tenir un
  objet) le remplace et couvre `ItemHeld`.
- **Fin confirmée** : un accomplissement ne se reprend pas sur disque, une fin vue pendant une prédiction peut être défaite
  (l'écriture d'essence de §38 ne l'attend pas : inchangé). Quitter l'écran de fin avant la confirmation perd l'évaluation (§75).
- **`ended_handles` via `LocalPlayers`** (revue) : l'ancienne règle (joueurs locaux vivants, sinon tous) perdait le profil d'un joueur
  mort et, en ligne, écrivait les profils distants quand tous les locaux étaient morts. Elle sert aussi l'essence : un joueur mort la
  reçoit maintenant (aucune attente existante ne le contredit).
- **`meta::save` refuse d'écraser un profil illisible ou `TooNew`** (revue) : le passage en v2 rend le cas courant (build v1 relancé
  après un build v2) et §38 disait « jamais écrasé » sans que ce soit vrai.
- **Pas de `RunFacts` par joueur** : `ItemHeld` reste l'union des joueurs (contrat de §64).

## 3. Vérifié (HEAD = commit LIVRÉ, fusion avec `origin/main` `f18f5945`)

- `make test_scenarios_par SHARDS=3` sans BLESS, sur la fusion finale (après les correctifs de revue et la fusion de T11c/T16) : 3 tranches ok,
  1 374 s. Sans BLESS hors les deux traces nouvelles (écrites à la création, rebénies après chaque fusion avec main : le contenu d'`isaac` avait changé).
- `make test_crates` : 1 181 tests passés, 0 échec (dont `meta` +3 et `game::profile` +6 de cette tâche).
- `make lint` ×5 : zombies, throne, testbed (107 avertissements, inchangé : aucun contenu touché), gungeon (1, le pool `boss` jamais
  tiré : T12), isaac (24 ; sur main avant la tâche, l'avertissement « accomplissement » en faisait partie, il n'y est plus : `lint games/isaac | grep accomplissement` = 0 ligne) ; aucune erreur.
- `make gen GAME=…` ×5 sur la fusion finale : code 0, `git status` sans diff sur les scénarios générés.
- `cargo fmt --all -- --check`, `check-forbidden.sh`, `check-rollback-registration.sh`, `make check_sim_boundary`, `make test_tools` (25 tests) : OK.
- Fuzz `ALACOD_FUZZ=0:8` (`-- --ignored`, avant la dernière fusion) : 1 test passé (467 s).
- `crates/scenario/tests/achievements.rs` : 1 passé (deux runs enchaînées).
- **p2p-stress** `isaac`, 3 000 frames, `/tmp/inputs_b6.ron` (`p2p-stress-inputs.py 3000`), deux clients : codes de sortie 0/0,
  0 `DesyncDetected`, **4 172 frames resimulées** (client 0 ; 6 pour le client 1), traces identiques (2 999 lignes contiguës). Docker,
  `MATCHBOX_PORT=3596`, `COMPOSE_PROJECT_NAME=b6`. Preuve de ce qu'elle couvre : `RunHistory` (nouvelle ressource rollback) est
  sauvegardée et restaurée sous resimulation sans divergence ; la run n'y atteint pas sa fin (pas d'accomplissement évalué en p2p :
  voir § 5).
- `cargo check -p game --features ui_hui` : la bibliothèque compile (donc `ui::game_over`), le test `l_ecran_de_fin_liste_les_accomplissements`
  (feature `ui_hui`) **n'a pas pu être exécuté** : la cible de test de `game` avec `ui_hui` ne compile pas sur main (`ui/screens/hui.rs`
  : champs `merchant`, `merchant_message`, `merchant_seen` manquants dans ses tests, antérieur à cette tâche).

## 4. Traces bougées

**Aucune.** Aucun `.trace` existant modifié (suite complète sans BLESS, `isaac` compris : le contenu ajouté est invisible tant que
le profil est vide). Traces ajoutées : `achievement_unlocks`, `achievement_not_met`. Preuve §5 : l'argument est structurel
(`draw_exhausting` ignore une entrée dont tous les candidats sont cachés) et vérifié par les 44+ traces `isaac` inchangées de la suite.

## 5. Non vérifié / dettes

- **Évaluation en p2p réel non jouée** : le stress ne finit pas la run (le boss n'y meurt pas), donc la porte `ConfirmedFrameCount` et la
  règle de l'hôte ne sont éprouvées que par le code (tests unitaires de `summary_is_final`, `grant_achievements`) et par le synctest du
  scénario `achievement_unlocks` (confirmation = frame courante − distance de contrôle). « p2p à deux profils différents rejoué par
  l'orchestrateur » (fiche) : à faire à la revue, avec `ALACOD_PROFILE_DIR` distinct par client.
- **Test `game_over`** : écrit mais non exécutable ici (voir § 3, `ui_hui`).
- **Quitter l'écran de fin avant la confirmation perd l'évaluation** (quelques frames en local, un aller-retour réseau en p2p) ;
  un abandon, lui, est évalué. Dit au §75.
- **Pas d'écran des accomplissements** (hors périmètre) : le profil les garde, l'écran de fin ne montre que ceux de la fin de run.
- **`ItemHeld` = union des joueurs** ; un accomplissement par joueur demanderait un `RunFacts` par joueur.
- **Q10** sans réponse (transformation).
- **Pas de session cloud** : le rapport vient de la machine de b6 (Docker disponible, p2p-stress joué).
- Environnement : `~/.cargo/bin/rustup` a disparu en cours de tâche (le dossier a été vidé à 12:55, hors de mon fait) : les commandes
  suivantes ont utilisé `~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin` directement. Disque partagé à 21–24 Go libres en fin
  de tâche (trois targets de 60 Go).

## 6. Relecture indépendante

Faite par un sous-agent `reviewer` (read-only) avant le commit LIVRÉ. Aucun P1. Trois P2 corrigés : (1) un joueur mort perdait ses
accomplissements et, en ligne, un pair écrivait les profils distants → `LocalPlayers` ; (2) un abandon n'évaluait jamais → définitif sans
confirmation et système ordonné après `apply_run_request_system` (`.chain()` après l'essence) ; (3) un profil `TooNew` ou illisible était
écrasé → `meta::save` refuse (test). P3 corrigés : boss tué après `Ended` (garde `is_playing`), coût par frame de `record_held_items`
(comparaison avant clone), trois affirmations fausses de §75 (ordre, « sans session », `TooNew`). Laissés : l'ordre de
`record_held_items` dans son set (correct tant qu'aucun système du set n'écrit `Inventory`), la source HUD vide quand le joueur affiché
est mort (dit au §75).

## 7. Journal (à ajouter sur `main` après le merge, README §6)

`| 2026-10-17 | M3-T10 (accomplissements : RunHistory rollback sans checksum, évaluation à Ended après confirmation, profil v2, HUD/écran de fin, objets neufs cachés dans isaac, §75) | m3-t10-profil-accomplissements | b6 | mergée ; suite sans trace bougée, test_crates 1 181, lint/gen ×5, fuzz 0:8, p2p-stress isaac 4 172 frames resimulées ; non fait : évaluation en p2p réel à deux profils |`

## 8. Revue 2 (indépendante) : À RELIRE fermé

1. P2 `items_held` : `RunHistory::held_by_player` (dernière valeur par joueur, union à l'évaluation) ; tests à deux joueurs dont un mort.
2. P3 boss tué à la frame de la défaite : `run_counts_at` (`Ended { at_frame == frame }`) pour boss et objets ; test.
3. P3 §75 : abandon évalué sans confirmation, raison et risque écrits.
4. Docs : `RunHistory` dans §10 « Exclusions assumées du checksum », règle de `rollback_and_trace_resource_no_checksum` dans la checklist de `CLAUDE.md` et le message de `check-rollback-registration.sh`.
5. Test `achievements.rs` : `PoolState::assign` sur la graine de la run, 400 emplacements ; seuls les emplacements débloqués changent, vers l'objet débloqué.
6. Test `ui_hui` : corrigé (`..Default::default()` dans `ui/screens/hui.rs`, pas de dette) ; `game_over` : 2 tests passés.
7. `cargo test -p game -p meta -p scenario --tests` (hors suite `scenarios`) vert ; `achievement_unlocks` et `achievement_not_met` sans BLESS (traces inchangées) ; fmt, `check-rollback-registration.sh` OK.
