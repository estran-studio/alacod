# m3-t13-generateur-v3 — générateur v3 : larmes, familiers, champions à l'échelle (M3, V3)

Lire d'abord `docs/taches/README.md`, `docs/taches.md` §8.1, `docs/conventions.md` §28 et §54 (générateur
v1 et v2), §66, §71, §72, et le code `crates/scenario/src/generate.rs`,
`crates/scenario/src/bin/alacod-gen.rs`. Branche `m3-t13-generateur-v3` depuis `origin/main`, **après le
merge de M3-T1, M3-T6 et M3-T7**. Estimation : 3 j. Section de conventions réservée : **§78**.

## À faire

1. **Gabarit `TearsOnTarget`** : le joueur tient l'objet (`PlayerScript::items`), tire sur une cible ; attentes
   déclarées par le `test:` de l'objet (`HitsAtLeast`, `BulletCount`, `Stat`) ; cible à PV réglable.
2. **Gabarit `FamiliarHeld`** : le joueur tient l'objet familier, attente `FamiliarCount` et coups du familier.
3. **Gabarit `ChampionForced`** : l'ennemi apparaît avec sa variante imposée (`CharacterSpawn.variant`) ;
   attentes `EnemyVariant` et l'effet de mort (`EntityCount`, `Event`).
4. **Échelle** : 100 objets générés en moins de N secondes de `make gen` (mesure au rapport), noms de
   fichiers stables, aucun `.ron` écrit à la main pour un objet ordinaire.
5. Lint du champ `test:` des nouveaux gabarits, fixtures ; `make gen` ×5 sans diff.

## Règles

Outillage seulement : traces existantes inchangées (les scénarios générés existants sont identiques après
`make gen`).

## Vérification

README §4 ; rapport `docs/taches/rapports/m3-t13-generateur-v3.md`, conventions §78.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Ignorer les consignes `orca` du README (§1). Pousser la
branche régulièrement. Voie rapide du README §6 : fusionner `origin/main` et rejouer la suite juste avant le
commit `LIVRÉ m3-t13-generateur-v3`, qui contient le rapport.
