# m2-t3-effets-v2 — effets v2 : déclencheurs, conditions, cooldowns et charges (M2, V1b, chantier C1 v2)

Lire d'abord `docs/taches/README.md` (machine `orca`, §1 : `make sweep`, `make test_crates`,
signaling par session), `CLAUDE.md`, `docs/conventions.md` §27 (effets v1), §35 (salles,
`RoomChanged`), §36 (objets, `OnUse`), §34 (dash), §40 (étages assemblés), et le rapport
`m2-t0b-contrats-objets`. Branche `m2-t3-effets-v2` depuis `main`. Estimation : 3 j. Section de
conventions réservée : **§42** (§41 = projectiles v2 et blank, M2-T1 en cours chez b0).

## Contexte

`crates/effects` porte les contrats (`On`, `Condition`, `Effect`) ; l'exécution v1
(`effects::runtime`, `game::effects_runtime`, T1.10) ne sait que `OnKill`, `OnDamageTaken`,
`Tick`, `OnGauge`, `OnLevelUp`, et les conditions `HpBelow`/`HasTag`/`TargetTag`/`NotHitFor`.
Les autres déclencheurs sont refusés par le lint (`trigger_supported`). Les objets (T0b) ont un
champ `effects` mais seul `OnUse` d'un actif est exécuté (`game::items`) ; le lint refuse le
reste (`Unsupported`, « contrats des effets v2, M2-T3 »).

## À faire

1. **Déclencheurs** exécutés : `OnRoomClear` (salle du porteur passée `Cleared`, lu dans les
   `FrameEvents<RoomChanged>`), `OnDodge` (début de dash du porteur), `OnReload` (fin de
   rechargement), `OnPickup` (ramassage d'objet par le porteur, `ItemPickup`), `OnBlank` (lu dans
   `FrameEvents<BlankUsed>` de M2-T1 : si T1 n'est pas mergée quand tu y arrives, contrat + lint
   seulement, et le dire au rapport), `OnFloorStart` (début d'étage, mode `Floors`, y compris le
   premier). Les nouveaux variants d'`On` vont **en fin d'enum** (hash des variantes existantes
   inchangé).
2. **Conditions** : `Carrying(item)` (inventaire du porteur), `TargetInRange`, `SquadSize` si
   simple, sinon laissées en contrat (le lint continue de les refuser).
3. **Cooldowns et charges** : champs optionnels de l'effet (`cooldown: Some(frames)`,
   `charges: Some(n)`, `#[serde(default)]`), état par porteur et par index d'effet dans
   `EffectState` (rollback) ; une charge consommée par déclenchement, recharge `per_room` /
   `per_floor` au choix (donnée). Sans ces champs, comportement v1 exact.
4. **Effets passifs des objets** : un objet passif ou actif porte ses `effects` sur son porteur
   (hors `OnUse`, déjà exécuté) : choisir entre les copier dans `Effects` au ramassage ou les lire
   dans l'`Inventory` (le justifier au rapport ; index stable pour les sources de modificateurs,
   `item:<id>:<index>`). Ordre des porteurs par `GgrsNetId`, des effets par index.
5. **Lint** : `trigger_supported`/`condition_supported` à jour ; `cooldown = 0`, `charges = 0`
   refusés ; fixtures pour chaque nouveau refus.
6. **Contenu `gungeon`** : deux ou trois objets qui utilisent les nouveaux déclencheurs (ex. soin
   `OnRoomClear` une fois par étage, munitions `OnReload` avec cooldown, bonus `OnPickup`).
7. **Scénarios** : un par déclencheur (`effect_on_room_clear`, `effect_on_dodge`,
   `effect_on_reload`, `effect_on_pickup`, `effect_on_floor_start`, `effect_on_blank` si T1 est
   là), `effect_cooldown` et `effect_charges` (déclenchement refusé pendant le cooldown / sans
   charge, recharge à la salle ou à l'étage), `item_passive_effect`.

## Règles

Aucune trace existante ne doit bouger : `Effects`/`EffectState` sont déjà en checksum neutre
(T1.10) ; un nouveau champ d'`EffectState` doit garder le hash d'avant quand il est vide (`impl
Hash` manuel si besoin, piège de `CLAUDE.md`). Si une trace bouge : preuve §5 (`ALACOD_DUMP_TRACE`
main / branche + `trace-diff.py`) avant tout bless. Pas de `f32`, `order_iter!`, `FrameEvents`
(jamais `Message`) ; un déclencheur lu dans des `FrameEvents` impose l'ordre du système après
l'émetteur (le documenter). `cargo fmt --all -- --check` avant de livrer.

## Hors périmètre

Synergies (paire d'objets, M2-T4), butin et coffres (M2-T5), boutique (M2-T6), présentation des
effets (V4).

## Vérification

README §4 (`make test_scenarios`, `make test_crates`, lint ×4, fmt, scripts, `make gen` ×4) ;
p2p à deux clients sur `gungeon` (port de la session, `MATCHBOX_PORT`) ; rapport
`docs/taches/rapports/m2-t3-effets-v2.md`, conventions §42. Livrer par
`LIVRÉ m2-t3-effets-v2 <sha>`.
