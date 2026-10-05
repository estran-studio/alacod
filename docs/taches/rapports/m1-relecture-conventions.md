# m1-relecture-conventions — relecture d'ensemble de `docs/conventions.md` et `CLAUDE.md`

## État en cours

- Fait : numérotation continue 1 à 33 (+ « Notes essentielles »), sommaire et note de
  correspondance en tête de `conventions.md`, corrections de fond des 33 sections, 14 références
  de code et de contenu corrigées, bilan `CLAUDE.md` avec diff proposé (ci-dessous). Branche
  depuis origin/main `913f307`, target purgé (46 → 13 Go). Aucune compilation.
- `CLAUDE.md` n'est **pas** modifié sur la branche (règle de session : pas d'édition d'un
  CLAUDE.md à la demande d'une autre session ; accord d'orch) : le diff est à appliquer par
  `git apply` (vérifié avec `git apply --check` sur la tête de branche).

## Méthode

Vérification section par section contre le code de `main` (grep de chaque type, champ, fichier,
cible `make`, variable, scénario, kind et fixture cités), en cinq lectures parallèles (§1–§8,
§8–§18, §19–§26, §27–§33 + Notes, `CLAUDE.md`), puis recoupement à la main des affirmations
chiffrées avant correction (horloges et butin de throne, cron du nightly, `ModifierOp`, liste
des actions refusées dans un effet, reset du restart, pilotage du portail). Chaque remplacement
a été appliqué par un script qui exige que le texte d'origine apparaisse exactement une fois.

## Numérotation

`grep -n '^## ' docs/conventions.md` : 1 à 33, uniques, croissants, continus.

- **Ancien second §9 « Feedback (présentation, T2.13) » → §7**, déplacé entre §6 et §8 (il n'y
  avait pas de §7). §8 Combat, §9 Stats, §10 Blesser une trace gardent leur numéro ; l'ancre
  `{#section7}` du §8 est retirée. C'est la renumérotation qui change le moins de références :
  les autres (§8 → 7, §9 → 8, §10 → 9, Feedback → 10) auraient déplacé les références de trois
  sections de plus, dont les nombreux « §9 » des stats.
- Inventaire : script Python sur `rg -l '§'` (hors `target/`, `docs/taches.md`,
  `docs/plan-engine.md`, rapports), une ligne par occurrence `§N` sur une ligne qui cite
  `conventions` (ou dans `conventions.md`), en écartant les « plan §N », « README §N ».
  **Avant : 432 occurrences** (`../d40/refs_conv.txt`), **après : 516** (dont 34 du sommaire, 9 de
  la note et les renvois ajoutés par les corrections ; `../d40/refs_conv_apres.txt`). Après :
  **aucune référence vers un numéro inexistant**, et contrôle de sens à la main des 47
  occurrences de §7 à §10.
- Références déjà fausses avant cette tâche, corrigées : Stats citées en « §7 » (numérotation de
  M0) et « Blesser une trace » citée en « §8 ».
- **Non réécrits** (décision 1) : fiches et rapports historiques, et `docs/taches/README.md`
  (décision 5). La note en tête de `conventions.md` donne la correspondance. Faux dans
  `docs/taches/README.md` : l. 82 « §8 protocole de preuve » et l. 145 « (protocole
  `docs/conventions.md` §8) » visent l'actuel **§10**.
- `### 4.3 Contrats de M1` n'a ni 4.1 ni 4.2 : gardé tel quel (cinq renvois « §4.3 » dans le
  fichier).

## Commentaires de code et de contenu touchés (numéros de section seulement)

| Fichier | Avant | Après |
|---|---|---|
| `crates/game/src/character/create.rs` (×2, `//`) | §7 (stats) | §9 |
| `crates/game/src/character/enemy/create.rs` (`///`) | §7 | §9 |
| `crates/game/src/character/mod.rs` (`//`) | §7 | §9 |
| `crates/game/src/state_trace.rs` (×2, `///`) | §8 « Blesser une trace » | §10 |
| `crates/scenario/src/runner.rs` (`///` et `//`) | §8 | §10 |
| `crates/scenario/tests/scenarios.rs` (`//`) | §8 | §10 |
| `crates/content/src/feedback.rs`, `crates/game/src/feedback.rs` (`//!`) | §9 et §31 | §7 et §31 |
| `games/{zombies,testbed,throne}/assets/ui/feedback.ron` (commentaire RON) | §9 et §31 | §7 et §31 |
| `games/zombies/README.md` | §8 | §10 |

Des commentaires `///`/`//!` (doc rustdoc) sont touchés : un chiffre dans du texte, sans
compilation ; `cargo check` non lancé (feu vert d'orch non demandé, rien à vérifier par le
compilateur).

## Sections

| § | Titre | Verdict |
|---|---|---|
| 1 | Cartes LDtk | corrigé (7) |
| 2 | Sprites et animations | corrigé (1) |
| 3 | Le dossier de jeu | corrigé (9) |
| 4 | Ajouter un vocabulaire : checklist | corrigé (8) |
| 5 | Commandes make | corrigé (1) |
| 6 | CI lente (nuit) | corrigé (3), écart |
| 7 | Feedback (présentation, T2.13) | corrigé (3) + numéro |
| 8 | Combat : équipes et dégâts | corrigé (3) + ancre |
| 9 | Stats et modificateurs | corrigé (3), écart |
| 10 | Blesser une trace : la preuve | corrigé (2) |
| 11 | Munitions et inventaire d'armes | corrigé (2) |
| 12 | Monnaie et achats | corrigé (1) |
| 13 | État de run et mode `Waves` | corrigé (8) |
| 14 | Power-ups | corrigé (1) |
| 15 | HUD : icônes, noms et sources | corrigé (1) |
| 16 | Projectiles composables | corrigé (5) |
| 17 | Mode `Floors` | corrigé (2) |
| 18 | Équilibrage par joueurs | corrigé (3) |
| 19 | Statuts | corrigé (2) |
| 20 | Patterns, émetteurs et tir ennemi | corrigé (1) |
| 21 | Terrain destructible et cavernes | corrigé (3) |
| 22 | Behaviors composables | corrigé (1) |
| 23 | Horloges et difficulté | corrigé (1) |
| 24 | Bots de validation | corrigé (4) |
| 25 | Variantes et élites | corrigé (1) |
| 26 | Surfaces | OK |
| 27 | Effets v1, jauges et mutations | corrigé (2) |
| 28 | Générateur v1 et placement scripté | corrigé (1) |
| 29 | Le jeu `throne` | corrigé (9) |
| 30 | Écran de mutation et transition | OK (ligne vide avant le titre) |
| 31 | Feedback v1 | OK (ligne vide avant le titre) |
| 32 | HUD throne | OK (ligne vide avant le titre) |
| 33 | Restart en ligne | OK |
| — | Notes essentielles | corrigé (5) : quatre notes déjà ailleurs remplacées par des renvois (§1, §2, §26, `CLAUDE.md`), anglicisme |

Nature des corrections : identifiants déplacés ou renommés (`FrameEvents` dans `sim_core`,
`collider.rs` et `weapons/mod.rs` dans `combat`, `ResolvedEconomyConfig`, `RngStreams`,
`HitFlash`/`CameraShake`), listes incomplètes (20 kinds au lieu de 11, `LintErrorKind::Unsupported`,
six émetteurs de `DamageEvent`, quatre `ModifierOp`, six fixtures de lint sans ligne, systèmes de
projectile), « à venir / non supporté » livrés depuis (`Floors`, restart p2p, bots sur caverne,
écran de choix et HUD des rads, T1.12, `Statuses` posé), valeurs périmées (horloges et butin de
throne, onze ennemis, boss `Large`), mesures caduques remplacées par un renvoi au journal
(throne 19/20, 17/20, 20/20 mesurés quand les trois étages chargeaient `niveau_1`), comportements
décidés depuis dans le code (pilotage du portail de m1-v3-bots-portail, navigation des bots en
`Floors` seulement, nightly sans déclencheur push et sans budgets absolus).

## Écarts à décider

1. **§9** — « Aucun système de `crates/game` ne rappelle `sim_core::modifier::resolve`
   directement » ; `character::variant::variant_health` l'appelle au spawn (santé de départ d'une
   variante, T1.5) — `crates/game/src/character/variant.rs`. Exception notée dans la doc ; à
   décider : documenter l'exception pour de bon, ou passer par `Stats`/`StatReader`.
2. **Notes essentielles (et `docs/taches.md` §1, règle 3)** — « Les seules voies autorisées à
   changer les traces : V1 (simulation), justifié par `BLESS=1` » ; en pratique des traces ont été
   bénies par des tâches hors V1, avec preuve (m1-throne-gen-et-d40 : 2, m1-integration-scenarios :
   32, m1-v3-bots-portail : 7) — journal de `docs/taches.md`. Phrase laissée telle quelle.
3. **§6** — « Artefacts : … Vidéos MP4 » ; l'artefact du nightly ne contient que
   `nightly/<commit>/` (`.github/workflows/nightly.yaml`), alors que `make videos` écrit dans
   `target/videos/<commit>/` (`scripts/nightly.sh` ne les copie pas) : les MP4 ne sont
   probablement pas dans l'artefact. Copier les vidéos, ou retirer la ligne.

## Commentaires de code périmés hors numéros (non touchés, aucun changement de code)

- `crates/game/src/character/health/mod.rs` : « trois émetteurs » de `DamageEvent` (six).
- `crates/game/src/run_state.rs` (~l. 45) : « Restart en p2p : non supporté … `ToLobby` » (livré, D14).
- `crates/run/src/run.rs` (l. 3, 21) : `Floors`/`Campaign` « à venir » (`Floors` livré).
- `crates/sim_core/src/kinds.rs` (l. 59-60) : renvoi à `crates/game/src/weapons/mod.rs` (→ `crates/combat/src/weapons/mod.rs`).
- `games/throne/assets/items/powerups.ron` (en-tête) : « poids 12 chacun », « un sur sept » (poids 36/12/12/12/24/25/25, `drop_chance` 0,25).
- `Makefile` (cible `gen`, l. 130-134) : le commentaire décrit le générateur de scénarios, la recette lance `map_generation`.

## Bilan `CLAUDE.md`

- **Attentes** : 43 dans `CLAUDE.md` (l. 313-328), 43 variantes d'`Expectation`
  (`crates/combat/src/weapons/expectations.rs`, réexporté par `game::replay`) : **aucun manque
  dans un sens ni dans l'autre**.
- **Kinds** : `CLAUDE.md` n'en liste **aucun** ; `KNOWN_KIND_NAMES` en compte **20** ; le §3 de
  `conventions.md` en listait 11, corrigé à 20 avec renvoi à `KNOWN_KIND_NAMES`. Rien à ajouter à
  `CLAUDE.md`.
- **make** : 11 cibles citées (`diff_log`, `test_scenarios`, `bench`, `play_scenario`,
  `review_videos`, `remote`, `record_session`, `test_multiplayer`, `videos`, `views`,
  `compare_video`), **toutes présentes** dans le `Makefile`. `test_multiplayer` exige `TARGET`
  (sans défaut : sinon `make _matchbox`), corrigé dans le diff.
- **Checklist « ajouter un vocabulaire »** : `CLAUDE.md` n'a pas de checklist de vocabulaire,
  seulement un renvoi (l. 13) et la « Checklist pour Nouveau Système GGRS ». Une contradiction
  réelle : l'étape 4 du §4 (`rollback_and_trace` pour chaque composant) omettait la variante
  `_neutral` qu'impose `CLAUDE.md` pour un type nouveau. Source unique : le **déroulé** dans
  `conventions.md` §4, les **règles GGRS et pièges** dans `CLAUDE.md` ; l'étape 4 du §4 renvoie
  désormais à `CLAUDE.md`, et le diff ajoute en tête de la checklist GGRS un renvoi au §4. Les
  pièges (parité du checksum, `derive(Hash)`) restent dans `CLAUDE.md`. Le piège « scale et
  collider » n'est pas dans `CLAUDE.md` (il est dans la mémoire de session et au §29, boss).
- **Autres identifiants** corrigés dans le diff : `Fixed::from_fixed_wide` (n'existe pas →
  `Fixed::from_num`), deux règles numérotées « 9 » (la seconde devient 10, et le « Pourquoi? » du
  logging, égaré sous la règle 9, revient sous la règle 8), chemin de `frame_events.rs`
  (`sim_core`), `TARGET` de `test_multiplayer`, renvoi « § HUD » → §15.
- **Non corrigés dans le diff, à décider par William** (contenu de fond) : section « Système IA
  (En Refonte) » périmée (`ZombieState` n'existe plus, `MonsterState` ; « Behaviors Engine
  (Futur) » livré, §22) ; exemple RON d'arme sans `firing_modes` ; arbre `ai/` sans `rules.rs` ;
  touche de debug F6 absente ; `docker compose` du serveur local (`docker-compose.ci.yaml`,
  service `signaling`, schéma de l'URL non vérifié).

Diff proposé (`git apply` depuis la racine de `alacod`) :

```diff
--- a/CLAUDE.md	2026-10-05 11:18:13.288134214 -0400
+++ b/CLAUDE.md	2026-10-05 11:18:13.282650480 -0400
@@ -138,7 +138,7 @@
 let fixed_val = Fixed::from_num(some_fixed_wide.to_num::<f32>());
 
 // ✅ CORRECT - Rester en fixed-point
-let fixed_val = Fixed::from_fixed_wide(some_fixed_wide);
+let fixed_val = Fixed::from_num(some_fixed_wide); // conversion fixe → fixe, sans f32
 ```
 
 #### 8. Logging Déterministe (pour comparaison des traces)
@@ -159,6 +159,9 @@
 info!("Frame {}: damage {} applied", frame.frame, damage);  // Valeurs de jeu
 ```
 
+**Pourquoi?** On compare les logs entre clients avec `diff` pour détecter les desyncs.
+Si les logs contiennent des Entity IDs, le diff montrera des différences même si la simulation est synchronisée.
+
 #### 9. Despawn différé des entités rollback
 
 Une entité rollback détruite par `despawn()` puis ramenée par un rollback (synctest, prédiction
@@ -180,10 +183,7 @@
 (`RollbackDespawned`) sont exclues des queries, des snapshots, du checksum et de la trace :
 la simulation se comporte exactement comme avec un despawn immédiat.
 
-**Pourquoi?** On compare les logs entre clients avec `diff` pour détecter les desyncs.
-Si les logs contiennent des Entity IDs, le diff montrera des différences même si la simulation est synchronisée.
-
-#### 9. Format GGRS Trace Logs (pour diff_log Makefile)
+#### 10. Format GGRS Trace Logs (pour diff_log Makefile)
 
 Les logs utilisés pour comparaison entre clients doivent suivre un format précis compatible avec `make diff_log`.
 
@@ -230,6 +230,10 @@
 
 ### Checklist pour Nouveau Système GGRS
 
+Le déroulé d'un nouveau vocabulaire (scénario → kind → composants → rollback → RNG → FrameEvents →
+traces → vidéo → doc) est dans `docs/conventions.md` §4 ; la checklist ci-dessous (règles GGRS et
+pièges) s'applique à chaque système et fait foi pour l'enregistrement rollback.
+
 - [ ] Query a `&GgrsNetId` en PREMIER si itération affecte l'état
 - [ ] Utilise `order_iter!` ou `order_mut_iter!` pour itérer
 - [ ] Trie par `net_id.0` (pas `entity.to_bits()`) avant traitement
@@ -275,7 +279,7 @@
 #### Événements dans la simulation
 **Jamais de `Message` bevy (`MessageReader`/`MessageWriter`) dans `GgrsSchedule`** : ils ne sont pas
 dans les snapshots et leurs curseurs ne sont pas rollbackés. Utiliser `FrameEvents<T>`
-(`crates/game/src/frame_events.rs`, `app.add_frame_events::<T>()`) : file vidée au début de chaque
+(`crates/sim_core/src/frame_events.rs`, réexporté par `game::frame_events` ; `app.add_frame_events::<T>()`) : file vidée au début de chaque
 frame (`RollbackSystemSet::FrameStart`), lue par les systèmes ordonnés après l'émetteur.
 
 Les visuels (portes, barres de vie, game over) se **dérivent de l'état** dans `Update`, jamais
@@ -398,10 +402,10 @@
 
 **Test à N joueurs via matchbox** (serveur de signaling allumette) :
 ```bash
-make test_multiplayer N=4
+make test_multiplayer TARGET=ldtk_map_explorer N=4
 ```
 
-La cible généralise le nombre de joueurs : `make test_multiplayer N=2` (défaut) lance 2 instances (alice et bob), `N=4` en lance 4 (alice, bob, charlie, diana), etc. Toutes les instances :
+`TARGET` (sans défaut) choisit le binaire lancé en `$(TARGET)_matchbox`. La cible généralise le nombre de joueurs : `make test_multiplayer N=2` (défaut) lance 2 instances (alice et bob), `N=4` en lance 4 (alice, bob, charlie, diana), etc. Toutes les instances :
 - rejoignent le **même** lobby (`LOBBY`, `test` par défaut) : c'est là que les pairs se trouvent ;
 - reçoivent `NUMBER_PLAYER=N` et `PLAYERS="localhost remote…"` (un `remote` par pair) ;
 - se lancent à `TIMEOUT` secondes d'intervalle (défaut 10 s), le temps des connexions.
@@ -414,7 +418,7 @@
 Pour utiliser un serveur allumette local (si disponible dans `docker-compose.yaml`) :
 ```bash
 docker compose up -d  # lance le serveur de signaling
-make test_multiplayer N=4 MATCHBOX_URL=http://localhost:3536  # URL personnalisée
+make test_multiplayer TARGET=ldtk_map_explorer N=4 MATCHBOX_URL=http://localhost:3536  # URL personnalisée
 ```
 
 Défaut : `MATCHBOX_URL=wss://allumette.bascanada.org` (serveur cloud).
@@ -514,7 +518,7 @@
 - `Text(source, prefix)` : texte, `prefix` devant la valeur. Pour les sources T2.12 (`perks`,
   `downed`, `powerups`, `prompt`), une valeur vide n'affiche rien, pas même le préfixe.
 - `Icons(source)` (T2.12) : rangée de carrés de côté `size.1`, un par entrée de la source
-  (`perks`), couleur et étiquette lues dans `icons` (voir `docs/conventions.md` § HUD).
+  (`perks`), couleur et étiquette lues dans `icons` (voir `docs/conventions.md` §15).
 
 #### Sources de données
```
