# m1-assembleur-d45-d47 — D45 (`.skip(r).last()`), D46 (chevauchement), D47 (appariement Spawn)

## État en cours

- Branche depuis main `0e4125b`. Code écrit, **pas encore compilé** (attente du feu vert d'orch).
- D45 : `.nth(r)` aux trois choix de `map::generation::imp::basic` (connexion libre, gabarit
  compatible, gabarit de départ) ; même nombre de tirages RNG.
- D46 : contrôle `is_overlapping` contre les salles placées avant `is_outside` (connexion →
  `DeadEnd`) ; `is_overlapping` en `<=` (salles adjacentes non chevauchantes).
- D47 : saut des `Spawn` supprimé dans `populate_level_connections` (boucle infinie si `Spawn`
  non premier ; asymétrique) — tous les `.ldtk` ont un unique `Spawn` en tête, appariement inchangé.
- Tests : `map` (D46 adjacence, D47 trois ordres), `map_ldtk` (avant_poste graines 1..20 : pas de
  chevauchement, > 1 carte distincte ; `ALACOD_MAP_SIGNATURES=1` imprime la mesure).
- Prochaines étapes : tests avant/après, suite scénarios + dumps (liste exacte des traces),
  20 graines zombies avant/après, rapport, LIVRÉ. Ne rien bénir.
