# m3-t16-hud-isaac — HUD `isaac` : cœurs, sous, bombes, clés, minicarte en grille (M3, V4, I1)

Lire d'abord `docs/taches/README.md`, `CLAUDE.md` (« HUD »), `docs/taches.md` §8.1, `docs/conventions.md`
§15, §56 (HUD gungeon, `bevy_hui`), §68 (cœurs), §70 (grille), §74 (curses). Branche `m3-t16-hud-isaac`
depuis `origin/main`, **après le merge de M3-T3 et M3-T5** (T9 pour les curses). Estimation : 3 j. Section de
conventions réservée : **§81**.

## À faire

1. Sources de HUD nouvelles (liste fermée `content::ui::HUD_SOURCES`) : `hearts` (cœurs pleins, demi, d'âme,
   selon Q2), `coins`, `bombs`, `keys` (les sources de clés existantes si elles suffisent), `item_banner`
   (nom de l'objet ramassé, quelques secondes, présentation), `curses`.
2. Widget `Hearts` (rangée d'icônes, demi-cœur) ; munitions masquées pour une arme `Infinite`.
3. Minicarte en grille : cases, icônes de type (trésor, boutique, boss, secret ouvert), salle courante ;
   masquée par la curse `HideMinimap`, vie masquée par `HideHealth`.
4. Attentes `HudText` dans des scénarios `isaac` ; captures en rendu logiciel au rapport.

## Règles

Présentation seule : aucune trace ne bouge. Ce qui est affiché dérive de l'état.

## Vérification

README §4 ; rapport `docs/taches/rapports/m3-t16-hud-isaac.md`, conventions §81.

## Session cloud

Tâche prévue pour une session Claude Code cloud. Ignorer les consignes `orca` du README (§1). Pousser la
branche régulièrement ; voie rapide du README §6 ; commit final `LIVRÉ m3-t16-hud-isaac` avec le rapport.
