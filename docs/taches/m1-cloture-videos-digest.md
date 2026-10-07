# m1-cloture-videos-digest — vidéos d'après D51 et digest final de M1 (vague 2)

Lire d'abord `docs/taches/README.md` (agent **local**, worktree de b0, branche
`m1-cloture-videos-digest` créée depuis `origin/main`). Voie V3/doc, ½ j. Rendu vidéo = compilation
`--features render` et capture : **feu vert d'orch** avant (mémoire partagée), `CARGO_BUILD_JOBS=2`.

## Contexte

Le critère des bots de M1 est atteint à 2 bots (`docs/taches.md` §9, 2026-10-06 : 198/200, 0 soft-lock,
0 desync sur `ed8a274`). Le digest `docs/digests/m1-fin-de-vague-2.md` est encore un **brouillon**
écrit avant D41, D42, D48, D51 et les correctifs de bots ; ses vidéos (`docs/digests/videos/throne_*`)
ont été rendues sur `146d09c`, avant la correction de la dispersion (D51) qui change le ressenti de
toutes les armes, `zombies` compris.

## Décisions (fixées ici)

1. **Vidéos** sur `main` à jour : `make videos SCENARIO=throne_solo,throne_three_floors,throne_quad,clone_solo,clone_duo,clone_quad`
   puis `make views SCENARIO=throne_quad` ; copier dans `docs/digests/videos/` celles de moins de 5 Mo
   avec leurs `events.json` (remplacer les anciennes, même nom) ; plus gros : lien seulement. Vérifier
   chaque MP4 (durée = frames / 30 à `--every 2`, non vide) ; mettre à jour le commentaire
   `// À regarder :` d'un scénario si ce qu'on y voit a changé (frames), sans toucher ses attentes.
1 bis. **Captures propres** (constat de William sur les vidéos du 2026-10-05) : la ligne « Wave | Spawned |
   Kills » (overlay `WaveDebugPlugin`, forcé par `WaveDebugEnabled(true)` dans `crates/scenario/src/runner.rs`)
   et le texte de debug caméra en haut ne doivent pas apparaître dans les vidéos d'un jeu en mode `Floors`
   (ni, idéalement, dans aucune vidéo de digest) : masquer l'overlay de vagues hors mode `Waves` et le texte
   de debug pendant `--capture`, sans toucher la simulation (traces identiques). Vérifier sur une image extraite.
2. **Digest final** : retirer « brouillon » ; réécrire « En bref », « Les chiffres » (20 graines du
   brouillon gardées comme historique ; 200 graines : 149/200 le 2026-10-05 puis **198/200** le
   2026-10-06, 4 bots : chiffre du journal quand il y est, sinon « en cours »), le tableau « Ce que le
   clone sait faire » (ajouter D48, D51, bots soft-locks/armes, lead non livré), la table des critères
   (bench : laisser « à vérifier au calme par orch » ; notes du jalon précédent : « revue humaine M0 en
   attente de William ») et « Autres manques » (D4 boss, D50 pénurie — non reproduite sur la graine 76
   après D51, à rejuger —, D40b `grunt`, **rééquilibrage du 3e étage à décider** : 2 défaites sur 200).
   Chaque chiffre renvoie à sa ligne de journal. Pas de nouveau calcul.
3. Section « **Pour la revue humaine** » à la fin du digest : 5 à 8 points concrets à regarder en jouant
   (`cargo run -p throne`, avec la commande exacte), dont le ressenti des armes après D51, la difficulté
   du 3e étage, l'écran de mutation, le HUD, le restart.
4. Hors périmètre : code, contenu, traces, `docs/taches.md` (orch reporte).

## Livrer

`make videos` vert, MP4 vérifiés, digest relu ; rapport court `docs/taches/rapports/m1-cloture-videos-digest.md`.
Merger `origin/main` juste avant. `git push -u origin m1-cloture-videos-digest`, puis `SendMessage` à
`orch` : `LIVRÉ m1-cloture-videos-digest <sha> : <vidéos, taille, ce qui a changé à l'image>`.
