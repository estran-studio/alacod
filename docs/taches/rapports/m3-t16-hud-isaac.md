# Rapport m3-t16-hud-isaac : HUD `isaac` (M3, V4)

Branche `m3-t16-hud-isaac` depuis `origin/main` a2bf564a (M3-T3, T5, T9, T8, T12, T17, `m2-lisibilite-gungeon` mergées ; `origin/main`
n'a pas avancé pendant la tâche : rien à fusionner). Conventions : §81. Présentation seule.

## 1. Fait

- **Sources** (`content::ui::HUD_SOURCES`, lint `UnknownKind`) : `hearts`, `coins`, `item_banner` ; `keys`, `bombs`, `active_item`,
  `active_charge`, `minimap`, `curses` existaient. Détail source → rendu → attente au §81 (tableau).
- **Widgets** (`game::ui::hud`) : `Hearts(source)` (rangée de carrés rouges, vides, d'âme ; demi-cœur à moitié rempli ; passe
  à la ligne), `Counter(source)` (icône `icons[source]` + nombre, rangée masquée sans valeur), `Icons(source: "active_item")`
  (icône de l'actif), `room_icons` (lettre et bordure teintée par type de salle sur la minicarte).
- **Modèle** (`game::ui::hud_model`) : `HudHearts` (`Health` + `ExtraHealth` en demi-PV, `HEART_POINTS` = 2), `ItemBannerState`
  (bandeau dérivé de l'inventaire, 3 s, `???` sous `HideItemNames`, objets de départ non annoncés), `ammo` vide pour une
  arme `Infinite`.
- **Dettes soldées** : T9 (`item_names` pour les objets sans prompt : « Ramasser Reliquaire » / « Ramasser ??? »), T5
  (« Ouvrir — clé (n) » / « Fermé — clé (0) » avec les clés du joueur).
- **Contenu** : `games/isaac/assets/ui/hud.ron` (cœurs et compteurs en haut à gauche, actif en bas à gauche, bandeau en haut au
  centre, `room_icons` trésor/boutique/boss/secrète/diable).
- **Scénarios** : `isaac_hud_items` (neuf), `isaac_banner_blind` (neuf), `isaac_reliquaire` et `curse_hides_health` (attentes
  `HudText` ajoutées, trace inchangée).
- Doc : conventions §81, CLAUDE.md (« HUD », liste des attentes).

## 2. Décisions

- **Pas de plafond de 12 cœurs** (§68 le renvoyait à T16) : borner `Health.max`/`ExtraHealth` est de la simulation, hors
  « présentation seule ». La rangée passe à la ligne. À décider par William si un plafond est voulu.
- `coins` est un alias de `shells` (même valeur) : l'icône est prise par source. Dette notée au §81.
- Le coût `n` d'une porte à clé n'est pas affiché (le HUD ne connaît pas la salle de la porte ; `isaac` n'a que `Key(1)`).
- Le prompt (`prompt`) n'est pas dans le `HudSnapshot` (il lit la présentation) : prouvé par tests unitaires
  (`prompt_texts`, `blind_curse_hides_…`) et captures, pas par `HudText`.
- Icônes de rendu = carrés colorés avec une lettre (pas de sprites), comme `Icons` existant.

## 3. Vérifié (résultats réels, sur le code final)

| Commande | Résultat |
|---|---|
| `make test_scenarios_par SHARDS=3` (sans BLESS) | 3 tranches ok, **612 scénarios** fusionnés, 249 s ; 610 existants + `isaac_hud_items`, `isaac_banner_blind` |
| `make test_crates` | rc 0 ; 1 172 passés, 0 échec, 14 ignorés (1 167 avant : +5 tests unitaires HUD, +1 test de `hud.ron`) |
| `make lint` | zombies, testbed, throne, gungeon, isaac : « aucune erreur » ; isaac **26 avertissements** (inchangé) |
| `make gen GAME=…` ×5 | zombies, testbed, gungeon, throne, isaac : rc 0, 0 ko |
| `cargo fmt --all -- --check` | rc 0 |
| `make check_sim_boundary` | OK, 11 crates, 5 en dette D69 |
| `make test_tools` | 25 tests OK |
| `check-forbidden.sh`, `check-rollback-registration.sh` | OK, OK |
| Tests unitaires ajoutés | `hud_model` : `coeurs_rouges_vides_et_ame_en_demi_coeurs`, `coeurs_depuis_la_vie_et_l_ame_et_caches_par_la_curse`, `bandeau_d_objet_ramasse_et_noms_caches`, `bandeau_derive_de_l_inventaire_sans_annoncer_le_depart` ; `hud` : `prompt_texts` (Item, KeyDoor), `blind_curse_…` (Item), `isaac_hud_ron_montre_…` |

**Traces bougées : aucune.** `git diff origin/main --stat -- '*.trace'` : seules deux traces **neuves** (`isaac_hud_items`,
`isaac_banner_blind`) ; `isaac_reliquaire`, `curse_*` et les 610 autres identiques (suite sans BLESS verte). Pas de p2p : aucune
simulation, session ni état rollback touchés.

## 4. Captures et vidéo

Rendu `play_scenario --capture … --at` (feature `render`, GPU de la machine, 960×540, étiquettes F6 allumées d'office).
Dossier `docs/captures/m3-t16-hud-isaac/` :

| Fichier | Scénario, frame | Montre |
|---|---|---|
| `hud-items-f030-coeurs-bandeau.png` | `isaac_hud_items` f30 | 2,5 cœurs sur 3 (demi-cœur), compteurs 12 / 2 / 3, actif « Calice d'âme 30 / 30 · prêt », bandeau « Bottines légères », minicarte T et B |
| `hud-items-f100-ame.png` | idem f100 | cœur d'âme bleu à droite des rouges |
| `hud-items-f235-demi-ame-bandeau-eteint.png` | idem f235 | demi-cœur d'âme, bandeau éteint |
| `banner-blind-f020.png` | `isaac_banner_blind` f20 | bandeau « ??? », « Malédiction : blind » |
| `porte-a-cle-f048-prompt.png` | `grid_treasure_key` f48 | « [H] Ouvrir — clé (1) », 1 clé, minicarte |
| `minicarte-secrete-ouverte-f120.png` | `grid_secret_bomb` f120 | salle secrète `?` révélée, T, B, D, salle courante en or |
| `prompt-ramasser-f007.png` | `isaac_reliquaire` f7 | « [H] Ramasser Reliquaire » |
| `floor-seed-1-f2010.png` | `isaac_floor_seed_1` f2010 | 1,5 cœur sur 3, 1559 sous, bombe, salles nettoyées en vert |
| `isaac_floor_seed_1.mp4` | `make videos SCENARIO=isaac_floor_seed_1` (2,6 Mo) | la partie ordinaire de bout en bout ; ffmpeg sans `drawtext` : pas d'étiquette |

Le grand rectangle sombre en haut à gauche des captures est le vide hors de la salle (bord du niveau), pas un widget.

## 5. Relecture indépendante (agent `reviewer`)

Aucun P0/P1. P2 corrigé : un joueur sans `Inventory` (créé au premier ramassage) n'annonçait pas son premier objet ; il est suivi
avec un inventaire vide. P3 corrigés : bandeau éteint si l'objet annoncé n'est plus tenu (rollback), attente `hearts` vide dans
`curse_hides_health`, §70 mis à jour, icône de compteur masquée sans valeur. P3 gardés en dette : alias `coins`/`shells`,
littéral `HudConfig` « vide » copié (un `Default` serait à poser), troncature `i32` des PV fractionnaires (héritée de `health`).

## 6. Non fait / incertain / dettes

- Pas de plafond de 12 cœurs (décision ci-dessus). Pas d'icône de cœur en forme de cœur (carrés).
- `boutique` n'existe pas dans `isaac` (T11d) : l'icône `S` est déclarée mais jamais vue.
- Le coût d'une porte à clé n'est pas affiché ; le prompt n'a pas d'attente de scénario.
- Pas de bench ni de p2p (machine partagée ; simulation non touchée).
- Journal : ligne ajoutée à `docs/taches.md` §10 sur la branche (la voie du README dit « sur `main`, après le merge » : à reprendre
  si le merge la déplace).
