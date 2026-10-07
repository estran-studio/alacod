# m1-cloture-videos-digest — vidéos d'après D51 et digest final de M1 (vague 2)

## État en cours

- Fini : vidéos rendues sur `0638b01` (`main` `c074576` + la branche), captures propres, digest final,
  « Pour la revue humaine ». Rien de béni, aucune trace changée.

## Fait

- **Captures propres** (décision 1 bis), présentation seule :
  - `game::waves::debug` : l'overlay « Wave | Spawned | Kills » ne s'affiche qu'en mode de run
    `Waves` (`Run` lu en `Option`) ; le runner de scénarios active `WaveModeEnabled` partout, d'où
    la ligne dans les vidéos de `throne`.
  - `game::camera::ui::CameraDebugUiEnabled` (défaut vrai) : visibilité du texte de debug caméra.
  - `scenario::runner::capture` coupe les deux (`WaveDebugEnabled(false)`,
    `CameraDebugUiEnabled(false)`) : aucune vidéo de digest ne les montre.
  - Vérifié sur deux images extraites à 30 s (`throne_solo`, `clone_solo`) : ni ligne de vagues, ni
    texte « Camera: … » ; seul reste l'étiquette de la vidéo (nom, commit, frame) ajoutée à l'encodage.
- **Vidéos** : `make videos SCENARIO=throne_solo,throne_three_floors,throne_quad,clone_solo,clone_duo,clone_quad`,
  `make views SCENARIO=throne_quad` ; durées = frames / 60 s pour les six (61,7 / 83,3 / 56,7 /
  50,8 / 77,8 / 76,7 s) ; copiées dans `docs/digests/videos/` avec leurs `events.json` : throne ×3,
  `clone_solo`, `clone_duo` ; **liens seulement** : `clone_quad` (5,2 Mo), `montage` (7,2 Mo),
  `throne_quad.vues` (13,7 Mo). L'ancien `clone_quad.mp4` (`4f0d550`, d'avant D51) est retiré du
  dépôt pour ne pas passer pour l'actuel.
- **Commentaires « À regarder »** mis à jour (frames seulement, attentes intactes) : `clone_solo`
  (vagues 2/3/4 à f771/f1556/f2292, kills par vague), `clone_duo` (vagues f1071/f2254/f3499, kills),
  `clone_quad` (joueur 0 à terre f69) ; les trois `throne_*` étaient déjà à jour (m1-v3-bots-armes).
- **Digest** `docs/digests/m1-fin-de-vague-2.md` : « brouillon » retiré ; En bref ; chantiers
  complétés (navigation par gabarit, D42/D43, D48, D51, restart, bots v1 suite, lead non livré, D53,
  dettes) ; 200 graines (149/200 → 198/200 à 2 bots, 200/200 à 4 bots) ; bench atteint après D53 ;
  critères ; autres manques (rééquilibrage du 3e étage, D50 à rejuger, D4, D40b) ; Vidéos ;
  « Pour la revue humaine » (7 points, `make throne`). Conflit de merge avec la ligne bench d'orch sur
  `main` : résolu en gardant la version de la branche (mêmes chiffres, ligne « oui »).

## Vérifié

- Suite complète des scénarios sans bless : **0 trace différente** (1 025 s).
- `game` lib 95 tests, `cargo fmt --check`. Compilation `--features render` (rendu) faite.
