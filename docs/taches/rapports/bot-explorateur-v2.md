# Rapport bot-explorateur-v2 — l'explorateur joue l'arène, passe en récompense et boutique, relève en premier (D65)

Branche `bot-explorateur-v2` depuis `origin/main` d2e3cef2, fusionnée avec `origin/main` 246b2464 (D61) juste avant la
livraison. Outillage seulement : la simulation ne change pas. Conventions : §47 (complété).

## 1. Fait

- **Arène** (`crates/bots/src/explorer.rs`, `room_pending`) : une salle `arene` dormante et vide est un but jusqu'à son
  nettoyage. Elle ne se verrouille qu'à l'entrée d'un joueur : avant, `Dormant` sans ennemi la sortait des buts.
- **Récompense, boutique** (et départ) : buts jusqu'à ce qu'un joueur y entre, sans rien acheter. Il n'y a rien à lire
  dans l'état pour une salle qui ne verrouille pas : `ExplorerVisits` (ensemble `(étage, salle)`, ressource hors
  rollback) est **la seule mémoire** du bot, écart assumé à « mémoire : aucune » (§47). Elle ne produit que des inputs,
  que GGRS enregistre. Ordre de visite inchangé (coût du chemin, puis identifiant), le boss reste en dernier.
- **Réanimation prioritaire** (`input.rs`) : l'explorateur relève un coéquipier à terre sans attendre les 600 dernières
  frames (`surfaces_any`) ; `prudent` et `fonceur` gardent `REVIVE_URGENT_FRAMES`.
- **Zone morte** (`surface_step_with`, `settle`) : un pas sous `ROUTE_DEAD_ZONE` sur les deux axes compte comme arrivé.
  Réservé à l'explorateur : appliqué à tous, il faisait bouger `bench_floor` (joué par `fonceur`, f1877).
- `docs/conventions.md` §47, scripts `campagne.sh` (macOS, sans `flock`), `arene.py`, `analyse.py` (dossier en argument),
  `scripts/test-scenarios-sharded.sh` (`DYLD_LIBRARY_PATH`, sinon la suite parallèle échoue sur macOS).
- Tests unitaires : `salle_a_visiter` (nouvelle). Aucune trace existante n'a bougé : pas de bless.

## 2. Graine 175 et graine 3 (soft-locks de réanimation, 2 bots, ancien code)

Graine 175 : le joueur 0 est à terre depuis f659 (saignement f2459). Le joueur 1, vivant et sans ennemi éveillé, ne le
relevait pas : la réanimation n'était permise que dans les 600 dernières frames (f1859). Première interaction f1887,
interrompue f1890 ; le détecteur « aucun kill depuis 600 frames » a tranché à f1929 (soft-lock). Le bot attendait
l'urgence. La graine 3 a la même cause. Les deux passent maintenant (objectif, f3357 et f3276 ± selon la fusion).
Pas de cause ni de réparation autre que l'ordre des buts.

**Soft-lock trouvé en route, graine 47 à 2 bots** (ancien code corrigé, avant la campagne finale) : joueur 0 à terre,
joueur 1 vivant à 36 px de lui, immobile 600 frames. `approach` rendait un pas non nul mais sous la zone morte
(`dead_zone` le ramenait à zéro) : ni déplacement, ni Interaction. Corrigé (zone morte ci-dessus), la graine passe.

## 3. Campagnes (`docs/taches/rapports/bot-explorateur-v2/`, 200 graines, lots de 10 poussés un par un)

Binaire de la fusion avec main 246b2464. `--until-floor 1 --max-frames 20000`.

| Campagne | Étage 2 atteint | Morts | Soft-lock | Desync | Avant (m2-t16) |
|---|---|---|---|---|---|
| b1 `etages_court_deux`, 1 bot | 195 / 200 | 5 | 0 | 0 | 195 / 200 (identique) |
| b2 `etages_court_deux`, 2 bots | 200 / 200 | 0 | 0 | 0 | 200 / 200 (identique) |
| e1 `etages_deux`, 1 bot | 0 / 200 | 200 | 0 | 0 | 0 / 200 |
| e2 `etages_deux`, 2 bots | **133 / 200** | 67 (33,5 %) | **0** | 0 | 93 / 200, 105 morts, 2 soft-locks |

- b1 et b2 : `etages_court_deux` n'a ni arène, ni boutique, ni récompense ; mêmes résultats graine par graine.
- e1 : tous morts, frames 352 / 605 (méd.) / 1668 (avant 320 / 558 / 1171) ; 3 salles visitées et 1 nettoyée en médiane
  (comme avant). Le taux de mort de `etages_deux` relève de **D64** (salles de combat trop mortelles) : chiffré, pas traité.
- e2 : 0 soft-lock (2 avant), 0 desync, 133 réussites (93 avant), sans toucher au contenu.
- **Arène verrouillée puis nettoyée**, `arene.py` (rejouée avec `--log`, graines 1 à 30 à 2 bots) : verrouillée sur 28
  graines, **nettoyée sur 23**. Exemples : graine 1 (verrou f45, nettoyée f945), graine 2 (f48, f948), graine 3 (f1527,
  f2427) ; la nettoyée dure 900 frames (horloge de la salle). Les 5 autres verrouillées sont des défaites en arène ou
  après (graines 4, 5, 14, 23, 24). À 1 bot (graines 1 à 10) : verrouillée 3 fois, nettoyée 0 (défaites f363, f535, f857).
- Récompense et boutique visitées : `rooms.visited == total` sur l'étage 0 pour les 133 parties réussies de e2 ; non
  vérifié par une trace spécifique du passage, seul le décompte du JSON.

## 4. Vérifié

- `cargo fmt --all -- --check` : rien. `cargo test -p bots` : 69 réussis (68 avant + 1).
- **Suite complète sur la fusion avec main 246b2464** (`make test_scenarios_par`, 4 tranches) : 4 tranches ok, 223 s.
  Un premier passage de la fusion avait fait échouer `bench_floor` (f1877) : cause, le correctif de zone morte
  appliqué à tous les profils ; réduit à l'explorateur, `bench_floor` repasse (`ALACOD_SCENARIO=bench_floor`).
- `make test_crates` : tous réussis, 0 échec. `make lint` ×4 : aucune erreur (gungeon : 29 avertissements préexistants).
- `check-forbidden.sh` : 6 avertissements, aucun dans `crates/bots`. `check-rollback-registration.sh` : OK.

## 5. Non vérifié / non fait

- p2p à deux clients (Docker), bench strict : hors portée ici (README « variante cloud », macOS sans Docker).
- `make gen` ×4 : non rejoué, aucune arme ni contenu modifié.
- Les tueurs et le détail des morts de e1/e2 (67 et 200) ne sont pas extraits (`morts.py` non adapté à macOS).
- La première moitié de l'exécution a reposé sur des campagnes faites avant la fusion de D61 ; elles ont été
  refaites sur la fusion (résultats identiques, ci-dessus).

## 6. Dettes, atteignabilité

- **D64** reste ouverte : salles de combat de `etage` trop mortelles (e1 100 % de morts, e2 33,5 %) ; mesure seulement.
- Le bot dort sur l'arène dans les 5 parties de 30 : les défaites en arène (900 frames de bullet_kin) relèvent de D64.
- Atteignabilité : `alacod-sim --game gungeon --bots 2 --profiles explorateur,explorateur --floors etages_deux --seeds 1..1
  --until-floor 1` joue l'arène (verrou f45, nettoyée f945) ; graines 175 et 47 pour la réanimation.
