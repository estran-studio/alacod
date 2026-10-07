# M1 — revue humaine de `throne` (réservée à William)

Dernier critère de sortie de M1 (`docs/plan-engine.md` §9.8) avec la revue de M0 (T3.3, en cours sur
le MacBook). Une à deux heures : jouer, puis regarder les vidéos. Notes dans
`tests/review-notes/m1-<commit>.md` (une ligne par constat : où, quoi, gravité bloquant / à corriger /
idée), ou directement en dettes si c'est net.

## Jouer

```bash
git pull && make throne ARGS="--profile headless"    # solo, manette ou clavier (profil optimisé : D30)
```

À deux en local : mêmes arguments que `make throne` avec `--players localhost localhost`
(`--number-player 2` si l'option est demandée) ; en ligne : `docs/jouer-a-deux.md`.

La run : trois cavernes générées par graine, portail quand l'étage est vidé, boss `roi_rat` au 3e étage,
rads → niveau → écran de mutation (trois cartes), horloge d'étage. Deux runs au moins, dont une
jusqu'au boss.

## À juger (les bots ne savent pas le dire)

1. **Armes après D51** (2026-10-06) : la dispersion de la config est enfin appliquée ; avant, toutes les
   armes sauf le fusil tiraient à ±29°. Le revolver et le laser sont maintenant précis, la mitraillette
   garde une gerbe. Est-ce que chaque arme a un caractère ? Le changement vaut aussi pour `zombies`
   (`make zombies ARGS="--profile headless"`) : le clone M0 est-il devenu trop facile ?
2. **Difficulté du 3e étage** : les bots finissent 198/200 depuis D51 (39 défaites sur 200 la veille).
   Options sur la table : munitions garanties (D50), retirer l'`arroseur` de `niveau_3`, ou remonter
   la pente de difficulté. Ou rien si à la main c'est déjà assez dur.
3. **Boss `roi_rat`** : lisible ? (couronne de tirs, télégraphe au sol). Les phases (D4) sont en M2.
4. **Écran de mutation** : les trois cartes se lisent-elles vite, le texte est-il clair (généré depuis les
   effets) ?
5. **HUD** : rads, niveau, munitions par type, étage ; ce qui manque ou gêne.
6. **Feedback** (T1.17) : hit stop, secousse, flash, chiffres de dégâts — trop, pas assez ?
7. **Cavernes** : variété d'une graine à l'autre, roche destructible, surfaces.
8. **Restart** (fin de run → rejouer) et, si possible, une partie à deux en ligne (`docs/jouer-a-deux.md`).

## Vidéos

`docs/digests/videos/` (après m1-cloture-videos-digest : rendues sur `main` d'après D51) ou
`make review_videos TAILSCALE=1`. Points à voir listés à la fin de `docs/digests/m1-fin-de-vague-2.md`.

## Décisions attendues en même temps

D28 (dégât direct des ennemis), D29 (multiplicateurs par vague du clone), D30 (profil de dev), D34
(CORDIC près de 2π), D40b (`grunt` sans attaque), D50 / rééquilibrage du 3e étage, et la relecture de
`docs/taches/m2-plan-brouillon.md` avant de lancer M2.
