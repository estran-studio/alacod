# m2-t6-boutique — boutique et monnaie `shells` (M2, V1b, chantier C5)

Lire d'abord `docs/taches/README.md` (machine `orca`, §1 : `make sweep`, `make test_crates`,
signaling par session), `CLAUDE.md`, `docs/conventions.md` §12 (monnaie `Currency`,
`CurrencyEvent`, achats, anti-rebond), §36 et §44 (objets, synergies), §48 (butin, tables, flux RNG
`"loot"`, M2-T5), §35 (salles typées : `room_kind`, types ouverts), §40 (grammaire d'étage, type
`boutique`), §15 (HUD, source `prompt`). Branche `m2-t6-boutique` depuis `main` (M2-T5 mergée).
Estimation : 2 j. Section de conventions réservée : **§50**.

## Contexte

`Currency` existe (zombies : points au kill, portes et armes murales payantes). `gungeon` n'a ni
monnaie qui tombe ni boutique. Dans Gungeon, les ennemis lâchent des douilles (`shells`), dépensées
dans une salle boutique où les objets sont posés sur des présentoirs à prix.

## À faire

1. **Monnaie au sol** : à la mort d'un ennemi, `shells` lâchés selon une donnée de l'ennemi ou de
   l'économie du jeu (`drop: (min, max)`, tirage par un flux RNG **dédié**, jamais celui du
   gameplay), ramassés au contact (comme un consommable) et crédités à `Currency` avec
   `CurrencyEvent` (raison `pickup`). Entité au sol rollback, `GgrsNetId` par clé de contenu,
   `despawn_rollback()`.
2. **Salle boutique** : type de salle `boutique` (fichier `rooms/boutique.ron`, déjà prévu par la
   grammaire de M2-T10 si présent) : à la génération, `n` présentoirs tirés dans une table de M2-T5
   (`loot/boutique.ron`, même flux `"loot"` indexé par salle) ; chaque présentoir : objet + prix
   (prix par rareté en donnée, `economy.ron` de gungeon). La salle n'est jamais verrouillée.
3. **Achat** : Interaction devant un présentoir → `Currency::spend(prix)` puis l'objet est donné
   comme s'il était ramassé (passif, actif, consommable : même chemin que `handle_item_interaction`)
   ; solde insuffisant : refus, `CurrencyEvent { delta: 0 }` ; anti-rebond comme les armes murales.
   Présentoir vidé (état rollback **neutre**). Prompt HUD « Acheter Trèfle — 30 ».
4. **Lint** : prix ≤ 0, boutique sans table, table de boutique vide, `drop` min > max → refusés ;
   fixtures.
5. **Contenu `gungeon`** : `shells` sur `bullet_kin` et les ennemis existants (via un ennemi ou une
   donnée de jeu qui ne change pas les traces si possible : sinon preuve §5), une salle boutique
   dans la grammaire `etage` si elle reste valide sur 1 000 graines (test de M2-T10), prix.
6. **Scénarios** : `shells_drop` (mort d'un ennemi → `Currency` augmente au ramassage),
   `shop_buy` (achat, `HasItem`, `Currency` débitée), `shop_refused` (solde insuffisant, rien ne
   change), `shop_seeded` (même graine → mêmes présentoirs).

## Règles

Aucune trace existante ne doit bouger hors `gungeon` (nouveaux types en variante **neutre**,
variantes d'enum en fin, champs optionnels hors hash quand absents) ; toute trace qui bouge :
preuve §5 (`ALACOD_DUMP_TRACE` main / branche + `trace-diff.py`), cause dans le commit, avant tout
bless. RNG uniquement par des flux dédiés. Pas de `f32`, `order_iter!`, `FrameEvents` (jamais
`Message`), BTreeMap. `cargo fmt --all -- --check` avant de livrer.

## Hors périmètre

Marchand PNJ et déblocages méta (M2-T12, M2-T13), vol en boutique, réduction de prix par objet,
HUD des shells (M2-T18).

## Vérification

README §4 (`make test_scenarios`, `make test_crates`, lint ×4, fmt, scripts, `make gen` ×4) ;
p2p à deux clients sur `gungeon` (port 3556, `MATCHBOX_PORT`) ; rapport
`docs/taches/rapports/m2-t6-boutique.md`, conventions §50. Livrer par
`LIVRÉ m2-t6-boutique <sha>`.

## Session cloud (ajout du 2026-10-15)

M2-T5 est mergée (`m2-t5-butin-coffres`, §48) : la tâche part en session Claude Code cloud au lieu
de b1. Ignorer les consignes `orca` du README (§1 : `make sweep`, ports de signaling) ; pas de
Docker : le p2p gungeon est rejoué par l'orchestrateur, le dire au rapport. Atteignabilité
(`docs/taches.md` §7) : la boutique doit être tirée par la grammaire `etage` d'une partie gungeon
ordinaire (nommer la graine `alacod-sim` qui la rencontre) ; `alacod lint gungeon` sans nouvel
avertissement. Pousser la branche régulièrement (commits WIP acceptés ; le conteneur redémarre
environ toutes les deux heures : suite au premier plan). Voie rapide du README §6 : fusionner
`origin/main` et rejouer la suite juste avant le commit `LIVRÉ m2-t6-boutique`.
