# Rapport — plan-m4-isaac (M4 `isaac` découpé en fiches)

Session cloud du 2026-10-15, branche `plan-m4-isaac` depuis `origin/main` `7172660`. **Documentation
seulement** : aucune ligne de code, aucune trace, aucun scénario touchés. Proposition à valider par William ;
rien n'est décidé sur le design (questions en fin de rapport).

## 1. Fait

- `docs/taches.md` **§8.1 « M4 : `isaac` en tâches »** (et une phrase dans l'intro du §8) :
  - tableau d'écart, chantier par chantier : ce que M0 à M2 ont livré, ce que M4 ajoute, quelles tâches ;
  - sections de conventions réservées **§60 à §82** (§58–§59 laissés à la fin de M2) ;
  - vague 0 (six contrats), vague 1 par voie, V2, V3, V4, vague 2 ;
  - tableau des prérequis (avant / après la fin de M2), ordre de merge ;
  - tableau d'atteignabilité (où une partie ordinaire de `isaac` rencontre chaque mécanique) ;
  - **critères de sortie de M4**, mesurables (scénarios, campagnes, bench, p2p, humain).
- `docs/plan-engine.md` §9.8 : renvoi vers les critères propres à M4.
- 24 fiches au format des fiches M2 (lire d'abord, contexte, à faire, règles, hors périmètre, vérification,
  atteignabilité, session cloud). Les fiches T0 sont détaillées, les autres plus courtes.
- Méthode : le code de `effects`, `items`, `combat` (projectiles, émetteurs, statuts, santé), `world`, `map`,
  `behaviors`, `run`, `meta`, `content`, `bots`, `scenario` et le contenu `gungeon` a été lu (trois
  inventaires parallèles, puis vérification ponctuelle des points cités). L'écart est celui du code, pas
  seulement de la doc.

### Liste des fiches

| Fiche | Tâche | Voie | Estimation | Prérequis |
|---|---|---|---|---|
| `m4-t0a-contrats-larmes.md` | munitions infinies, `ShotSpeed`/`ShotSize`, `Luck`, modificateurs portés par les objets, composition, `Spectral`/`Split`/`Orbit` (contrats) | V1a | 2-3 j | aucun |
| `m4-t0b-contrats-objets-sante.md` | compte d'exemplaires, limites par exemplaire, `PerCopy`, `Price`, i-frames après un coup, couche de vie (Q2), `bomb`, plafond, partage (Q4) | V1b | 2-3 j | aucun (fusion de `main` : M2-T12 touche `items`) |
| `m4-t0c-contrats-monde.md` | `Explode`, bombe et bit 18, rochers sur les salles, piques, `DoorLock`, `FloorLayout::Grid` | V1d | 3 j | aucun |
| `m4-t0d-contrats-familiers-champions.md` | `Follow`, `Orbit`, `MirrorShot`, `BlockBullets`, `FamiliarOf`, champ `familiar`, `VariantDef` v2 | V1c | 2 j | aucun |
| `m4-t0e-contrats-curses-pools.md` | kind `Curse`, `RunCurses`, pools à épuisement, kind `Achievement` | V1e | 2 j | **M2-T12** |
| `m4-t0f-squelette-isaac.md` | `games/isaac` démarre | V2 | 1-2 j | aucun |
| `m4-t1-larmes-v3.md` | tirs composés, `Spectral`/`Split`/`Orbit`, `Poison`/`Fear`, chance par `Luck` | V1a | 5 j | T0a |
| `m4-t2-effets-v3.md` | `OnHit`, `OnEvent`, `TargetInRange`, `ApplyStatus`, exemplaires, transformations (Q10), 100 effets tenus | V1b | 4 j | T0b |
| `m4-t3-coeurs-pickups.md` | couche de vie, i-frames, sous/bombes/clés au sol, plafonds, partage | V1b | 3 j | T0b, Q2, Q4 |
| `m4-t4-bombes-rochers.md` | bombe posée, rochers cassés et à butin, mur secret, piques, baril sur `Explode` | V1d | 4 j | T0c |
| `m4-t5-grammaire-grille.md` | planificateur en grille, secret, trésor à clé, post-boss | V1d | 5 j (risque) | T0c, Q8 |
| `m4-t6-familiers.md` | familiers et orbitaux en jeu, coop à 4 | V1c | 4 j | T0d, T1, Q13 |
| `m4-t7-champions.md` | champions v2 en jeu | V1c | 2 j | T0d, T0c |
| `m4-t8-pools-marche-diable.md` | pools par type de salle, épuisement, prix en vie max, salle du diable | V1b | 4 j | M2-T12, T0e, T3, T5, Q5, Q6 |
| `m4-t9-curses.md` | curses tirées par étage, effets | V1e | 3 j | T0e, T5, Q7 |
| `m4-t10-profil-accomplissements.md` | accomplissements → déblocages | V1e | 3 j | M2-T12, T0e |
| `m4-t11-contenu-isaac.md` | contenu en quatre parties a/b/c/d | V2 | 10 j | selon la partie, Q1, Q9, Q14 |
| `m4-t12-lint-atteignabilite.md` | lint des kinds M4, atteignabilité | V2 | 2 j | vague 0 |
| `m4-t13-generateur-v3.md` | gabarits larme, familier, champion | V3 | 3 j | T1, T6, T7 |
| `m4-t14-bot-isaac.md` | bot en grille, bombes, clés, achats, diable, campagnes | V3 | 4 j | T3, T4, T5 |
| `m4-t15-bench-larmes.md` | `bench_tears`, `bench_floor_isaac`, coût du rollback | V3 | 2 j | T1, T6 |
| `m4-t16-hud-isaac.md` | cœurs, sous, bombes, minicarte en grille, curses | V4 | 3 j | T3, T5 (T9) |
| `m4-t17-feedback-v3.md` | larmes lisibles, familiers, champions, explosions | V4 | 2 j | T1, T4, T6, T7 |
| `m4-t18-integration-sortie.md` | scénarios du clone, campagnes, bench, p2p, revue, dossier de sortie | intégration | 5 j | tout |

Total : environ 80 jours-agent, six à huit semaines calendaires à quatre voies.

## 2. Tâches lançables tout de suite

Techniquement, sans attendre M2 ni William : **M4-T0a** (larmes, `combat`), **M4-T0c** (monde, `world`/`map`),
**M4-T0d** (familiers et champions, `behaviors`), **M4-T0f** (squelette `games/isaac`) ; **M4-T0b** pour sa
partie technique (la couche de vie attend Q2, le partage attend Q4). Aucune tâche M2 restante ne touche
ces crates, sauf `items` (M2-T12) pour T0b.

Attendent : **M4-T0e**, **M4-T8**, **M4-T10** (merge de M2-T12) ; tout le reste (leur T0, puis les
mécaniques qu'ils utilisent : colonne « Prérequis »).

Limite d'orchestration : la règle 7 de `docs/taches.md` §1 (quatre voies actives au plus) et les tâches M2
encore ouvertes (M2-T12, M2-T13, M2-T14d, `revue-sortie-m2`, fiches de dettes) ; combien lancer est Q15.
Ordre conseillé si une seule place se libère : T0a (bloque T1, T6, T11a, T15), puis T0c, T0d, T0f.

## 3. Vérifié

- Chemins de fiches cités par `docs/taches.md` et par les fiches : tous présents (boucle `[ -f ]`, aucun
  manquant) ; 24 fiches.
- Sections réservées : une par fiche, de §60 à §82, sans doublon ; aucune ne recouvre une réservation M2
  (§51 et §57 restent à M2-T12 et M2-T13).
- Points de code cités vérifiés à la lecture : `MagBulletConfig` (deux variantes), `StatId` (`Custom` en
  dernier, `stat_reader`), bits d'input 0 à 17, `loot_pool_weapons` vide, sens de `ReloadSpeed`.
- Non fait ici : aucune compilation, aucun scénario, aucun p2p (tâche documentaire ; rien à exécuter).

## 4. Constats de lecture (pas des décisions ; à numéroter en dettes par l'orchestrateur)

1. **`ReloadSpeed` multiplie la durée du rechargement** (`crates/combat/src/weapons/mod.rs`,
   `reload_time_seconds.saturating_mul(reload_speed_mult)`). Le perk zombies l'utilise dans ce sens
   (`Mul 0.5` = deux fois plus rapide, commenté), mais l'objet gungeon `chargeur_rapide`
   (`Pct 0.3`, « 30 % plus rapide ») **ralentit** le rechargement de 30 %. Q18.
2. Deux exemplaires d'un même objet **partagent leurs limites d'effet** (clé `item:<id>:<index>`) :
   `amulette` et `gilet` (gungeon, `charges: Some(1)`) ne donnent rien au second exemplaire. Les pools gungeon
   n'excluent pas les objets tenus, donc le cas peut arriver en partie. Pris par M4-T0b/T2.
3. `EffectsPlugin` ne déclare pas `OnFloorStart` ni `OnBlank` dans ses kinds ; `CombatPlugin` ne déclare pas
   `Fan`, `Spiral`, `Burst` (registre informatif seulement). Pris par T0a et T2.
4. `content::reach::loot_pool_weapons` renvoie toujours un ensemble vide. Pris par T12.
5. Docs périmées : `docs/conventions.md` §3 annonce 20 kinds (le registre en connaît 26), §40 parle de
   9 gabarits gungeon (il y en a 12) ; le sommaire des conventions s'arrête à §39.
6. `BulletType::Explosive.blast_radius` n'est lu par rien (une balle « explosive » ordinaire ne fait que
   disparaître) ; `Run.flags` n'est ni lu ni écrit.

## 5. Non fait, incertain

- Estimations : à l'œil, sur le modèle des tâches M2 de même taille ; à recaler après la vague 0.
- La faisabilité des rochers par le `CellGrid` des cavernes (T0c, point 3) est une proposition technique,
  non prototypée ; la fiche laisse à la tâche le choix d'une entité par rocher si c'est plus simple.
- Le planificateur en grille (T5) peut demander d'étendre `FloorPlan` de l'arbre au graphe : risque principal
  de M4 côté moteur.
- Le coût de `Split` en chaîne et du `Homing` à 400 projectiles sous rollback n'est pas mesuré (T15).

## 6. Atteignabilité

Pas de mécanique livrée ici. Le tableau du §8.1 nomme pour chaque mécanique où une partie ordinaire de
`isaac` la rencontre ; chaque fiche a sa section « Atteignabilité », et T11d remplit le tableau graine par
graine.

## 7. Questions pour William

| # | Question | Proposition par défaut (rien n'est codé avant la réponse quand la fiche dit « bloqué ») | Fiches |
|---|---|---|---|
| Q1 | **Volume du clone** : le tableau des jalons dit « 100 objets », le §1 du plan dit qu'un clone minimal a « 15 à 20 objets ». Lequel pour M4 ? | 100 objets (c'est la preuve « effets à l'échelle »), dont 30 de larmes, 8 familiers, 10 actifs ; 10 ennemis, 2 boss, 20 gabarits | T11, critères |
| Q2 | **Modèle des cœurs** : demi-cœur = 1 PV ? Cœurs d'âme (une couche absorbée point par point, différente du bouclier D61) ? Cœurs noirs ? | demi-cœur = 1 PV ; cœurs d'âme en couche `ExtraHealth` ; pas de cœurs noirs | T0b, T3, T16 |
| Q3 | **Empilement des modificateurs de larmes** : la table de T0a (rebond et perce s'additionnent, taille multipliée, guidage plafonné, durée au maximum) convient-elle ? Une larme divisée hérite-t-elle des modificateurs ? | la table de T0a ; les enfants de `Split` héritent de tout sauf `Split` | T0a, T1 |
| Q4 | **Coop** : sous, bombes et clés partagés par l'équipe (Isaac) ou par joueur ? Les bombes blessent-elles les coéquipiers ? | par joueur (comportement actuel) ; bombes : blessent le poseur et les coéquipiers | T0b, T3, T4 |
| Q5 | **Épuisement des pools** (un objet vu sort du pool) : oui ? Pour la run ou l'étage ? Il rend le tirage dépendant de l'ordre si on tire à l'ouverture. | oui, pour la run, tirage à la génération de l'étage (garde l'indépendance du moment) | T0e, T8 |
| Q6 | **Marché du diable** : chance d'apparition (fixe, ou réduite si dégâts au boss) ; prix en vie max (1 ou 2 cœurs) ; refus si mortel ; salles d'ange dans M4 ? Trésor fermé à clé à partir de quel étage ? | chance par étage en données, réduite si dégâts au boss ; refus si le paiement laisse 0 ; pas d'ange ; trésor à clé dès l'étage 2 | T5, T8 |
| Q7 | **Curses du clone** : lesquelles ? (minicarte cachée, vie cachée, noms d'objets cachés, étage agrandi ; l'obscurité demande la visibilité E8, hors M4) | les quatre sans obscurité | T0e, T9 |
| Q8 | **Salles en grille** : taille d'une case de salle (15×9 cases de 16 px proposé), une seule taille ou aussi 2×1 et L ? | 15×9, une seule taille | T0c, T0f, T5 |
| Q9 | **Noms et visuels** : noms originaux en français, sprites provisoires sous licence libre (plan des assets) ; nom du jeu `isaac` gardé comme nom de dossier ? | oui aux trois | T0f, T11 |
| Q10 | **Transformations** (N objets d'un même tag → bonus) dans M4 ? | oui, par extension des synergies (`requires_tag`) | T2, T11b |
| Q11 | **Armes non-larmes** (laser continu, couteau, tir chargé, bombes en larmes) dans M4 ? | non, sauf les bombes en larmes (une larme à `on_expire: Explode`, sans code de plus) | T1, T11 |
| Q12 | **Statuts v2** : poison et peur suffisent-ils ? Le charme (ennemi qui combat les siens) demande une IA d'allié (D5). | poison et peur ; charme hors M4 | T1 |
| Q13 | **Familiers** : tirent-ils leurs propres larmes ou celles du porteur (avec ses modificateurs) ? | leur arme, sans les modificateurs du porteur | T0d, T6 |
| Q14 | **Difficulté visée pour les bots** : quel taux de franchissement de l'étage 1 à 1 et 2 bots ? Politique du bot face au diable ? | 50 à 85 % à 1 bot, plus de 85 % à 2 ; diable pris si vie > 4 | T11d, T14, critères |
| Q15 | **Lancement** : combien de T0 de M4 maintenant, vu la règle des quatre voies et les tâches M2 ouvertes ? M4 passe-t-il avant la fin de M2 ou en recouvrement ? | une ou deux T0 en recouvrement (T0a puis T0c), le reste à la sortie de M2 | §8.1 |
| Q16 | **Exemplaires** : chaque exemplaire d'un objet a-t-il ses propres limites d'effet (Isaac), pour tous les objets ou par objet ? Changer la règle déplace des traces gungeon où un doublon est tiré. | indépendantes pour tous, preuve §5 des traces gungeon touchées | T0b, T2 |
| Q17 | **Visée** : 360° (l'engine aujourd'hui) ou 4 directions façon Isaac ? Touche de bombe (E au clavier) et bouton manette ? | 360° ; E et un bouton d'épaule | T0c, T1 |
| Q18 | **`ReloadSpeed`** (constat 1) : corriger le contenu gungeon (`Pct -0.3`) ou changer le sens de la stat (vitesse, et adapter le perk zombies) ? | corriger le contenu, garder le sens « durée » documenté | dette à créer |

## 8. Livraison

Commits sur `plan-m4-isaac`, poussés au fil de l'écriture ; commit final `LIVRÉ plan-m4-isaac` avec ce
rapport. Pas de fusion de `main` nécessaire au moment de livrer si `main` n'a pas bougé (doc seule, aucune
suite à rejouer).
