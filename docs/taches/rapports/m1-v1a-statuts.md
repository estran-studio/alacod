# Rapport — m1-v1a-statuts : statuts Burn/Slow/Stun/Freeze (T1.3, voie V1a)

**SHA de tête : celui annoncé dans le LIVRÉ** (commit de ce rapport). Branche `m1-v1a-statuts`
depuis `65a4d25`, reprise de zéro en local (la session cloud c3 n'avait jamais livré) ;
`origin/main` `4393db6` mergé (`31f4937`, sans conflit). Fiche :
[m1-v1a-statuts](../m1-v1a-statuts.md), amendée (décisions acceptées par l'orchestrateur ;
« Règles cloud » ignorées). Worktree `alacod_tasks/m0-v7-phase2-bots-finisent-le-clone/` ;
agent : Claude Code (b1) ; date : 2026-10-04.

## État en cours

- **Fait** : tout le périmètre (quatre statuts, pose par projectile, attentes, contenu testbed,
  visuel, §19).
- **Chiffres** : toutes les traces existantes identiques ; 6 traces nouvelles à bénir ;
  504 tests de crates réussis.
- **Prochaine étape** : LIVRÉ, attente de la vérification de l'orchestrateur.

## Fait

- **`combat::status`** : `StatusSpec`, `StatusLibrary`, `StatusEntry` étendu en fin (id, source,
  équipe de la source, prochain tick), `Statuses::{apply, tick, has, stacks, incapacitated,
  frozen}` purs ; tests : ticks de `Burn` et expiration, empilement au plafond (2 × la durée, 2
  piles), rafraîchissement de `Slow`/`Stun`/`Freeze`, incapacité et gel, source créditée.
- **`effects::Action::ApplyStatus { status, stacks }`** (dernier variant) ; exécutée par
  `apply_projectile_on_hit_system` sur le personnage touché (`on_hit`) ; `Slow` pose deux
  modificateurs (`MoveSpeed`, `EnemyMoveSpeed`) jusqu'à l'expiration. Dans un effet T1.10 :
  `Unsupported` ; dans un power-up : refusée (suite §19 : `OnDamageTaken` connaît sa source).
- **`status_tick_system`** (`Projectiles`, après la pose) : ticks de `Burn` en `DamageEvent`
  `Fire` crédités à la source, résolus dans la même frame ; expiration ; composant retiré quand
  vide. **`status_motion_system`** (`Movement`, avant `move_characters`) : vitesse nulle sous
  `Stun`/`Freeze`, recul nul sous `Freeze`.
- **`Stun`/`Freeze`** : joueur — inputs neutralisés dans `apply_inputs` (visée gardée), pas de
  tir (`weapon_rollback_system`), pas de mêlée ; ennemi — aucune règle retenue (`rules`), pas de
  déplacement (`move_enemies`), pas d'attaque (`enemy_attack_system`), émetteur suspendu.
- **Contenu** : kind `Status` (`statuses/<id>.ron`, miroir `StatusKindEntry`), `lint_statuses`,
  `lint_weapon_statuses`, fixtures `status_out_of_range` et `status_unknown` ;
  `game::statuses::resolve_status_library_system` (`OnEnter(GameLoading)`).
- **Attentes** `HasStatus(entity, status, present, at_frame)` et `StatusStacks(entity, status,
  stacks, at_frame)` ; test `status_expectations_pass_and_fail`.
- **Testbed** : statuts `brulure`, `lenteur`, `etourdi`, `gel` ; armes `status_burn`,
  `status_slow`, `status_stun`, `status_freeze` (`test:` avec les deux attentes) → scénarios
  générés `weapon_status_*` ; scénarios écrits `status_freeze_enemy`, `status_slow_enemy`.
- **Visuel dérivé** : `game::feedback::tint_status_sprites` (teinte des calques par statut).
- **Docs** : `docs/conventions.md` §19, `CLAUDE.md` (attentes).

## Décision prise en cours : enregistrement de `Statuses` gardé checksummé

Variante neutre essayée (comme convenu) : **toutes** les traces générées du testbed (et donc
tous les scénarios) changeaient dès la frame 0 — le type était déjà enregistré en rollback +
checksum par T1.0a, sans porteur, et les traces de référence contiennent cette contribution. Le
piège de parité concerne l'enregistrement d'un type **nouveau** ; ici, garder l'enregistrement
d'origine laisse les traces identiques (vérifié : `weapon_pistol`, `idle`, puis la suite).

## Traces

Aucune trace existante ne change. Six nouvelles, à bénir : `status_freeze_enemy`,
`status_slow_enemy`, `generated/testbed/weapon_status_{burn,freeze,slow,stun}`. Preuve « diff
réduit au nouveau composant » sans objet : aucune trace existante ne bouge.

## Vérifié

Commandes précédées de `source ../env.sh` et `export CARGO_BUILD_JOBS=2`, profil headless, target
reconstruit de zéro (targets supprimés par William).

- **Générés** (`make gen GAME=testbed`) : `weapon_status_burn` (une pile à f100, deux à f175 :
  deux balles touchent `target` vers f75 et f165, les quatre autres « coups » comptés sont les
  ticks), `weapon_status_stun` (`etourdi` présent à f100, expiré à f150), `_slow`, `_freeze` :
  attentes vertes ; toutes les autres traces générées identiques.
- **Effet en jeu** (couloir B de `testbed/surfaces_enemy.ldtk`, breacher net_id 20, contact à
  f338 sans tir) : `Slow` → contact à **f410** ; `Freeze` → breacher gelé tant que les balles
  arrivent, contact à **f625** (le gel expire pendant le rechargement) ; `Stun` (mesuré, pas de
  scénario) → f551.
- `make test_scenarios` (sans bless) : **toutes les traces existantes identiques** ; échecs
  uniquement « pas de trace de référence » pour les six nouvelles. `bench_horde` **42,3 fps**
  (plancher 38).
- `cargo test` des quatorze crates : **504 réussis, 1 échec, 9 ignorés** (l'échec : `scenarios`,
  traces nouvelles).
- `make lint` (trois jeux), `cargo fmt --all -- --check` vide, `check-forbidden.sh` 4 occurrences
  préexistantes, `check-rollback-registration.sh` OK, `make gen GAME=zombies` sans modification,
  `make gen GAME=testbed` : seules les 4 traces nouvelles absentes, `cargo check -p throne` et
  exemples code 0.

## Non fait / dettes

- Statut posé par contact (mêlée, `Charge`) ou par un effet `OnDamageTaken` : v2 (§19).
- `make gen GAME=throne` toujours rouge : T1.13 a ajouté `generate_template`, mais `games/throne`
  ne le déclare pas encore (carte `gabarit_armes.ldtk` prête) — petite suite à faire.
- Pas de vérification visuelle de la teinte (jeu fenêtré non lancé).
