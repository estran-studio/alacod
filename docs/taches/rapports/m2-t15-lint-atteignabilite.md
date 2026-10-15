# Rapport m2-t15-lint-atteignabilite (M2-T15, §53)

## 1. Fait
- `sim_core::stats::stat_reader` : liste fermée des stats lues (avec l'endroit). `UnreadStat` = erreur
  (`Luck`, `Custom` inconnu) ; **`Armor` = avertissement** (D61 non mergée).
- `content::reach` : `LintWarning` / `warnings()` ; `alacod lint` les affiche sans changer le code de sortie.
- `MapEntry::levels` (type de salle, personnages, armes murales par niveau LDtk).
- Avertissements : type de salle, gabarit, ennemi, objet, arme inatteignables (calcul statique, §53).
- Point de branchement des pools de M2-T5 : `reach::loot_pool_items` / `loot_pool_weapons` (vides ;
  à remplir en une dizaine de lignes).
- 6 fixtures et tests (`lint_fixtures.rs`), 2 tests unitaires (`reach.rs`). Audit : tableau en §53.
- Décision : pas de séquence « par défaut » (la première par ordre d'id de gungeon est `etage_1`, carte
  fixe) ; toute séquence `Floors` déclarée et `start_map` sont des racines.

## 2. Vérifié (résultats réels)
- `cargo test -p content --profile headless` : 106 réussis, 0 échec.
- `cargo test -p sim_core -p effects --profile headless` : 26 + 39 réussis.
- `alacod lint` des 4 jeux : 0 erreur ; avertissements : zombies 1, testbed 59, throne 3, gungeon 11
  (listes ci-dessous). Code de sortie 0.
- `cargo fmt --all -- --check` : OK. `check-rollback-registration.sh` : OK. `check-forbidden.sh` :
  6 occurrences préexistantes (aucune dans mes fichiers ; la fiche en annonçait 4).
- Scénarios non rejoués : aucun fichier de simulation ni de contenu touché (seuls `sim_core::stats`,
  une fonction pure, `content` et le CLI). `make gen` non lancé : aucune arme modifiée.

## 3. Non fait / non vérifié
- `make test_crates` complet (autres crates) et `make lint` via Makefile non lancés tels quels : j'ai lancé
  les crates touchées et le binaire `alacod lint` sur les 4 jeux directement.
- Dépendance de `StatId::Armor` : D61.

## 4. Avertissements par jeu (aucun contenu corrigé, à transformer en dettes)
- **zombies (1)** : ennemi `zombie_scaled` jamais placé ni appelé par une vague.
- **throne (3)** : ennemis `cible`, `mannequin` (cibles d'essai), arme `arsenal`.
- **gungeon (11)** : ennemi `cible` ; 9 objets sans source (amulette, bottes, cartouchiere, etendard, fiole,
  key, pierre_de_soin, ration, trefle) — vrai trou : aucune source d'objet avant M2-T5/T14b ; arme `arsenal`.
  Tous les types de salle de gungeon sont tirés par la grammaire `etage`.
- **testbed (59)** : 6 types de salle (la sandbox n'a aucune séquence `Floors`), 18 ennemis de test, 19 objets,
  16 armes de test ; attendus (arènes de scénarios), pas des trous.

## 5. Dettes
- Objets de gungeon sans source (ci-dessus) : à brancher via M2-T5 / T14b.
- Les scénarios ne sont pas lus par l'analyse (une arme donnée seulement par scénario est signalée).

## 6. Atteignabilité (README §7 pt 5)
Tâche de lint : aucune mécanique de jeu livrée ; la partie `gungeon` ordinaire (`etage`) n'est pas modifiée.
