# Revue de C-5 (riposte au coup) — consigne du relecteur, 27 nov 2026

Tu es **relecteur, lecture seule** : aucun fichier modifié, aucun commit, **aucun cargo ni make**. Français partout.
Branche : `origin/wquintal/c-5-riposte-coup` (@ `0c878402` ou plus récent : le contrôle rapide tourne sur le VPS, tu n'en dépends pas), contre `origin/main`. Diff : `git diff origin/main...origin/wquintal/c-5-riposte-coup`.
À lire : brief `docs/taches/specs-coordinateur/c-5-riposte-coup.md` ; § 6 ligne **C-5** de `docs/taches/plan-1837-cli.md` ; rapport `docs/taches/rapports/c-5-riposte-coup.md` ; `docs/conventions.md` § 133 (nouveau), § 22, § 119 ; `CLAUDE.md` (champ ajouté à un composant rollback sous checksum, `FrameEvents`).

## Ce que tu vérifies
1. **Opt-in au bit près** (P1) : `BehaviorRuntime` est sous checksum ; `provoked` absent ⇒ `Hash` et `Debug` identiques à avant (champs d'avant dans le même ordre) ; aucun ennemi sans `provocation` ne change de comportement ; aucune trace existante modifiée (seules les quatre `riposte_*.trace` neuves). Le test unitaire `provocation_is_hashed_only_when_set_and_expires` le prouve-t-il vraiment ?
2. **Déterminisme** : lecture de `FrameEvents<DamageEvent>` ordonnée (émetteur avant lecteur dans `GgrsSchedule`, même frame), tie-break `net_id` si plusieurs coups la même frame, `Fixed`, aucun `f32`, aucun `Entity` loggé ; échéance en frames.
3. **Sens** : la cible est l'attaquant pendant la durée, prioritaire sur le plus proche (scénario à deux : le tireur le plus loin) ; garde `ReturnHome` levée puis retour à l'échéance ; « marche droite sur l'attaquant si le champ de flux en désigne un autre » (D241) : risque de traverser un mur ou un décor solide (C-3) ? le pas reste-t-il soumis à `check_wall_collision` ? Classe P2 si un ennemi provoqué peut traverser un mur ou une caisse `solid`.
4. **Réserves du rapport** : le tir à f20 qui ne provoque pas (« cause non élucidée ») : est-ce un défaut de la règle (coup non vu) ou de l'arme ? Lis le code, dis ce qui est le plus probable ; le `Melee("zombie_claws")` ajouté pour le lint.
5. Preuves : rouge crédible au vu des attentes ; `describe`, § 133, D241, ligne `CLAUDE.md` présents ; rien de 1837 dans l'engine.

## Rendu
`~/orca/runs/alacod/revue-c-5-verdict.md` (seule écriture permise) : verdict en une ligne (**merge** / **merge après corrections** / **ne pas merger**), puis P1/P2/P3 avec `fichier:ligne`, problème, correction. Puis `worker_done`.
