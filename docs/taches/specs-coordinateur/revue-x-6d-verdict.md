# Revue X-6d — agrippeur générique `Grab` (`origin/wquintal/x-6d-agrippeur`, tête 2550feb4)

Relecteur en lecture seule (aucun cargo/make, aucun checkout) : `git diff origin/main...origin/wquintal/x-6d-agrippeur` (73 fichiers, +4539/−45), `git show`,
journaux `runs/alpha/logs/x-6d-*`. Checklist GGRS de `CLAUDE.md` appliquée.

## Verdict : **merge après corrections** (aucun bloquant dans le code ; corrections de preuve et de rapport, plus quelques interactions non couvertes)

Le code de simulation respecte la checklist GGRS. Ce qui manque : des preuves (interactions demandées au point 3 sans scénario, attentes jamais vues
rouges) et un rapport qui annonce plus qu'il ne prouve (contrôle rapide encore en cours, section citée absente).

## 1. Déterminisme GGRS (vérifié ligne à ligne, conforme)

- `grab_system` (`crates/game/src/character/enemy/ai/grab.rs`) : `order_iter!`/`order_mut_iter!` sur une requête qui commence par `&GgrsNetId` ; décompte
  des porteurs en `BTreeMap<usize, u32>` ; recherche du joueur par `net_id` unique (`iter().find`, pas de sélection arbitraire) ; `Fixed` et entiers seulement
  (le `to_num::<i32>()` ne sert qu'au journal) ; aucun RNG, aucune `HashMap`, aucun `to_bits`, aucun `despawn`.
- Journaux au format `ggrs{f=… grab net_id=… phase=…}`, porteur en `GgrsNetId`, jamais d'`Entity`.
- `BehaviorRuntime` (`ai/state.rs:770-855`) : les trois champs neufs (`grab`, `grab_ready_at`, `grab_bite`) sont hachés et listés par `Debug` **seulement
  quand ils sont actifs** (`grab_active`) : c'est la règle « champ ajouté à un type sous checksum ». `GrabPhase` a cinq variantes, donc le piège du
  discriminant d'un enum à une seule variante ne s'applique pas.
- `sim_core::grab::Riding` : type neuf enregistré avec `rollback_and_trace_neutral` (`character/mod.rs:129`), dans `BaseCharacterGamePlugin`, ajouté par
  `CoreSimPlugin` (`core_sim.rs:73`) avant les cinq enregistrements de clôture (`core_sim.rs:172`). Conforme.
- `Behavior::Grab` est ajouté en fin d'enum ; `Sway` dérive `Hash`/`Ord`.
- Ordre des sets : `EnemyAI` vient après `Movement`, `Weapon` et `CollisionDamage` (`sim_core/src/system_set.rs`). La mêlée du porteur
  (`attack_started_frame == now`) et les `DamageEvent` de mêlée de la frame sont donc visibles le jour même ; la morsure passe par
  `charge_damage_translate_system` la frame suivante (même schéma que `Charge`, l'ordre des tags de `Charge` est conservé).
- Expiration du ralentissement : `until = now + 1` et `Modifier::is_expired` vaut `until < frame` (`sim_core/src/modifier.rs:45`). Le ralentissement
  reste donc actif à la frame N+1 (lu par `apply_inputs`, avant le renouvellement), puis est ignoré et purgé par `stats/src/plugin.rs:53`. Correct.
- Aucune donnée dérivée non rollbackée n'est introduite.

## 2. Secouage (conforme)

`axes(inputs[handle].0.buttons)` lit `PlayerInputs<PeerConfig>` dans `GgrsSchedule` (`grab.rs` `GrabPhase::Ride`) ; le clavier et la manette ne sont
jamais lus directement. Le seuil (`shake`) et la fenêtre (`shake_window`, défaut 20) sont des données. Le stick et le clavier passent par les mêmes bits :
le journal des régressions montre que le stick produit des segments `[Left]`/`[Right]` (`x-6d-agrippeur-regressions.log:23-27`).

## 3. Défauts

### Bloquant
Aucun.

### À corriger
1. **Contrôle rapide non conclu alors que la tâche est déclarée LIVRÉE.** `x-6d-agrippeur-check-rapide-2550feb4.log` était encore en cours à 00:14 UTC :
   il s'arrête sur une compilation, sans code de sortie ni ligne `test_crates découpé`. Seule la ligne `check_sim_boundary` OK apparaît (l. 603).
   Pourtant la ligne `docs/taches.md` (fin) dit LIVRÉ, et le rapport `docs/taches/rapports/x-6d-agrippeur.md:22` renvoie à une « § Contrôle rapide » qui
   n'existe pas dans le fichier. Il faut attendre la fin du contrôle et écrire cette section avec son résultat.
2. **Le rapport revendique « chaque attente rouge sur régression provoquée » (point 5), mais le journal ne le montre pas pour plusieurs attentes neuves.**
   Les attentes suivantes ne sont jamais rouges sous aucun interrupteur :
   - `grab_bond_accroche` : `GrabState Idle/Crouch/Leap` (f20/46/58), `EnemyPosition` immobile pendant le ramassé (f42, f52), `EnemyPosition x_max 480`
     (f58), `Health min 100` (f100, aucun dégât au contact), `EnemyState Grab` (f119) ;
   - la **morsure** (`bite_every`) : aucun interrupteur ne la casse (`grab_duree_max` `Health 98` f240, `grab_coop` `Health min 99`) ;
   - le télégraphe immobile et la cible figée au début du bond : aucun interrupteur ;
   - `grab_deux` `Leap` f100 ; `grab_coop` `PlayerPosition` ×2.

   Il faut soit ajouter des interrupteurs (bite, telegraph/crouch, cible figée), soit réécrire la phrase du rapport et de `docs/taches.md`
   (« onze régressions provoquées ») pour dire exactement ce qui est couvert.
3. **Interactions du point 3 sans aucun scénario** (le code les traite, mais rien ne le prouve) :
   - **porteur à terre, feu follet ou mort pendant l'accroche** : la branche `Release::Carrier` (`grab.rs`, `None | Some((_, _, true))`) n'est exercée
     par aucun scénario. Le feu follet est bien couvert par le code, parce que `Wisp` est posé avec `Downed` (`combat/src/downed.rs:65-75`) ;
   - **agrippeur tué alors qu'il est accroché** (balle d'un coéquipier, par exemple) : le rapport et le § 126 affirment « un agrippeur tué ne laisse rien »
     (expiration du modificateur), sans scénario. `grab_tir_porteur` ne prouve que la face inverse ;
   - **dash du porteur** : aucune règle (l'agrippeur suit le dash et reste accroché ; on ne sait pas si le ralentissement s'applique au dash) et aucun
     scénario. C'est un choix de design à écrire dans le § 126 ou dans D222, avec un scénario qui le fige ;
   - **changement d'étage** avec un agrippeur accroché : non traité, non testé ;
   - **mur** : l'agrippeur décroché retombe à la position exacte du porteur (D222 (2)). Pour un porteur collé à un mur, le collider de l'agrippeur
     (12×12, offset −8 × 0,5) peut chevaucher le mur ; aucun scénario n'utilise `EnemyNeverInWall`. L'invariant `joueur_hors_mur` (joueurs seulement)
     n'est pas concerné.

### Mineur
4. `grab.rs` `grab_system` : `let Some(rule) = grab_rule(phase_rules) else { continue; }` passe **avant** le nettoyage. Un boss dont seule une phase
   liste `Grab` et qui change de phase pendant `Ride` garde `Riding` pour toujours : `move_enemies` le colle au porteur. Le ralentissement, lui, expire.
   Aucun contenu actuel n'est concerné. Remède : retirer `Riding` et repasser en `Idle` quand la règle a disparu.
5. `grab.rs` `GrabPhase::Leap` : le décompte `riders` ne décroît pas quand un autre agrippeur se décroche la même frame (son `remove` est différé). Un
   bond peut alors échouer à tort pendant une frame. C'est déterministe, donc sans conséquence pour le rollback.
6. `hold_slow` renouvelle `until` mais ne met pas à jour `value` si `slow` change (rechargement à chaud du contenu pendant une accroche). Négligeable.
7. `combat/src/projectile.rs` `gather_targets` : le guidage des projectiles du porteur vise encore son propre agrippeur (exclu ensuite à la collision).
   C'est déjà noté dans D222 ; les dégâts de zone à l'expiration (`projectile_expire_system`) ne filtrent pas `Riding` : à vérifier.
8. `content/src/lint.rs` (bras `BehaviorEntry::Grab`) : le contrôle d'ordre s'exécute pour **chaque** règle `Grab` d'une liste et dupliquerait le
   message s'il y en a deux. `lint_model_states` n'exige les poses `leap`/`ride`/`stunned` que pour les `behaviors` du personnage, pas pour les phases
   de boss.
9. Le rapport (point 2) écrit « mirroir » pour « miroir » (`registry.rs`) ; il cite le contenu de 1837 (`saut`, `accroche`, `etourdi`,
   `design/production/lutin.md`) dans la section « Pour 1837 » du rapport seulement. Le code, le testbed et les fixtures restent neutres : conforme à
   « rien de 1837 dans l'engine ».

## 4. Présentation (conforme, sauf ce qui n'est pas vérifiable sans rendu)

- Les poses `leap`/`ride`/`stunned` sont lues dans `decide_poses` (`grab_pose`, `f32` côté présentation seulement) et `sim_activity` donne « bond »,
  « accroché » et « étourdi » à `animation_vivante`. Le lint `MissingPose` couvre ces poses (fixtures `model_poses_grab_missing`/`_complete`).
- L'interrupteur `pose` met au rouge les attentes `Clip` et `animation_vivante` (« bond » seulement : « accroché » et « étourdi » ne sont jamais rouges
  par cet invariant).
- `ride_on_shoulders` (PostUpdate, chaînée avant `y_sort`) : l'agrippeur et son porteur partagent la même y fixe, donc le +0,05 de z survit au `y_sort`.
  Présentation seule, la position simulée n'est pas touchée.

## 5. Tests

- **Aucune `.trace` existante modifiée** : les 12 `.trace` du diff sont toutes neuves (`grab_*`).
- Trois gestes d'entrée réelle (harnais T-3b, `gestes_entree_reelle.rs`) : stick, clavier et bouton East. Ils sont rouges sous `shake`, `melee` et
  `pose` (journal, l. 20-35, 41-46, 108-125).
- Traces existantes : 107 scénarios rejoués sur un échantillon, pas la suite complète. Le rapport le dit honnêtement.

## Ce que je n'ai pas pu vérifier sans compiler ni lancer

- Que la suite complète des scénarios et `test_crates` restent verts (le contrôle rapide est en cours, voir défaut 1).
- `make schedule_order_diff` : je m'appuie sur le récit du rapport (le faux positif dû à la cible partagée, puis le dump refait) sans avoir relu
  `x-6d-schedule/`.
- Le rendu de l'ancre des épaules et des poses : la vidéo a été regardée par l'auteur, pas par moi.
- Le comportement réel du dash, du changement d'étage et de la chute contre un mur (défaut 3).
- Le p2p réel (le synctest du runner seulement, comme le dit le rapport).
