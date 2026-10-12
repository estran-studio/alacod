# M2-T1 : projectiles v2 et blank

Branche `m2-t1-projectiles-v2-blank`. Conventions : `docs/conventions.md` §41.

## 1. Fait

- **Patterns** `Fan`, `Spiral`, `Burst` (fin de l'enum `Pattern` et de `PatternEntry`, lint : références,
  valeurs, `on_expire`). Un émetteur multi-salves (`Emitter.multi: Option<MultiVolley>`, haché seulement
  si `Some`) porte Spiral et Burst ; Fan est une salve unique. Rebond des balles ennemies : vérifié
  (`enemy_bullet_bounce`), rien à corriger.
- **Blank** : consommable à effet (`ItemDef.blank: BlankSpec { radius, iframes }`), front montant de
  l'input avec mémoire rollback neutre (`BlankState`), `despawn_rollback` des seules balles ennemies dans
  le rayon, i-frames, `FrameEvents<BlankUsed>`, trace `ggrs{f= blank …}`. Consommables de départ
  (`CharacterEntry.starting_consumables`) et d'étage (`FloorsEntry.consumables_per_floor`, remonte à au
  moins n au début de chaque étage).
- **Contenu gungeon** : `bullet_kin` tire `salve` (Burst), le boss `gatling` a `eventail` (phase 0) et
  `spirale` (phase 1), `pistolero` a 2 blanks, `etage_1` en rend 2 par étage.
- **Scénarios** : `pattern_spiral`, `pattern_fan`, `pattern_burst`, `enemy_bullet_bounce`,
  `blank_clears_bullets`, `blank_empty`, `blank_floor_refill`, `bench_bullets_room` (535 balles, plancher
  20 fps dans `tests/budgets.ron` ; mesuré 24,2 fps sur machine chargée). `gungeon_boss` modifié (appuis
  de Blank).
- **Présentation** : onde du blank (24 frames) dérivée de `BlankUsed` dans `Update`, moment clé `blank`.
- 4 fixtures de lint ; outil `bullet_lines` (test ignoré) pour lire les balles frame par frame ;
  `ALACOD_PRESS_BLANK_AT` pour le p2p.

## 2. Vérifié (résultats réels)

- `make test_scenarios` complet **sans bless**, sur l'état fusionné avec origin/main (5dfbaf2a) : seules
  15 traces bougent (liste §3), toutes gungeon ou M2-T1 ; zombies, testbed, throne inchangés.
- Preuve §5 contre origin/main (`ALACOD_DUMP_TRACE` + `scripts/trace-diff.py --ignore
  BlankState,Inventory,FrameEvents<items::BlankUsed>`) : `weapon_pistolet` (199 frames) et
  `weapon_bare_hands` (399) **identiques** ; `gungeon_start` : seule différence à la frame 0 = données de
  `bullet_kin` (projectiles `etincelle`/`ricochet`, pattern `visee` → `salve`).
- `make test_crates` : RC=0, 632 tests, 0 échec (une première passe a échoué sur des artefacts d'un
  ancien rustc nightly et sur un disque plein ; refaite après `cargo clean -p` des crates concernés).
- `make lint` ×4 : 0 erreur. `cargo fmt --all -- --check` : propre. `check-rollback-registration.sh` : OK.
  `check-forbidden.sh` : avertissements préexistants.
- p2p `gungeon`, 2 clients, un blank à f100 (`ALACOD_PRESS_BLANK_AT=100`) : traces identiques (399 lignes).
  Mesuré avant la fusion d'origin/main, non refait après.
- `make gen` ×4 : voir le message de livraison.

## 3. Les 15 traces rebénies et leur cause

Gungeon, `Inventory` + `BlankState` du pistolero (2 blanks de départ) : `gungeon_start`, `gungeon_clear`,
`gungeon_boss`, `generated/gungeon/weapon_pistolet`, `weapon_bare_hands`. Gungeon, table de projectiles
de l'arme `arsenal` (+ `etincelle`, `ricochet`) et pattern `salve` de `bullet_kin` :
`generated/gungeon/enemy_bullet_kin_moving`, `enemy_bullet_kin_still`, `weapon_arsenal`. Scénarios M2-T1,
rebénis après le passage des tourelles sur l'arme `tourelle_gun` : `pattern_spiral`, `pattern_fan`,
`pattern_burst`, `enemy_bullet_bounce`, `blank_clears_bullets`, `blank_empty`, `bench_bullets_room`.

## 4. Non fait / incertain

- Compteur de blanks du HUD : pas de source dans `HUD_SOURCES`, laissé à M2-T18.
- Le p2p n'a pas été rejoué après la fusion d'origin/main.
- En-tête de commentaire de `gungeon_boss.ron` (frames citées) pas relu après les appuis de Blank.

## 5. Dettes et pièges rencontrés

- Deux fautes corrigées en route, chacune aurait déplacé des traces hors périmètre : projectiles ajoutés à
  `fireball_gun` (33 traces) et `BlankState` posé sur tout inventaire (4 traces testbed). Voir §41.
- Un worktree de comparaison partageant le `CARGO_TARGET_DIR` écrase les artefacts des crates du workspace
  (même nom, même hash) : toucher les sources (`find crates games -name '*.rs' -exec touch {} +`) avant
  de rebâtir la branche.
