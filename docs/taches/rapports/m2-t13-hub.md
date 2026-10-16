# M2-T13 — Hub

Branche `m2-t13-hub` (depuis `verif/train15`). Conventions : `docs/conventions.md` §57.

## Ce qui est livré

- **Services en données** : kind `HubService` (`hub/<id>.ron` : `name`, `needs` = conditions de profil
  `Unlock(id)` / `Stat(nom, n)`, `action` = `EnterRun` | `Merchant`), `entry.hub: Some((map: …))` dans
  `game.ron`, schéma, lint (carte inconnue, hub sans porte, condition inconnue, `Stat(_, 0)`) et trois
  fixtures. Contrats dans `meta::hub` (aucune dépendance au rendu).
- **Carte** `games/gungeon/assets/maps/hub.ldtk` : une salle 16×12, quatre `PlayerSpawn`, deux
  entités `HubStation` (`marchand`, `porte`). Chargée avec `MapGenerationMode::Static` (nouvelle
  variante, en dernier) : jouée telle que dessinée, sans assemblage.
- **Stations dans la simulation** : `HubStation` (rollback neutre) + `Interactable { Hub }`
  (`InteractionType::Hub`, en fin d'enum) ; `HubState` (ressource rollback neutre) reçoit `HubExit`
  (porte) et `HubVisit` (marchand) par `hub_station_system`.
- **Sortie décidée dans la simulation** (demande de l'orchestrateur) : `hub_exit_trigger` (`Update`) ne
  pose `RunRequest::EnterRun` qu'une fois `HubExit.frame` ≤ `ConfirmedFrameCount`. `EnterRun` et
  `ToHub` passent par le chemin du restart (local : retour direct en `GameLoading` ; en ligne : salle
  `{lobby}-r{n}`, §33) ; `HubSession` (hors rollback) change de carte et de mode (hub = `Sandbox`).
- **Marchand** : écran `bevy_hui` (`ui/merchant_screen.html`, même cadre que la pause/inventaire),
  ouvert par une visite confirmée ; achat par `purchase_for_handle` hors simulation (profil écrit
  tout de suite). `RunUnlocks` n'entre dans la simulation qu'à la porte.
- **Fin de run** : le bouton « Rejouer (R) » devient « Retour au hub (R) » (`RunRequest::ToHub`).
- **Scénarios** : `hub_use_stations` (marchand puis porte, `HubVisit`/`HubExit`), `hub_buy_unlock`
  (écran, achat, `ProfileHas`, `RunUnlocks` inchangé au hub) ; `Scenario::hub` (`stay`), attentes
  `HubExit`, `HubVisit`. **Test** `crates/scenario/tests/hub.rs` : la porte → requête une fois la
  frame confirmée → `Floors { etage_1 }`, étage 0, `HubState` remis à zéro, `RunUnlocks` de la run
  contient le déblocage acheté au hub.

## Décisions

- **Le jeu démarre au hub, les outils non.** `games/gungeon/src/main.rs` pose `HubSession` ; le runner
  de scénarios, `alacod-sim`, `make gen` et les campagnes de bots n'en posent pas (sauf champ `hub`) :
  ils jouent la run directement. Choix retenu plutôt que d'apprendre la porte à l'explorateur : aucun
  faux soft-lock, aucune trace existante ne bouge.
- **Coop (décision de William, 2026-10-15)** : l'hôte décide. Local = union des profils locaux ; en
  ligne = profil de l'hôte reconstitué de la configuration de session (`ALACOD_RUN_UNLOCKS` pour les
  déblocages, `ALACOD_HOST_STATS` pour les statistiques) : stand-in de la synchronisation des profils
  (hors périmètre). Chaque joueur gagne et écrit son propre profil (T12).
- **Porte : n'importe quel joueur la déclenche** (valeur par défaut, à confirmer par William).
- **Marchand ouvert dès `first_run`** : le premier passage se fait sans marchand, le jalon arrive à
  la fin de la première run.
- **Fin de run** : l'écran de résumé reste affiché ; seul le bouton principal change de destination.

## Preuves

- Sur main 3bab4764 : `make test_crates`, suite des scénarios en 3 tranches, lint ×4, `make gen` ×4,
  `check_sim_boundary`, fmt, fuzz `ALACOD_FUZZ=0:8` : tout vert.
- p2p à deux clients **depuis le hub** (gungeon démarre au hub ; profils différents, `ALACOD_RUN_UNLOCKS`
  et `ALACOD_HOST_STATS` identiques, port 3536) : 599 lignes de trace identiques. La porte franchie en
  ligne (relance par le chemin du restart, §33) reste à rejouer par l'orchestrateur.
- Aucune trace existante n'a bougé (`HubState`, `HubStation` neutres ; `hub` absent des scénarios
  existants ; `MapGenerationMode::Static` et `InteractionType::Hub` ajoutés en fin d'enum).

## Pièges rencontrés

- `Basic` (mode de génération par défaut) réécrit toute carte LDtk qu'on lui donne : sans
  `MapGenerationMode::Static`, le hub était réassemblé (joueur déplacé, `HubStation` perdues).
- `ALACOD_SCENARIO=… make test_scenarios` est écrasé par la variable Make `SCENARIO` (vide) : toute la
  suite tourne. Utiliser `SCENARIO=a,b`.
- Le runner de scénarios ne suit pas une relance de session (sa boucle compte les frames d'une
  seule session) : la relance a son propre test.

## Non fait (hors périmètre)

Synchronisation réseau des profils (pubkey allumette), PNJ de lore et services de narration (G3/G4),
prompt HUD de la station, graine de run variable en local (le restart local garde la graine, §33).
