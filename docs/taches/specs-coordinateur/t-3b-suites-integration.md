# Tâche : T-3b « Suites d'intégration : entrées réelles par geste et mécaniques entre personnages » (outillage ; Codex permis pour le runner)

Lis `~/orca/runs/alpha/commun.md`, `CLAUDE.md`, la ligne **T-3** et la ligne **« Changement d'ordre du 16 nov (William) »** du § 6 de `docs/taches/plan-1837-cli.md` (elles font foi).
Branche `wquintal/t-3b-suites-integration`, worktree `~/orca/workspaces/alacod/t-3b-suites-integration` (partie d'`origin/main`).

Décision de William (14 nov) : « ça devrait être dans des suites de tests d'intégration qui roulent souvent, qui valident qu'il n'y a pas de régression dans les états
d'animation et les autres mécaniques inter-personnages ». Et le 16 nov : « nos tests d'intégration devraient attraper tout problème d'animation qui ne joue pas ».

## Partage avec P-1d (autre worker, en parallèle)
- **P-1d possède** le contrôle générique d'animation (invariant du runner) et le **composant de présentation** calculé hors rendu (clip, progression, fondu, cap) : tu le
  **réutilises**, tu ne le réécris pas. Son nom et son module te sont donnés par le coordinateur ; s'ils ne sont pas encore mergés, travaille d'abord sur (1) et (2) ci-dessous.
- Fichiers à éviter tant que P-1d n'est pas mergé : `crates/game/src/model3d/**`, `crates/content/src/models.rs`, le lint des poses.

## Ce que tu livres
1. **Entrées réelles par geste** : au moins un test par geste (visée, tir, épaulé, hache, pas/dash, rechargement) qui part d'un **`CursorMoved` de fenêtre** (fenêtre factice sans rendu) et
   d'un **axe de manette**, convertis par le vrai chemin d'input (`collect_device_inputs`, leafwing), jamais `PlayerScript::aim` ; champ de scénario ou harnais de test, au choix le plus simple.
2. **Mécaniques entre personnages** (attentes de scénario, en réutilisant l'existant `EnemyState`, `HitsAtLeast`, `PlayerDowned`/`PlayerRevived`, `DoorLocked`, `CoverSight`, `Event`, et en
   ajoutant ce qui manque) : cycle du fantassin face au joueur, cône qui suit, salve, coup reçu qui fait perdre la salve, balle renvoyée par la hache, relève à deux, leurre du feu follet,
   clé, couvert. Scénarios **neutres** du testbed ; chaque attente nouvelle **échoue sur une régression provoquée** (preuve dans le rapport).
3. **Roulent souvent** : un jeu court étiqueté (budget écrit, quelques dizaines de secondes) dans `make check_rapide` et une option équivalente dans `alacod test` (1837 la joue avant chaque
   merge), et la nuit.
4. Ensuite, une fois le composant de P-1d mergé : attentes fines `Clip`, `ClipProgress`, `ClipSpeed`, `ModelYaw` sur ce composant ; une entrée par ligne de la revue observée de P-1d.

Règles : aucune trace existante modifiée ; jamais de profil debug ; un seul cargo ; sweep après ; pas de suite complète sur le Mac ; commite et pousse à chaque étape (Mac souvent sur batterie :
sous 20 % sans secteur, pas de cargo) ; ne touche ni `~/Project/bascanada/alacod` ni `~/Project/bascanada/1837` ; `make check_rapide` (journal
`~/orca/runs/alpha/logs/t-3b-suites-integration-check-rapide-<sha>.log`) ; § 123, D219, rapport `docs/taches/rapports/t-3-suites-integration.md` ; « E prêt <sha> » par tranche ; un seul `worker_done`.
