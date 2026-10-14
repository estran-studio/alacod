# D61 — Armure et bouclier

Branche `d61-armure-bouclier`. Conventions : `docs/conventions.md` §18 (sous-section D61).

## Ce qui est livré

- **Armure** : `StatId::Armor` lue au résolveur de dégâts (`dégât × 100 / (100 + armure)`, `after_armor`
  pure et testée), sauf `DamageKind::True`. Armure 0 = identité exacte. `Armor` ajoutée à
  `sim_core::stats::stat_reader` ; l'exception d'avertissement de M2-T15 est retirée (lint, `reach`,
  test et fixture `stat_armor_warning`).
- **Bouclier** : `combat::actors::Shield(u32)`, composant rollback **neutre** posé au premier gain ;
  `Action::GainShield(n)` en fin d'enum (effets, objets `OnUse`, consommables, synergies). Un point absorbe
  un coup entier, 15 frames d'i-frames, bouclier avant armure, `True` l'ignore.
- **Lint** : `GainShield(0)` refusé (effet, objet, frise) ; refusé dans un power-up ; fixture `shield_zero`.
- **Contenu testbed** : `plaque` (+100 d'armure), `egide` (`GainShield(1)`).
- **Scénarios** : `armor_reduces_damage`, `armor_true_damage`, `shield_absorbs_hit`, `shield_then_armor`.

## Écarts à la fiche

- La stat sans lecteur est traitée par `stat_reader`/`lint_stat_readers` (M2-T15, §53) ; mon lint maison a
  été retiré.
- HUD `shield` : laissé à M2-T18.
- `True` ignore aussi le bouclier (choix documenté §18).

## Pièges

- `Armor` n'a pas de base dans `Stats` : `StatReader::get(.., ZERO)`, pas `try_get`. Une attente `Stat`
  échoue donc (« sans stat Armor ») ; les scénarios mesurent la `Health`.
- La spirale de test ne tire qu'un coup de 8 PV toutes les ~90 frames : les attentes de bouclier regardent
  la seconde volée (f155) pour voir le coup qui touche.
