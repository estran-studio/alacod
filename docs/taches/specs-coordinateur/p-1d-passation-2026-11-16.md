# Passation P-1d (lun 16 nov ~18:40 UTC, coordinateur alacod) — session neuve

Spec : `~/orca/runs/alpha/p-1d-parite-observee.md` (fait foi, avec sa section « Ordre (studio, sam 14) ») ; preuve : ligne P-1d du § 6 de `docs/taches/plan-1837-cli.md`.
Branche `wquintal/p-1d-parite-observee`, worktree `~/orca/workspaces/alacod/p-1d-parite-observee`. L'ancienne session (arrêtée samedi sur un 429 Anthropic) est fermée.

## Où on en est
- **Tranche 1 (visée au curseur) : mergée** (`main` `85c86745`, puis `00fc97f1` avec M-7). 1837 `main` `9e7c6df` l'embarque.
- **Verdict de William (lun 16, sur 1837 `9e7c6df`)** : « mon personnage a les animations correctes et complètes, mais pas les ennemis ». La visée et le Frère sont **validés** ;
  restent **les fantassins** (rechargement, et tout ce que la revue observée a trouvé sur eux).
- Poussé **non compilé** après la tranche 1 :
  - `8dae053c` défaut 2 : le tireur posté joue ses gestes (épaulé, en joue, tir, rechargement complet une fois étiré sur la durée) ;
  - `35f1ed00` tranche 3 : le corps prend la plus proche des directions du modèle ;
  - `ae88a4f7` tranche 3 : gestes étirés sur la durée de la simulation (coup, épaulé, tir), fondu ;
  - `6bd0f6c2` : pose « hit » (coup reçu d'un tireur posté, stupeur), pas de gigue repartant au premier clip.
- Captures : `~/orca/runs/alpha/logs/p-1d-captures/` (défaut 1 avant correctif) ; un script de séance 1 est écrit dans le worktree, non lancé.

## Ce que tu fais, dans l'ordre
1. **Tranche des fantassins, seule** : `git merge origin/main` (`00fc97f1`, M-7), compile, fais passer les tests ; garde dans cette tranche ce qui touche le fantassin
   (défaut 2 + pose `hit` + ce que la revue a trouvé sur eux) ; si `35f1ed00`/`ae88a4f7` touchent aussi le joueur, vérifie que le Frère validé par William ne change pas
   (sinon sépare). Test qui échoue avant / passe après par écart ; **capture côte à côte banc / jeu du cycle complet du fantassin** : épauler, cône, tir, recul, rechargement,
   retour (séances courtes, `caffeinate -d -- <commande>`, fenêtre fermée et pointeur rendu après chaque séance) ; `make check_rapide` ; « E prêt <sha> » **tout de suite**.
2. Ensuite seulement : le reste de la revue observée (spec, section « Ordre », point 2) et le texte des lignes de `parite-labo.md` pour 1837.
- Mac **sur batterie** (~40 %) : commite et pousse à chaque étape ; si `pmset -g batt` dit « Battery Power » sous 20 %, pousse et arrête cargo. Si le coordinateur dit SUSPENDS :
  plus de fenêtre, de pointeur ni de cargo.
- Règles : jamais de profil debug ; un seul cargo ; sweep après ; pas de suite complète sur le Mac ; ne touche ni `~/Project/bascanada/alacod` ni `~/Project/bascanada/1837`
  (copie jetable `/tmp/p1d-1837` pour reproduire) ; un seul `worker_done` à la toute fin.

## PRIORITÉ UNIQUE (studio, lun 16, 14 h 50 NST) : établir le fait des fantassins AVANT de compiler la tranche 3
Précision de William, qui change le diagnostic : « je n'ai rien vu comme animation sur les fantassins », sur 1837 `9e7c6df` (engine `85c86745`), en natif par `./jouer.sh`.
Pas « rechargement incomplet » : **aucune animation**.
1. Dans le **vrai jeu en release avec fenêtre**, sur **1837 `9e7c6df` exactement** (copie jetable `/tmp/p1d-1837` remise sur ce commit, engine épinglé tel quel = `85c86745`,
   pas ton worktree) : les **quatre fantassins de la salle** jouent-ils un clip, lequel, à quels moments du cycle (attente, épauler, en joue, tir, recul, rechargement, retour) ?
   Observe par l'écran (séance courte, `caffeinate -d --`) et par l'état de présentation (journal, bevy_remote : `AnimationPlayer` des fantassins, clip actif, temps).
2. Départage **par l'observation** trois hypothèses :
   a) ils n'ont **jamais animé dans la salle** (modèle chargé mais pose figée, ou clips non résolus par `models.ron` de 1837 : son commentaire dit le fantassin « sans arme dans la
      simulation pour l'instant ») ;
   b) **régression** entre l'engine `9e937158` (1837 `7d78906`, où William voyait un rechargement incomplet) et `85c86745` (rejoue `7d78906` si besoin) ;
   c) les captures de P-1/R-1 montraient un **fantassin isolé** (`arene_vide`, `fantassin_cycle`), pas ceux de la salle.
3. Envoie **tout de suite** un `status` « cause fantassins : … » dès que c'est établi (le coordinateur le relaie au studio et, si la cause est dans les données de 1837,
   au coordinateur 1837).
4. Le test qui échoue avant porte sur **les fantassins de la salle réelle de 1837** (son RON, sa carte, son manifeste), pas sur un fantassin de banc.
5. La revue des autres états attend. Ne compile pas la tranche 3 telle quelle avant d'avoir établi la cause.

## CHANGEMENT D'ORDRE (William, lun 16, 15 h NST ; plan § 6, `ec067999`)
« Peu importe ma réponse, nos tests d'intégration devraient attraper tout problème d'animation qui ne joue pas ; déjà que vous devez y aller de mon observation, ça ne marche pas. »
**Le test porte la correction.** Avant de livrer la correction des fantassins (`4373c05d` reste poussée, mais elle n'est livrée qu'avec ce qui suit) :
1. **Contrôle générique d'animation** (nom à choisir, ex. `animation_vivante`) : dans les scénarios de la **vraie salle de 1837**, pour **chaque personnage qui a un modèle**, à
   **chaque changement d'état de la simulation** (attente, épauler, en joue, tir, recul, rechargement, retour, touché, mort), un clip attendu est **résolu**, **joué**, et sa
   **progression avance** ; un personnage dont l'état change sans que le clip joué change ou avance, ou qui n'a aucun clip, est un **échec**. Générique : tout personnage, tout
   scénario, tout jeu, **sans attente écrite à la main** (invariant du runner, comme `sante_bornee`, désactivable par scénario). Doit tourner **sans fenêtre ni GPU** : la décision
   du clip et sa progression doivent être calculées hors rendu (déplace hors d'`animate_models` ce qu'il faut : collecte de `PoseInput`, `pose::choose`, fondu, temps du clip,
   en un composant de présentation écrit aussi en headless ; `animate_models` ne fait plus qu'appliquer). **Il doit échouer sur 1837 `9e7c6df`** (fantassins figés sur `fusil-idle`) :
   preuve dans le rapport (sortie de l'échec), puis passer avec ta correction.
2. **Lint** : `alacod lint` échoue si une pose de `models.ron` ne se résout pas dans le manifeste (`anime.json`), ou si un état que la simulation peut prendre n'a aucune pose
   (ex. un personnage qui tire par `Shoot` sans pose de tir ; le fantassin « sans arme dans la simulation »). Fixtures rouges/vertes.
3. Les deux entrent dans `make check_rapide` de l'engine et dans `alacod test` de 1837 (invariant actif par défaut) **dès cette tranche**.
4. **Livré** = couvert par la suite **et** observé par toi dans le vrai jeu (capture côte à côte du cycle complet du fantassin). Plus aucune question de diagnostic à William.
5. Envoie un `status` « contrôle : <nom> rouge sur 9e7c6df » dès qu'il échoue comme attendu. Un autre worker (T-3b) fera les entrées réelles par geste et les mécaniques entre
   personnages : il **réutilisera ton composant de présentation** ; garde son nom et son module stables, et dis-les dans ce status.
Batterie basse : pousse à chaque étape ; sous 20 % sans secteur, pas de cargo.
