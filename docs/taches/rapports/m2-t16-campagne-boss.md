# Rapport m2-t16-campagne-boss — campagne du bot explorateur contre le boss à trois phases (M2-T16 rejouée sur M2-T7)

Branche `m2-t16-campagne-boss` (depuis `origin/main` 6de77f0, qui contient M2-T7 et M2-T16). **Mesure seulement** :
ni le bot (`explorateur`), ni le boss, ni aucun contenu n'ont été modifiés. Seuls des JSON, des scripts de mesure
et ce rapport sont ajoutés. Conventions : §47.

## Méthode

`alacod-sim --game gungeon --profiles explorateur[,explorateur] --floors <séquence> --seeds 1..200 --until-floor 1
--max-frames 20000`, lots de 10 graines, un JSON par lot, chacun commité et poussé séparément
(`campagne.sh`, reprenable : saute les lots déjà présents). Quatre campagnes, `docs/taches/rapports/m2-t16-campagne-boss/` :

| Préfixe | Séquence | Bots | Contenu |
|---|---|---|---|
| `b1_*` | `etages_court_deux` (grammaire `court`×2 : départ, combat, boss) | 1 | comparable au bilan M2-T16 |
| `b2_*` | `etages_court_deux` | 2 | idem |
| `e1_*` | `etages_deux` (grammaire `etage`×2, 10 salles dont une arène exigée) | 1 | séquence « étage avec arène » |
| `e2_*` | `etages_deux` | 2 | idem |

`etage_1.ron`/`etages_grammaire.ron` (un seul étage, victoire à la mort du boss) ne se prêtent pas à `--until-floor 1` ;
`etages_deux` est la séquence de l'étage `etage` qui a un étage 2 à atteindre. Le boss est le `gatling` de M2-T7
(120 PV, trois phases, renfort `bullet_kin` en phase 3). Mesures : `analyse.py` (agrégats JSON) et `morts.py`
(rejoue une graine avec `--log` et lit la dernière salle verrouillée et le tueur ; sorties `morts_*.tsv`).

## Résultats

| Campagne | Étage 2 atteint | Morts | Soft-lock | Desync | Frames (objectif) min / méd. / max |
|---|---|---|---|---|---|
| M2-T16 d'origine, 1 bot (ancien boss) | 177 / 200 | 23 | 0 | 0 | 653 / 764 / 825 |
| M2-T16 d'origine, 2 bots | 200 / 200 | 0 | 0 | 0 | 469 / 614 / 679 |
| **b1** `etages_court_deux`, 1 bot | **195 / 200** | 5 (2,5 %) | 0 | 0 | 512 / 692 / 967 |
| **b2** `etages_court_deux`, 2 bots | **200 / 200** | 0 | 0 | 0 | 432 / 581 / 698 |
| **e1** `etages_deux`, 1 bot | **0 / 200** | 200 (100 %) | 0 | 0 | — |
| **e2** `etages_deux`, 2 bots | **93 / 200** | 105 (52,5 %) | 2 | 0 | 1020 / 1697 / 3167 |

**Desyncs : 0 sur les 800 parties** (le premier lot a été rejouée deux fois par accident : mêmes résultats
au bit près, seuls les temps d'horloge changent).

### b1 : `etages_court_deux`, 1 bot (comparaison directe avec M2-T16)

- 23 morts → **5** (2,5 %, très sous le seuil de 15 %). Les 200 graines visitent les 3 salles de l'étage 0.
- Morts par salle et par frame (`morts_b1.tsv`) : **les 5 dans la salle du boss**, aux frames 446 (graine 50, étage 0)
  et 611-612 (graines 10, 54, 150, 180). Avant : 23 morts toutes à la frame 591 face au boss.
- Lecture des journaux (graines 50 et 54) : le boss est à **12 PV sur 120** (phase 3, « rage » sous 10 %) ;
  le bot est tué par le `bullet_kin` de renfort que cette phase fait apparaître, pas par les rafales du `gatling`.
  Le bot perd donc les combats serrés de la dernière phase, pas l'éventail ni la spirale.

### b2 : `etages_court_deux`, 2 bots

200 / 200, 0 mort, 0 soft-lock, 0 desync ; 36 dégâts pris en médiane (66 à 1 bot).

### e1 et e2 : `etages_deux` (étage `etage`, 10 salles avec arène)

- **e1 : 200 morts sur 200**, toutes à l'étage 0, entre les frames 320 et 1171 (médiane 558). **Aucune au boss ni à
  l'arène** : le bot ne les atteint jamais. Au moment de la mort, 3 à 5 salles visitées sur 10 et 1 à 3 nettoyées.
  Type de salle de la dernière salle verrouillée (`morts_e1.tsv`) : 176 `combat`, 24 sans salle verrouillée
  (mort peu après le nettoyage, tirs encore en vol, comme la graine 1 : salle nettoyée f554, mort f557).
  Tueur : `bullet_kin` dans 184 cas ; 16 non extraits.
- **e2 : 93 / 200**, 105 morts (frames 598 à 2433, médiane 1089, quand le tueur est identifiable), 2 soft-locks.
  Par type de salle (`morts_e2.tsv`) : 78 `combat`, 28 sans salle verrouillée, **1 `boss`** (graine 163, f1412,
  même mécanisme qu'en b1 : renfort `bullet_kin`), 0 `arene`. 28 tueurs non extraits (mort par saignement après
  une mise à terre, non vérifié).
- **Soft-locks e2 : graines 3 (frame 2170) et 175 (frame 1929).** Graine 3 inspectée : le joueur 0 est à terre
  depuis la frame 863 (mort par saignement prévue à la 2663), le joueur 1, vivant et à une vingtaine de pixels,
  ne le réanime pas et ne progresse plus (4 salles nettoyées sur 5 visitées, 10 au total). C'est un blocage de
  comportement du bot (réanimation / choix de but), pas du boss. La graine 175 n'a pas été inspectée.
- **L'arène n'est pas jouée par le bot.** Sur une graine réussie à 2 bots (graine 5), seules des salles `combat`
  et `boss` passent à `Locked`, jamais l'`arene` exigée par la grammaire `etage` : l'explorateur la contourne
  (la salle n'est pas dans ses buts) ; le boss est vaincu avec 6 salles visitées sur 9-10. **Cette campagne ne mesure donc
  pas l'arène**, ni la boutique ni la récompense.

## Conclusion

- Avec le nouveau boss, `etages_court_deux` passe de 177 à 195 réussites à 1 bot (23 → 5 morts), sans soft-lock
  ni desync ; à 2 bots toujours 200 / 200. Le boss à trois phases est **plus facile à battre** pour le bot que l'ancien.
- Sur la séquence de l'étage `etage`, le taux de mort est **très supérieur à 15 %** (100 % à 1 bot, 52,5 % à
  2 bots), mais **le boss n'y est pour presque rien** (1 mort sur 305 au boss/à l'arène) : ce sont les salles de
  combat intermédiaires (`bullet_kin`, 6 dégâts par balle, 3 ennemis par salle) qui tuent le bot avant qu'il arrive au boss.

## Réglage proposé (non appliqué, taux de mort > 15 % sur `etage`)

Hors périmètre de cette mesure (bot et boss figés), à décider par l'équipe contenu :
1. **Équilibrage des salles de combat de la grammaire `etage`** (cause mesurée) : retirer un `bullet_kin` par salle de
   combat, ou baisser leur dégât, ou ajouter des `salles de récompense` (soin) sur le chemin. Objectif : passer
   sous 15 % de morts à 1 bot, à rejouer par la même campagne (`campagne.sh e 1 etages_deux`).
2. **Bot** (hypothèse, non testée ici) : ne pas viser 1 bot sur ces étages (profil conçu pour `court`), ou lui faire
   utiliser soin/blank plus tôt ; corriger le soft-lock de réanimation (graines 3, 175 à 2 bots).
3. **Boss** : le réglage n'est pas nécessaire (2,5 % de morts à 1 bot). Si on veut le rendre encore plus clément,
   c'est la phase 3 (renfort) qui concentre les 6 morts au boss mesurées.
4. **Couverture** : faire visiter l'arène à l'explorateur pour que la campagne mesure vraiment l'arène.

## Limites et non vérifié

- La séquence `grand` n'est pas couverte. Les tueurs « ? » (16 à e1, 28 à e2) et la graine 175 (soft-lock)
  ne sont pas expliqués. Le bilan « par salle » est par **type** de salle : les identifiants de salle changent
  d'une graine à l'autre dans une grammaire.
- Mesures de temps (fps) prises sous charge (deux ou trois campagnes en parallèle) ; non utilisables comme bench.
- Non fait ici : p2p à deux clients, bench strict, vidéos, `make test_scenarios` (aucun fichier de simulation touché).
- Cloud : toolchain nightly et paquets de la CI installés à froid (`apt-get update` nécessaire avant l'installation).
