# P-1d — Parité observée : visée au curseur et rechargement des fantassins (WIP, rapport à compléter)

Jalon servi : **S1, dim 15** (plan `docs/taches/plan-1837-cli.md` § 6, ligne P-1d). Branche `wquintal/p-1d-parite-observee` (base `d54ab899`).

## État au 14 nov (pause cargo, batterie)

Rien n'est encore compilé ni mesuré. Copie jetable de 1837 `origin/main` (`7d78906`) : `/tmp/p1d-1837` (worktree détaché, target cloné de celui du checkout 1837), build release
interrompu par l'ordre de pause (non fini). Aucune cause n'est établie : les pistes ci-dessous sont des **hypothèses à vérifier par l'observation**, pas des conclusions.

Pistes (lecture de code seulement, à confirmer au vrai curseur) :

1. Visée : `character::input_devices::collect_device_inputs` lit la caméra par `Query<(&Camera, &GlobalTransform)>` **sans filtre** puis `q_camera.single()` ; R-1 ajoute des `Camera3d`
   (un banc par personnage, `model3d/mod.rs:224`) : avec plus d'une caméra, `single()` échoue, `mouse` vaut `None`, `pan` reste à 0 et la visée comme le regard retombent sur les touches.
2. Rechargement du fantassin : `model3d::animate::animate_models` ne lit `reloading_ending_frame` que d'un `WeaponInventory` ; le fantassin n'en a pas (son « rechargement » est le
   `cooldown_frames` de la règle `Shoot`, `ranged_cycle`), donc aucune entrée `reload`/`since_shot`/`trigger` n'atteint `pose::choose`. À vérifier : quel clip joue réellement.

## Prochaine étape

Rejouer le build release de 1837 (engine pointé sur ce worktree), fenêtre réelle, pointeur déplacé par un événement système (swift `CGEventPost`, pas de `cliclick`), trace de la chaîne.
