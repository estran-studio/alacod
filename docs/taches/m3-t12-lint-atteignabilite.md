# m3-t12-lint-atteignabilite — lint des kinds M3 et atteignabilité (M3, V2)

Lire d'abord `docs/taches/README.md`, `docs/taches.md` §8.1, `docs/conventions.md` §53 (lint M2 et
`content::reach`), §60 à §65 (contrats de M3), et le code `crates/content/src/lint.rs`,
`crates/content/src/reach.rs`. Branche `m3-t12-lint-atteignabilite` depuis `origin/main`, **après le merge
des tâches de vague 0 (T0a à T0e)**. Estimation : 2 j, puis une passe courte après T11. Section de
conventions réservée : **§77**.

## À faire

1. **Audit** des règles de lint des kinds M3 (une ligne par règle au §77 : ce qu'elle refuse, sa fixture).
2. **Atteignabilité statique** étendue : familier jamais cité par un objet atteignable ; objet d'un pool que
   rien ne tire (trésor, diable, secret, rocher à butin, champion) ; type de salle `secrete`/`tresor`/`diable`
   qu'aucune grammaire ne pose ; curse jamais dans une séquence ; accomplissement vers un déblocage inconnu.
3. **`reach::loot_pool_items`/`loot_pool_weapons`** : le premier lit les pools (M2-T5), le second renvoie
   encore un ensemble vide (`crates/content/src/reach.rs`, constat du 2026-10-15) ; y ajouter les sources M3
   (rochers à butin, champions, salles `tresor`/`diable`/`secrete`) et, si `isaac` a des armes en pool, le
   second.
4. **Stats** : `ShotSpeed`, `ShotSize`, `Luck` présentes dans `stat_reader` ; aucune stat sans lecteur.
5. Fixtures pour chaque règle nouvelle ; `make lint` ×5 sans erreur.

## Règles

Lint seulement : aucune trace ne bouge. Un avertissement nouveau sur `isaac` est listé au rapport avec la
partie de T11 qui le fermera.

## Vérification

`cargo test -p content`, `make lint` ×5, `make test_crates` ; rapport
`docs/taches/rapports/m3-t12-lint-atteignabilite.md`, conventions §77.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Ignorer les consignes `orca` du README (§1). Pousser la
branche régulièrement ; commit final `LIVRÉ m3-t12-lint-atteignabilite` avec le rapport.
