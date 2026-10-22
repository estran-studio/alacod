# polissage-zombies-map-equilibrage — zombies : cartes, armes, campagne (S1-10)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1 axes 4 et 11),
`docs/taches/m0-revue-suite-contenu.md:47-65` (S4 armes ; « les 5 `ZombieSpawn` sont tous dans `Depart` »),
`docs/digests/revue-m0.md:38-48,297-315`, `crates/game/src/character/enemy/spawning.rs`, `docs/conventions.md` §1, §16, §29,
`games/zombies/assets/maps/avant_poste.ldtk`, `la_fonderie.ldtk`, `floor_grammars/la_fonderie.ron`, `weapons.ron` de
zombies, `docs/taches/rapports/m0-v8-carte-zombies.md`. Branche `polissage-zombies-map-equilibrage` depuis `origin/main`.
Estimation : 1,5 j. Clone : zombies. Machine : **VPS** (campagnes, LDtk ; vidéo sur Mac par S1-8).

## Contexte

Zombies : 4 armes, 6 configs, 1 boss, 6 salles (inventaire §4) ; deux cartes (`avant_poste`, 9 niveaux, et `la_fonderie`,
16 sections avec boucles). **Aucune campagne `alacod-sim` chiffrée.** Plaintes de William : fusil à pompe « super fort »,
pistolet inutile, mitraillette trop imprécise (S4) ; spawners concentrés dans `Depart`.

## À faire

1. **Audit de cartes** : par carte, nombre de salles, spawners et fenêtres par salle, couvertures, couloirs problématiques
   (tableau chiffré).
2. **Campagne « avant »** : `alacod-sim`, 1 et 2 bots, ≥ 100 graines, sur `avant_poste` et `la_fonderie` : vagues atteintes,
   morts par cause, armes utilisées.
3. Placer spawners et fenêtres dans les autres salles (LDtk ou script) ; régler les trois armes (`weapons.ron` : dégâts,
   cadence, dispersion, prix) pour un rôle distinct chacune ; `make gen GAME=zombies`. Aucune arme, effet ni mécanique
   nouveaux.
4. **Campagne « après »**, tableau comparé ; traces et scénarios régénérés (`BLESS=1`) avec **preuve §5** (première frame qui
   bouge et sa cause), dit dans le commit.

## Acceptation

- Audit et deux tableaux de campagne dans `docs/taches/rapports/polissage-zombies-map-equilibrage.md` ; 0 desync, soft-locks
  non expliqués ouverts en dettes (**D131+**).
- `make test_scenarios` vert ; William a joué 15 min et note les armes.
