# m1-v1c-variantes — variantes et élites (T1.5, chantier D2)

Lire d'abord `docs/taches/README.md` (agent **local**, worktree de b0, branche
`m1-v1c-variantes` créée depuis la tête livrée de T1.4 `m1-v1c-behaviors-composables`, même
target). **Prérequis** : T1.4 (behaviors composables, `EnemyState`), T1.2 (émetteurs), T1.3
statuts (c3, si mergée : sinon ne pas en dépendre). Petite tâche (2 j), voie V1c ennemis.

## Contexte

Plan §6 : « T1.5 Variantes (D2) — `variants: { nom: (modifiers, tags, skin) }`, tirage par flux
`variants`. Acceptation : scénario `variant_fast`. » Plan-engine D2 : « Variantes et élites
(S). Modificateurs + tags + skin, tirage seedé (saison, nuit, champion). »

Ce qui existe : `sim_core::modifier` (`Modifier { stat, op: ModifierOp, value, source:
ModifierSource }`, `Modifiers(Vec<Modifier>)`, `remove_by_source`, résolution par
`stats::StatReader`, conventions §9) ; `Tag(String)` (`sim_core/src/tag.rs`, désérialise une
liste `["froid", "arbres"]`) et les politiques par tag (immunités §8) ; `CharacterConfig`
(`crates/game/src/character/config.rs` : `starting_skin`, `skins: HashMap<String,
CharacterSkin>`, `base_health`, `ai`) ; `spawn_enemy` (`enemy/create.rs` l.65) appelé par les
vagues (`waves/systems.rs`, type d'ennemi tiré par palier via le flux `"waves"`) et par les
`CharacterSpawn` LDtk ; santé résolue F5 (`ResolvedBalance.health_max_by_character`). Flux RNG
existants : `weapons`, `waves`, `loot`, `patterns` (T1.2), `behaviors` (T1.4). **Aucun flux
`variants`.**

## Décisions (fixées ici après revue avec b0, pas à réinventer)

1. **Déclaration** : dans le RON du personnage (`CharacterConfig`), champ optionnel
   `variants: Some((chance: "0.25", table: { "rapide": (weight: 1, modifiers: [(stat:
   EnemyMoveSpeed, op: Mul, value: "1.5")], tags: ["rapide"], skin: Some("rapide")), "blinde":
   (weight: 1, modifiers: [(stat: MaxHealth, op: Mul, value: "2.0")], tags: ["champion"],
   skin: None) }))`. `chance` (`Fixed`, défaut 1.0 quand la table existe) = probabilité d'avoir
   une variante ; puis tirage pondéré par `weight` (`u32`, défaut 1) dans l'ordre `BTreeMap`.
   `skin` référence une clé de `skins` du personnage (lint) et remplace `starting_skin` pour
   cette entité ; `modifiers` au format RON de §9 ; `tags` en **union** avec les tags du
   personnage (immunités §8, `Targeting::Nearest{ignore}` de T1.4 les voient). **Élite** = une
   variante comme les autres (tag `champion` + modificateurs), aucun mécanisme à part (pas de
   points ×2 : D4 boss et l'économie le décideront).
2. **Tirage déterministe et sans effet sur `RngStreams`** : un `RollbackRng` **local** de graine
   `fnv1a("variants") ^ run_seed ^ net_id` (deux tirages : chance, puis poids), au `spawn_enemy`
   de tout personnage qui déclare `variants` — vagues et `CharacterSpawn` compris, indépendant
   de l'ordre d'apparition et du moment de chargement de la carte (`RngStreams::get_mut`
   créerait une entrée, changerait le hash de la ressource et toutes les traces). Un
   personnage **sans** `variants` ne tire rien.
3. **Application** : composant rollback **neutre** `Variant(nom)` (`rollback_and_trace_neutral
   ::<C>`), posé seulement si une variante est choisie ; `modifiers` dans le `Modifiers` déjà
   présent avec `source: Named("variant:<nom>")` (durée de vie de l'entité) ; `tags` en union ;
   `skin` à l'apparition (présentation, sans effet headless). **Santé** : un ennemi n'a pas de
   stat de base `MaxHealth` (`enemy_stat_defaults` en pose 4, la santé vient de F5) → pour un
   personnage **à variantes seulement**, poser la base `MaxHealth` = santé F5 au spawn et
   appliquer le modificateur à la santé initiale (`sync_health_from_stats` le voit ensuite) ;
   les autres personnages ne changent pas (traces). Vitesse ennemie = `EnemyMoveSpeed` ; le lint
   **refuse** `MoveSpeed` sur un personnage IA avec un message qui renvoie à `EnemyMoveSpeed`.
4. **Variante forcée** : champ LDtk optionnel `variant` sur `CharacterSpawn` (conventions §1) :
   s'il est rempli, aucun tirage, la variante est imposée (lint : nom connu). C'est ce que les
   scénarios déterministes utilisent ; `variant_draw` (graine fixée, sans champ) couvre le
   tirage.
5. **Attentes** : `EnemyVariant { entity, variant: Option<String>, at_frame }` (nouvelle,
   `game::replay`, liste `CLAUDE.md`, test unitaire) ; `Stat` existante pour `EnemyMoveSpeed` ;
   kind d'événement `"variant_spawn"` (nom) dans `scenario::events::detect_events`.
6. **Contenu et scénarios testbed** : personnage `grunt` (copie mobile de `dummy`, behaviors
   `[Chase]`) avec `variants` `rapide` (×1,5 vitesse) et `blinde` (`MaxHealth` ×2, `champion`) ;
   arène `arena_ia` de T1.4 ou nouvelle `arena_variantes` avec trois `CharacterSpawn grunt`
   (`variant: "rapide"`, `variant: "blinde"`, sans champ) ; scénarios : `variant_fast`
   (`EnemyVariant("rapide")`, `Stat(EnemyMoveSpeed)` ×1,5, `EnemyDistance` plus courte que dans
   `variant_none`, même partie), `variant_none`, `variant_elite` (`EnemyVariant("blinde")`,
   `EntityHealth` ×2), `variant_draw` (graine fixée, `EnemyVariant` = la valeur tirée mesurée,
   `Event("variant_spawn")`). Quatre nouvelles traces (bless orchestrateur).
   `games/zombies` : **aucune variante** (contenu du clone figé pour M0, traces inchangées) ;
   une tâche à part, avec bless, pourra en donner aux zombies.
7. **Lint** : `weight` 0, `chance` hors [0,1], `skin` inconnu, stat inconnue, `MoveSpeed` sur
   IA, nom en double, `variant` LDtk vers un personnage/variante inconnus ; fixtures.
8. **Hors périmètre** : saison/nuit/champion comme sources de tirage (M3, contenu 1837),
   variantes d'armes (B5 v3), boss (D4), points d'élite, modificateurs de projectiles par
   variante (M4).

## Traces attendues

Aucune trace existante ne change : composant neutre posé seulement sur les ennemis à variante,
`RngStreams` jamais touché (RNG local), base `MaxHealth` posée seulement sur les personnages à
variantes, zombies sans variante.
Vérifiable par `make test_scenarios` vert sans bless (critère central, comme T1.4).

## Règles

Deux compilations au plus ; purge du target + point d'état après chaque suite (README §1) ;
aucune trace bénie par l'agent ; `docs/conventions.md` : **uniquement** §25 « Variantes et
élites » ; `CLAUDE.md` : attente `EnemyVariant` dans la liste ; `docs/taches.md` : ne pas toucher.
Merger `origin/main` juste avant de livrer ; conflits : garder les deux.

## Critères d'acceptation (vérifiés par l'orchestrateur sur l'état fusionné)

1. Tests unitaires : tirage déterministe (même graine + net id ⇒ même variante, poids 0 jamais
   tiré, `chance` 0 ⇒ jamais, 1 ⇒ toujours), application des modificateurs/tags/skin, base
   `MaxHealth` posée seulement avec variantes, champ LDtk `variant`, `EnemyVariant`, lint.
2. Scénarios `variant_fast`, `variant_none`, `variant_elite`, `variant_draw` verts ; **tous
   les scénarios existants verts sans trace modifiée**.
3. Suite des crates verte, `make lint` (deux jeux), `make fmt`, scripts, `make gen` des deux
   jeux sans modification ; exemples racine compilés.
4. §25 écrit ; rapport honnête avec point d'état.

## Livrer

Rapport `docs/taches/rapports/m1-v1c-variantes.md` sur la branche (README §7 ; sha de tête, base
`origin/main`). `git push -u origin m1-v1c-variantes`, puis `SendMessage` à `orch` :
`LIVRÉ m1-v1c-variantes <sha> : <une ligne>` (ou `BLOQUÉ …`). Ne merge pas, ne bénis pas.
