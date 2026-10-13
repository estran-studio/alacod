# m2-t2-esquive-v2 — esquive v2 : charges, variantes en données, saut par-dessus les balles (M2, V1a, chantier B6)

Lire d'abord `docs/taches/README.md` (machine `orca`, §1 : `make sweep`, `make test_crates`,
signaling par session), `CLAUDE.md`, `docs/conventions.md` §34 (course et esquive : `DashState`,
`dash_step`, i-frames, buffer), §19 (statuts : étourdi, gelé), §41 (blank, i-frames du blank), et
le rapport `m2-t1-projectiles-v2-blank`. Branche `m2-t2-esquive-v2` depuis `main`. Estimation :
2 j. Section de conventions réservée : **§43** (§42 = effets v2, M2-T3 en cours chez b1).

## Contexte

Le dash v1 (§34) est un seul dash à cooldown, réglé par `MovementConfig` (`dash_distance`,
`dash_duration_frames`, `dash_cooldown_frames`, `dash_iframes`, `dash_buffer_frames`), état dans
`combat::actors::DashState` (rollback, **sous checksum et porté par tous les joueurs** : toute
modification de son hash déplace toutes les traces dès la frame 0). Les i-frames passent par
`Health.invulnerable_until_frame` et valent contre tout dégât. M2-T3 (b1) branche `OnDodge` sur le
début de dash : ne change pas ce qui marque ce début (`is_dashing` qui passe à vrai à la frame de
l'appui), ou préviens orch.

## À faire

1. **Charges de dash** : `dash_charges: Some(n)` (absent = v1 exact, une seule « charge » gérée
   par le cooldown) ; une charge consommée par dash, recharge d'une charge toutes les
   `dash_recharge_frames` (donnée). Le cooldown v1 reste l'écart minimal entre deux dashs.
2. **Variantes en données** : un dash nommé par personnage (`dash: Some(...)` dans le RON du
   personnage, défaut = réglage v1 de `MovementConfig`) : distance, durée, i-frames, et effet de
   variante en enum fermé, ajouté **en fin d'enum** : `Roll` (v1), `Shove` (bousculade : repousse
   les ennemis touchés pendant le dash, recul en donnée, `Velocity.knockback`), `InPlace` (sur
   place : aucune distance, seulement des i-frames, ex. parade sans mouvement).
3. **Saut par-dessus les balles** : i-frames **contre projectiles seulement** (`iframes_kind:
   Projectiles` / `All`, défaut `All` = v1) : les balles traversent, la mêlée et les contacts
   touchent. Ne pas surcharger `Health.invulnerable_until_frame` pour ça : état à part (neutre),
   lu par le test de collision des balles et des projectiles composables.
4. **Lint** : `dash_charges = 0`, `dash_recharge_frames = 0` avec des charges, durée 0 hors
   `InPlace`, i-frames > durée → refusés ; fixtures pour chaque refus.
5. **Contenu `gungeon`** : le pilote reçoit un roulé façon Gungeon (i-frames contre projectiles,
   deux charges si ça reste lisible) ; `zombies`, `testbed`, `throne` inchangés.
6. **Scénarios** : `dash_charges` (deux dashs enchaînés puis un refusé, recharge à la frame
   attendue), `dash_shove` (un ennemi repoussé, `EnemyDistance`), `dash_in_place` (position
   inchangée, `NoDamageBetween`), `dash_jump_bullets` (une balle ennemie traverse, un coup de
   mêlée touche dans la même fenêtre).

## Règles

Les `dash_*` existants, `movement_melee`, `surface_ice` et toutes les autres traces restent
**identiques** : `DashState` garde exactement son hash quand les nouveaux champs sont à leur défaut
(`impl Hash` manuel, ou mieux un composant séparé en variante **neutre** posé seulement quand la
donnée l'exige ; piège « type vide » et piège « enum à une variante » de `CLAUDE.md`). Les
traces `gungeon` bougeront avec le nouveau roulé du pilote : preuve §5 (`ALACOD_DUMP_TRACE` main /
branche + `trace-diff.py`), différences limitées aux nouveaux composants à la frame 0, cause écrite
dans le commit, avant tout bless. Pas de `f32`, `order_iter!`, front montant rollbacké (piège
« donnée dérivée non rollbackée »), `fuzz_inputs` (`ALACOD_FUZZ=0:8`) vert sur un scénario de dash.
`cargo fmt --all -- --check` avant de livrer.

## Hors périmètre

Parade (B5 parade), `OnDodge` des effets (M2-T3), présentation de l'esquive (V4), équilibrage.

## Vérification

README §4 (`make test_scenarios`, `make test_crates`, lint ×4, fmt, scripts, `make gen` ×4) ;
p2p à deux clients sur `gungeon` (port de la session, `MATCHBOX_PORT`) ; rapport
`docs/taches/rapports/m2-t2-esquive-v2.md`, conventions §43. Livrer par
`LIVRÉ m2-t2-esquive-v2 <sha>`.
