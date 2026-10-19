# dette-d113-tir-ami-explode : tir ami de l'`Explode` (D113)

Branche `dette-d113-tir-ami-explode`, depuis `origin/main` `f80c1ea3`. Décision de William (bilan du 19) : champ `friendly_fire`, défaut d'avant.

## Changement

- `effects::Action::Explode` gagne `friendly_fire: ExplodeFriendlyFire` (`Always` | `Never`, `#[serde(default)]` = `Always`, fin de variante). Type propre (pas `sim_core::FriendlyFire`, dont le défaut est `Never` et qui a `Cursed`). **`impl Hash` manuel** : `Always` n'écrit rien (hashs d'avant intacts), `Never` écrit un octet (règle « champ nouveau sous checksum » de `CLAUDE.md` ; test unitaire).
- Exécution : `effects_runtime` (`Explode` d'effet/d'objet) et `champions::death_plan` (mort de champion) posent la `Bomb` avec ce `FriendlyFire` au lieu d'un `Always` en dur ; `Never` = aucun personnage du même bord que le porteur (porteur compris) n'est touché (`resolve_damage`, inchangé). `scaled_action` et `describe` (« épargne les alliés ») passent le champ. Lint : rien à refuser (deux valeurs valides) ; table de `conventions.md` à jour.
- Contenu : `nova`, `bombe_fiole`, `epines_de_chair` à `friendly_fire: Never` ; `GainShield(1)` retiré de `nova` et `bombe_fiole`.
- Scénario `isaac_nova_coop` (2 joueurs à 6 PV à portée, `nova` utilisée à f30) : les deux restent à 6 PV, la cible à 38 unités prend 10 (explosion bien jouée). **Contrôle** : avec les anciens objets (`nova` à `Always` + `GainShield`), même scénario : le joueur 1 meurt (santé 0 < 6 à f119).
- Point (b) (charge `Damage` comptant ses propres dégâts) : **laissé ouvert**, une phrase dans D113 : l'explosion d'un effet est créditée au porteur avec le tag `explosion`, indiscernable de sa bombe de joueur ; l'exclure bougerait les actifs à charge `Damage` qui dépendent d'une bombe ; il faut une source distincte. `nova` reste en `Rooms(2)`.

## Traces bougées (preuve §5)

Dumps `ALACOD_DUMP_TRACE` + `scripts/trace-diff.py`, même moteur, anciens objets contre objets de la branche. Première frame qui diffère = la **pose de la bombe de l'objet** (`friendly_fire: Always` → `Never`, et le `Shield` en moins pour les deux actifs) :

| Trace | Première frame |
|---|---|
| `isaac_nova` | 35 (`UseActive` f30 + délai d'input ; bombe `explode_145`, fuse 36) |
| `isaac_bombe_fiole` | 35 |
| `generated/isaac/item_nova` | 15 |
| `generated/isaac/item_bombe_fiole` | 15 |
| `generated/isaac/tears_epines_de_chair` | 2 (la bombe d'épines après un coup reçu) |
| `isaac_nova_coop` | nouvelle |

`generated/isaac/item_epines_de_chair` : identique (59 frames). `make test_scenarios_par SHARDS=3` avant BLESS : **ces cinq seulement** (+ la coop sans référence) échouaient ; BLESS ciblé sur ces six ids, dit dans le commit. Note : la première frame est la pose de la bombe, pas « l'explosion » (frame suivante) ; l'explosion est à f36 / f16 / f3.

## Vérifié (résultats réels, VPS)

- `make test_scenarios_par SHARDS=3` : vert après BLESS ciblé (avant : les cinq diffs ci-dessus).
- `make test_crates` : exit 0.
- `make lint` ×5 (zombies, testbed, throne, gungeon, isaac) : aucune erreur ; `make gen GAME=` ×5 : tout `ok`, aucune trace générée en désaccord après BLESS.
- `cargo fmt --check` ; `check_forbidden` ; `check_rollback_registration` : OK.
- `scripts/p2p-stress.sh isaac tests/scenarios/p2p_smoke.ron 3000` : 0 desync, 2999 lignes identiques, 1883 frames resimulées (client 0, 20 rollbacks, profondeur max 11). `MATCHBOX_PORT`/`COMPOSE_PROJECT_NAME` n'étaient pas exportés dans ce shell : projet `alacod`, port 3536 par défaut.
