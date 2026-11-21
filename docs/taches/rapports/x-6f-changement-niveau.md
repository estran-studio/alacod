# X-6f — Changement de niveau par une porte

Jalon servi : **D2 dim 29 nov 2026** (`plan-1837-cli.md` § 6, ligne X-6f). Branche `wquintal/x-6f-changement-niveau`, partie de `94038bb9` (contenu de `ca3d7e28` + docs).

## (a) Relevé de l'existant (avant tout code, état `94038bb9`)

| Sujet | fichier:ligne | Ce que c'est |
|---|---|---|
| Passage de niveau | `crates/map_ldtk/src/game/floors.rs:473-677` (`floor_transition_system`) | garde `portal_open` :505 ; joueur debout qui franchit : :518-526 (`crosses_portal`, `run/src/floors.rs:65`) ; détruit tout non-joueur `despawn_rollback` :535-543 ; recrée le niveau du slot suivant :548-659 ; **place tous les joueurs** (par handle) sur les `PlayerSpawn` :588-619 ; `floor_state.index += 1`, `portal_open = false` :669-671 |
| Ouverture du portail | `floors.rs:360-391` (`floor_portal_open_system`) ; `run/src/floors.rs:59` (`portal_should_open`) | niveau vidé (aucun `Enemy`) ; victoire au dernier niveau si `victory_at_end` |
| Ordre des systèmes | `floors.rs:237-250` | `debug_clear_enemies → floor_portal_open → floor_transition`, `Run`, `run_if(FloorPlan)` |
| État du mode | `crates/run/src/floors.rs:24` (`FloorState`) ; `floors.rs:78` (`FloorPlan`, hors rollback) ; `:179-233` (`compute_floor_plan`) | `FloorPlan` lue de `content::registry::FloorsEntry` (`registry.rs:971`, `FloorsFileSchema:1190`, insertion :3873) |
| Slots / assemblé par graine | `floors.rs:88-111` (`slot_of_level` : chaque `floor:<id>` est son slot) ; conventions § 40 « Étapes 2 » | graine = graine de run + slot × `0x3C6E_F372` |
| Porte de carte | `crates/map/src/generation/entity/door.rs:15` (`DoorConfig`, `lock`) ; `map_ldtk/src/game/entity/door.rs:58` (lecture des champs LDtk) ; `map_ldtk/src/game/plugin.rs:432-488` (entité rollback : `Wall` + `DoorComponent` + `Collider` + `Interactable{Door}` + `DoorShape`) | ouverte = `Collider`, `CollisionLayer`, `Interactable` retirés |
| La serrure traverse la génération | `map_ldtk/src/generation/from.rs:116-150,308` ; `map/src/generation/imp/basic.rs:463-620` (jumelle :595) ; `map_ldtk/src/generation/to.rs:564` | un champ de porte d'auteur qui doit survivre à `map_generation` suit ce chemin |
| Ouverture avec clé | `crates/game/src/interaction.rs:305` (`handle_door_interaction`) | clé d'équipe cherchée avant le prix ; `consume` retire ; jumelle ouverte avec |
| Clé d'équipe | `crates/run/src/keys.rs:21` (`TeamKeys`) ; `crates/game/src/keys.rs:105` (`key_pickup_system`) ; `items/src/loot.rs:684` (`KeyPickup`) | `BTreeMap`, neutre ; remise à zéro à chaque run |
| Prompt de porte | `crates/game/src/ui/hud.rs:1624` (`door_prompt`) | « Ouvrir » / « Fermé — clé » |
| Attentes | `combat/src/weapons/expectations.rs:284` (`FloorIndex`) ; `MapDoor`, `TeamKeys`, `KeyPiles` (§ 117) | |

Constat : le passage de niveau **place déjà toute l'équipe** (joueurs debout, à terre, feu follet, tous gardés) ; seul le **déclencheur** (portail « niveau vidé ») est à rendre remplaçable par une porte.
