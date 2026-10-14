# d61-armure-bouclier — armure (réduction à la League) et bouclier (à la Gungeon)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md`, `docs/conventions.md` §18/§19 (stats,
modificateurs, `DamageKind`), §36/§44 (objets, synergies), et la dette **D61** (`dettes.md`,
décision de William du 2026-10-14). Branche `d61-armure-bouclier` depuis `main`. Estimation : 1 j.
Prérequis de M2-T14b (objets d'armure). Section de conventions : sous-section de §18 (stats).

## À faire

1. **Armure** (`StatId::Armor`, existante) : au résolveur des dégâts
   (`game::character::health::rollback_resolve_damage_events`), un dégât reçu devient
   `dégât × 100 / (100 + armure)` (formule de League of Legends : 100 d'armure = moitié des dégâts),
   en `Fixed`, armure négative bornée à 0, **sauf `DamageKind::True`**. Armure 0 (ou stat absente) :
   dégât strictement inchangé (aucune trace ne bouge).
2. **Bouclier** (`Shield`, distinct de l'armure) : un nombre de points sur le joueur, chacun absorbe
   **un coup entier** (le dégât est annulé, le point disparaît, bref i-frames comme un coup reçu),
   à la Gungeon. Composant rollback **neutre** (posé au premier gain) ; gagné par une action d'effet
   (`GainShield(n)`, variante **en fin d'enum** d'`Action`) et utilisable par un objet, une synergie,
   un consommable. Ordre : bouclier d'abord, puis armure sur ce qui passe.
3. **Lint** : stat lue par aucun système → avertissement générique (la liste des stats lues est
   tenue à jour), `GainShield(0)` refusé ; fixtures.
4. **Contenu** : un objet d'armure et un consommable de bouclier dans `testbed` (pas dans `gungeon` :
   c'est M2-T14b) ; HUD : source `shield` (nombre de points) si simple, sinon laisser à M2-T18.
5. **Scénarios** : `armor_reduces_damage` (100 d'armure : dégâts divisés par 2, `Health`),
   `armor_true_damage`, `shield_absorbs_hit` (un coup absorbé, le suivant touche), `shield_then_armor`.

## Règles

Aucune trace existante ne doit bouger (armure 0 = identité exacte, bouclier neutre). Pas de `f32`.
`cargo fmt --all -- --check`. Vérification README §4 (`make test_scenarios_par` ou
`make test_scenarios`), rapport `docs/taches/rapports/d61-armure-bouclier.md`, `LIVRÉ d61-armure-bouclier <sha>`.
