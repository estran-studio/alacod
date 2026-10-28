# X-2a « CLI : dépendance et lancement » — arrêt (2026-10-27, décision de William : gelé faute de besoin daté)

- **Branches poussées, non fusionnées** : `x-2a-cli-dependance-lancement` @ `928e88c2` (worker b2, base `25af5b5f`, antérieure à SB-6/SB-7) ;
  `integration-x2a` @ `adc042d4` = `928e88c2` fusionné sur `integration-1837` (X-1 + `main` `f2e39634`), réconcilié avec SB-6/SB-7.
- **Intégration `adc042d4`** : un seul client remote (`content::remote` de SB-6/SB-7) gagne `--port` ; le doublon `bin/alacod/remote.rs` est retiré ;
  `reload` en échec écrit le JSON du jeu sur stdout, code 1 (`Failure::reply`) ; `new`/`engine`/`run` ajoutés à côté de `sandbox`, `editor`, `get`, `set`, `undo`, `new-arena` et `self-check` ; contrat en §92 des conventions de la branche.
- **Tests lus sur `adc042d4`** : `cargo test -p content --bin alacod --test cli` donnait 20 réussis, 2 échecs. Les deux assertions comptaient sept dépendances ;
  elles comparent maintenant le manifeste réécrit. Ensuite, `--test cli engine_` : 6 réussis, mais la suite `cli` complète n'a pas été relancée.
  `cargo test -p scenario --no-default-features --test gifts` : 2 réussis (don enregistré puis rejoué en synctest). `cargo check -p content --bin alacod`, `check-forbidden`,
  `check-rollback-registration` et `cargo fmt --check` passent.
- **Non vérifié** : la suite `cli` complète après correction, les e2e ignorés (`make test_alacod_cli`), `make test_scenarios` et `make schedule_order_diff` sur `adc042d4`.
- **Revue indépendante de `adc042d4` : deux P2 ouverts**, corrigés puis jetés par la consigne d'arrêt (aucun code non commité gardé) :
  (1) `script.rs::run` jette `Failure::reply` : `remote run` perd le JSON d'un `reload` en échec ;
  (2) `engine.rs::pin`, dépendance par chemin relatif : `git -C <racine>` lu depuis le dossier courant au lieu du dossier du jeu (`game_dir.join(root)`).
- **Dettes prises** (sur la branche seulement) : D144 (`set_modifier` non enregistré), D145 (don rejoué dans `Effects`), D146 (gabarit lu au checkout de compilation),
  D147 (`pin`/`local` ne mettent pas `Cargo.lock` à jour), D148 (accès réseau implicite de `cargo metadata`). D126 confirmée, D130 et D131 fermées sur la branche.
- **Pas de tag `v0.1.0`.**
- **Reprise** : partir de `integration-x2a` (`adc042d4`), **pas** de `928e88c2` (il supprimerait les commandes de SB-6/SB-7) ; rebaser sur `main` ;
  corriger les deux P2 avec un test chacun ; relancer `cli` complet, `make test_alacod_cli`, `gifts`, `make test_scenarios` et `schedule_order_diff`.
- **Brouillon X-2b** construit sur cette intégration : voir `x-2b-arret.md`.
