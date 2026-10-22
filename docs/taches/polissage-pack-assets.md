# polissage-pack-assets — pack d'assets acheté ou libre (S-0)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§1 axe 9, §3 S-0, §5 décision 8), `docs/taches/plan-assets.md`,
`docs/conventions.md` §83 (langage visuel) et les conventions sprites (§ LDtk/sprites), `docs/plan-engine.md` §7
(« Consoles » : licence compatible commerciale **et console**), `games/*/assets/assets.yaml` (registres, licences
existantes), `crates/game/src/character/visuals.rs:183-200`, `docs/taches/rapports/m3-salles-murs-isaac.md`.
Branche `polissage-pack-assets` depuis `origin/main`. Estimation : 1,5 j (J1-J2). Voie : **v1, agent de recherche (pas de
build)** ; l'intégration est faite ensuite par les voies « clone complet » (S1-8, S2-1, S2-4, S2-10). Machine : aucune
compilation ; navigation web.

## Contexte

**Décision de William (2026-10-22)** : pour les sprites « propres au jeu », on **achète ou trouve un pack libre** qui couvre
le besoin ; plus de génération CC0 maison. Les quatre jeux partagent aujourd'hui des tilesets SunnyLand/Dungeon (inventaire
§10) ; gungeon/isaac ont des silhouettes générées ou des tuiles Kenney ; zombies et throne n'ont pas de `wall_art`.

## État au 2026-10-22

**La recherche est faite** : `docs/taches/rapports/polissage-pack-assets.md` (besoins § 1, candidats § 2, recommandation et options A / B / B′ / C § 3, ce qu'il faudra fabriquer § 4, étapes d'intégration § 5). **Reste à William** : choisir l'option (défaut B′, 8,99 $) et acheter ; puis archiver licences et `assets.yaml`. Points à connaître (travaux moteur dans S1-8) : icônes de HUD sans champ image dans `HudIcon` ; `wall_art` attend une feuille de 36 tuiles ; la surcharge locale ne couvre que `sprites.ron` ; licence « No Redistribution » de TheLazyStone à confirmer par l'auteur. Les points 1 à 3 ci-dessous sont remplis par ce rapport.

## À faire

1. **J1 — besoins par clone** : tableau « classe d'entité × clone » (joueur et chaque famille d'ennemi, boss, projectiles,
   ramassables/objets, portes, murs, sol, props, portail, HUD) avec **taille de tuile** (16 ou 32 px ; les murs gungeon/isaac
   ont leur format, `m3-salles-murs-isaac.md`), **animations requises** (idle, marche, attaque, mort ; nombre de directions
   `FacingDirection`), nombre d'entités par classe (compter dans les RON du jeu), ambiance (zombies : survie industrielle ;
   throne : cavernes ; gungeon : donjon ; isaac : sous-sol macabre [INFERENCE : lire les manifestes]).
2. **J1-J2 — recherche** : itch.io, OpenGameArt, Kenney, CraftPix, LPC, etc. **2 à 3 candidats par clone**, chacun avec :
   lien, prix, **licence exacte** (usage commercial oui, **redistribution des assets dans un build web** oui, console
   `docs/plan-engine.md` §7 : pas de clause « non console »/« personal use »), tailles de tuile, ce qui est couvert par
   rapport au tableau de besoins (%) et ce qui manque, cohérence de style entre clones si un même pack couvre plusieurs.
   Écarter tout pack dont la licence n'est pas lisible.
3. **J2 — William choisit et achète** ; l'agent livre une **page de décision** (tableau comparatif court, recommandation
   par clone, coût total) avant midi J2.
4. **J2-J3 — archive** : licence copiée dans `games/<jeu>/assets/` ou `docs/licences/`, entrée `assets.yaml` (source, auteur,
   licence, prix, date d'achat), mapping « classe → fichiers du pack » pour les voies d'intégration.
5. Ce qui reste hors pack : listé, avec son estimation ; la génération maison ne sert qu'à combler, dit à William.

## Acceptation

- `docs/taches/rapports/polissage-pack-assets.md` : besoins × clones, candidats avec prix et licence, choix de William,
  couverture par classe, trous.
- Licences archivées et `assets.yaml` à jour ; aucun code touché, aucune trace.
- **À demander à William** : achat (compte, budget, validation des licences), à J2.
