# Rapport m2-t3-effets-v2 (M2-T3, effets v2, voie V1b)

Branche `m2-t3-effets-v2` depuis `main` (6c712e25). Conventions : §42.

## Livré

- **Déclencheurs** `OnRoomClear`, `OnDodge`, `OnReload`, `OnPickup`, `OnFloorStart` exécutés ;
  `OnBlank` en contrat + lint seulement (**M2-T1 pas mergée** : `BlankUsed` n'existe pas encore ;
  quand elle l'est, ajouter la lecture du `FrameEvents<BlankUsed>` dans
  `apply_triggered_effects_system`, une ligne, et `OnBlank` à `trigger_supported`).
- **Conditions** `Carrying` et `SquadSize` ; `TargetInRange` laissée en contrat.
- **Cooldown / charges / recharge** (`PerRoom`, `PerFloor`), état dans `EffectState::limits`.
- **Effets passifs des objets** lus dans l'`Inventory` (choix justifié en §42).
- **Lint** à jour + fixtures (`effect_limits_invalid`, `effect_unsupported`, `item_effect_unsupported`).
- **Contenu** gungeon : `trefle`, `cartouchiere`, `amulette` ; testbed : huit objets d'essai.
- **Scénarios** : `effect_on_room_clear`, `effect_on_dodge`, `effect_on_reload`, `effect_on_pickup`,
  `effect_on_floor_start`, `effect_cooldown`, `effect_charges`, `item_passive_effect`.

## Choix et pièges

- **Événements plutôt que détection d'état** : `DashStarted` et `ReloadFinished`
  (`sim_core::triggers`) sont émis là où le fait a lieu (`Option<ResMut<FrameEvents<_>>>`, donc
  sans effet si la file n'est pas enregistrée). Le rechargement n'émet que quand le chargeur est
  réellement rempli.
- **Deux systèmes** : le système v1 (`DeathManagement`) garde `OnKill`/`OnDamageTaken`/`Tick`/
  `OnGauge`/`OnLevelUp` ; `RoomChanged` est émis dans `Run` et vidé au `FrameStart` suivant, donc
  les déclencheurs v2 vivent dans un second système après `Run` (et après `item_charge_system`, qui
  écrit `Inventory` : sans cet ordre bevy refuse le planning, ambiguïté d'accès).
- **`OnRoomClear` = salle du porteur** : position du porteur dans les bornes de la salle de
  l'événement (`LevelId` + `RoomBounds`), exact même quand les salles de plusieurs étages se
  recouvrent (§40).
- **`OnFloorStart` et le premier étage** : un porteur présent aux frames 0–1 déclenche à l'étage 0 ;
  un objet ramassé en cours d'étage ne se déclenche qu'au prochain (testé dans
  `effect_on_floor_start` : pas de +1 au ramassage).
- **Hash** : `Effect` et `EffectState` hachent leurs nouveaux champs seulement s'ils sont non vides ;
  `Effect` n'est plus `derive(Hash)`. Aucune trace existante n'a bougé (voir Vérifications).
- **Limites partagées** entre deux exemplaires du même objet (clé `item:<id>:<index>`).
- Effet d'objet `Modifier` : un modificateur **permanent** par exécution (il s'empile) ; utiliser
  `charges` pour un bonus unique.
- Une panne de disque (0 Go) a coûté un build : `scenario` demande ~10 Go de build seul.

## Mesures

Valeurs lues par sonde (attentes à 0 dans une copie du scénario) : `effect_charges` dégâts 1 (f100),
2 (f200, premier rechargement), 2 jusqu'à f700 (deuxième refusé), 3 (f800, après la salle) ;
`effect_on_floor_start` 1, 11 (rechargement f~290), 12 (étage 1, f~380), 22 (rechargement étage 1,
f~540) ; `item_passive_effect` 1, 2 (f100), 3 (f140).

## Vérifications

À compléter en fin de tâche (suite complète, crates, lint ×4, fmt, p2p).
