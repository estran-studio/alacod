# polissage-sons-evenements — S1-7 : sons d'événements du moteur et volume global

Branche `polissage-sons-evenements` (sur `origin/polissage-integration`), voie b2, VPS ovh-inner. Présentation seule :
aucun son ne part de `GgrsSchedule`, aucune trace ne bouge, aucun fichier sonore ajouté (S1-9 pour zombies, tâches de clone sinon).

## Points de la fiche

| Point | Issue | Preuve |
|---|---|---|
| 1. Clés optionnelles | fait | `content::feedback::SOUND_KEYS` (28 clés) ; `death_<id du personnage>` avec repli sur `death` (`death_sound_key`) ; lint : clé inconnue = `OutOfRange`, `death_<id>` d'un personnage inconnu = `BrokenReference` ; tests `cles_de_sons_inconnues_signalees`, `mort_par_personnage_avec_repli`, fixture `feedback_sound_key_unknown` |
| 2. Dérivé de l'état | fait | `crates/game/src/feedback_sounds.rs` (`Update`, décisions pures testées : `wave_sound`, `window_changes`, `menu_sound`, `log_sound_key`, `first_look`) ; absent en headless (`FeedbackPlugin`) donc sans effet sur les traces |
| 3. Volume global | fait | `audio::MasterVolume` (0–100, pas de 10, ressource de présentation, valeur de la session, pas de fichier de réglages) ; « Réglages » de la pause affiche `Volume : N %  [###---]` et ← → / croix gauche-droite le règlent ; ajouté en dB à `volumes`, 0 % ne joue rien, `ALACOD_MUTE=1` prioritaire ; scénario `ui_pause_menu` (f32–f34) |
| 4. Audio navigateur | **non vérifié** | aucun code ajouté ; protocole ci-dessous |
| 5. Tableau événement × clone | fait | ci-dessous |

## Clés et déclencheurs

| Clé | Déclencheur (état ou journal, `Update`) |
|---|---|
| `shot*`, `reload`, `hit`, `melee`, `hurt`, `dash`, `blank`, `door` | inchangés |
| `death`, `death_<id>` | un ennemi disparaît ; la clé du personnage (`CharacterConfigKey`) d'abord, `death` en repli ; le plus petit `GgrsNetId` mort donne le son |
| `wave_start`, `wave_end` | `WaveState` : numéro de vague qui monte ; phase qui passe à `WaveComplete` |
| `room_cleared` | `FrameEvents<RoomChanged>` vers `Cleared` |
| `floor` | `FloorState::index` qui monte |
| `boss_spawn`, `boss_phase`, `boss_death` | `BossState` qui apparaît ; `FrameEvents<BossPhaseChanged>` ; indice `BossEnd` du `FeedbackLog` |
| `purchase`, `pickup` | `CurrencyEvent` négatif d'un joueur local ; `ItemPicked` / `PowerUpPickedUp` d'un joueur local |
| `explosion`, `bomb` | indices `Explosion` / `BombFuse` du `FeedbackLog` (un son par clé et par frame) |
| `window_break`, `window_repair` | PV de `WindowHealth` : passe à 0 ; monte (demande de b1, S1-9) |
| `menu_navigate`, `menu_confirm`, `menu_back` | changement de `Screens::state` (focus ; ouverture ou validation de Réglages ; fermeture) et du volume |

Hors périmètre, non fait : ramassage d'une arme au sol (pas de `FrameEvents` dédié), mort d'un joueur, victoire/défaite.

## Tableau événement × clone

« son » : un fichier est déclaré dans `ui/feedback.ron` du clone aujourd'hui ; « repli » : la clé n'est pas déclarée, un repli
existe (`death_<id>` → `death`) ; « vide » : la clé n'est pas déclarée, rien ne joue (rendu d'avant, par construction). Le moteur
ne déclenche que les événements que le clone produit (n/a : le clone n'a pas l'événement).

| Événement | zombies | throne |
|---|---|---|
| tir / recharge / hit / melee / hurt / dash / door | son | son |
| mort | son (`death`) | son (`death`) |
| mort par personnage `death_<id>` | repli | repli |
| blank | n/a | n/a |
| vague début / fin | vide | n/a (sans vagues) |
| fenêtre brisée / réparée | vide | n/a |
| salle nettoyée | n/a | n/a (caves, pas de salles typées) |
| étage | n/a | vide |
| boss (apparition, phase, mort) | n/a | vide (`roi_rat`, niveau 3) |
| achat | vide | vide |
| ramassage (objet, power-up) | vide | vide |
| explosion | n/a | vide (roquette, lance-grenades) |
| bombe | n/a | n/a |
| menu naviguer / confirmer / retour | vide | vide |

Tous les « vide » sont des tâches de contenu : S1-9 (b1) pour zombies, S2-2 et S2-5 pour throne. gungeon et isaac n'ont aucune de ces
clés, donc aucun changement.

## Protocole d'audio navigateur (non vérifié, à jouer sur le Mac)

1. Build web du clone : `make build_wasm_apps` (ou le build `WEB_ENABLE_SOLO=1` du site local), servir `website/`, ouvrir l'URL de `/play?id=zombies` dans Chrome, **sans cliquer** : la console peut dire « AudioContext was not allowed to start ».
2. Cliquer dans la page (ou appuyer sur une touche) : l'audio doit démarrer au premier geste ; tirer, recharger, tuer un zombie doit s'entendre. Rien d'audible avant le geste est normal.
3. Échap → Réglages : ← abaisse le volume (la marche d'un cran s'entend sur le tir suivant), 0 % coupe tout, → remonte.
4. Refaire sous Safari si disponible. Noter pour chaque navigateur : son au premier tir après clic (oui/non), après 0 % puis 50 % (oui/non).
5. Si rien ne sonne après le clic : reprendre l'`AudioContext` sur le premier `pointerdown`/`keydown` dans `website/` (page `loader.html`) ; ce correctif n'est pas écrit, faute de pouvoir l'essayer ici.

## Constats et reste

- Rebase sur `origin/polissage-integration` : conflits avec la manette (b0, S1-1) dans `ui/screens/{input,mod,model}.rs` et `ui_pause_menu.ron`, résolus en gardant leurs tables `KEYBOARD_NAV` / `GAMEPAD_NAV` et en y ajoutant gauche/droite ; l'entrée « Touches » (aide) reste inchangée.
- Le texte « Aucun réglage pour l'instant. » est remplacé par le curseur (mis à jour dans `docs/conventions.md`).
- Non persisté : le volume revient à 100 % à chaque lancement (dette possible ; estimation 0,25 j avec `localStorage` côté web).
- Reste : vérification audio navigateur par le coordinateur (0,25 j côté Mac).
