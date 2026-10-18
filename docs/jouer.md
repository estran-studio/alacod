# Jouer à gungeon (pour William)

Toutes les commandes se lancent depuis la racine du dépôt (`alacod/`). Elles ont été essayées sur un Mac Apple
Silicon (rapport `docs/taches/rapports/m2-salles-murs-gungeon.md`, §5 et « Suivi de revue »). Ce qui n'a pas pu être
essayé est dit comme tel.

## Préparer

1. Toolchain Rust nightly choisie par `rust-toolchain.toml` (installée par `rustup` au premier `cargo`), plus les
   prérequis système de `.github/workflows/pr.yaml`.
2. Compiler une fois (≈ 20 min à froid, quelques secondes ensuite) :

   ```bash
   APP_VERSION=dev cargo build -p gungeon --features native
   ```

   Le binaire est `target/debug/gungeon` (profil `dev` optimisé : il tourne à pleine vitesse). **Ne pas déplacer le
   dépôt après la compilation** : le binaire lit `games/gungeon` par le chemin capturé à la compilation.

Les commandes `cargo run` ci-dessous recompilent au besoin ; `make gungeon` est le raccourci du solo.

## Les modes

| Je veux… | Commande | Notes |
|---|---|---|
| **Jouer seul** | `make gungeon` | Équivaut à `APP_VERSION=dev cargo run -p gungeon --features native -- --local-port 7000 --players localhost` (le Makefile ajoute aussi les features de développement de `scripts/dev-features.sh`, d'où une compilation à part). Démarre dans le **hub** : le marchand (profil, déblocages) et la porte, qui lance la run. |
| **Jouer seul, directement dans une run, toujours la même carte** | `ALACOD_SKIP_HUB=1 make gungeon GARGS="--seed 10"` | Saute le hub. `--seed <n>` fixe la graine de la carte (deux parties avec la même graine ont les mêmes salles). |
| **Deux joueurs sur le même Mac** | `APP_VERSION=dev cargo run -p gungeon --features native -- --local-port 7000 --players localhost localhost` | Deux héros dans la partie. **Limite : les deux reçoivent les mêmes touches et la même manette** (même table de touches pour chaque joueur local, voir plus bas) : utile pour voir deux héros, pas pour jouer à deux sur un clavier. Pour jouer à deux : en ligne, ci-dessous. |
| **Jouer à deux en ligne (un Mac chacun, ou deux fenêtres)** | voir « En ligne à deux » | Une machine héberge le serveur de rencontre (signaling), les deux lancent le jeu. |

**N'utilisez pas** `make gungeon GARGS="--players localhost localhost"` : le Makefile ajoute déjà un `--players localhost`,
et `--players` s'**ajoute** au lieu de remplacer : cela donne **trois** joueurs.

## En ligne à deux

Il faut Docker (ou OrbStack) sur **une** des deux machines, appelée A ci-dessous (celle dont l'adresse IP est connue de
l'autre ; même réseau local, ou adresse joignable).

1. Sur A, lancer le serveur de rencontre :

   ```bash
   docker compose -f docker-compose.ci.yaml up -d signaling
   ```

   Port 3536 par défaut ; `MATCHBOX_PORT=3570 docker compose -f docker-compose.ci.yaml up -d signaling` pour un autre port.
   Arrêt : `docker compose -f docker-compose.ci.yaml down`.
2. Sur **chaque** machine (le même salon, le même `--players`, un `--cid` et un `--name` différents) :

   ```bash
   # machine A
   APP_VERSION=dev target/debug/gungeon --number-player 2 --matchbox ws://<IP de A>:3536 --lobby salon \
     --players localhost remote --cid alice --name alice
   # machine B
   APP_VERSION=dev target/debug/gungeon --number-player 2 --matchbox ws://<IP de A>:3536 --lobby salon \
     --players localhost remote --cid bob --name bob
   ```

   (`localhost` = le héros de cette machine, `remote` = l'autre.) Quand les deux se voient, la partie démarre ; chacun
   joue son héros avec **sa** souris et **son** clavier. Pour une run précise : ajouter à chaque ligne
   `ALACOD_SKIP_HUB=1` devant la commande et `--seed 10` à la fin, **identiques des deux côtés**.
3. Le serveur public `wss://allumette.estran.studio` **ne marche pas** depuis la ligne de commande (« socket dropped » ;
   `--allumette` exige l'identifiant d'un salon créé par le site) : utiliser le `docker compose` ci-dessus.

Si les deux parties divergent, un écran « DESYNC » s'affiche (la partie commune est perdue) : notez la graine et les
frames, c'est un défaut du moteur.

## Touches

Clavier + souris. Les touches sont des **positions physiques** : sur un clavier AZERTY, W A S D sont les touches Z Q S D
et la touche « Q » (blank) est la touche A.

| Action | Touche |
|---|---|
| Se déplacer | **W A S D** |
| Viser / tirer | souris / **clic gauche** |
| Interagir (ouvrir, acheter, ramasser, relever) | **H** |
| Recharger | **R** |
| Changer d'arme / de mode de tir | **Tab** / **Z** |
| Mêlée | **F** |
| Lâcher l'arme | **L** |
| Objet actif | **Espace** |
| Blank (efface les balles proches) | **Q** |
| Bombe | **E** |
| Sprint | **Maj gauche** |
| Esquive (dash) | **C** |
| Pause / inventaire | **Échap** / **I** |
| Choix de mutation | **1 / 2 / 3** |
| Étiquettes au-dessus des entités (nom, prix, état) | **F6** |
| Légende des silhouettes et couleurs | **F7** |
| Débogage : champ de flux / profil / états des ennemis | **F3** / **F4** / **F5** |

Manette (non testée : pas de manette sur la machine d'essai) : croix directionnelle = déplacement, stick gauche = visée,
Nord = interagir, Ouest = recharger, Est = mêlée.

## Variables utiles

- `ALACOD_SKIP_HUB=1` : démarrer directement dans la run.
- `--seed <n>` (argument du jeu) : graine de la carte.
- `ALACOD_LABELS=1` : étiquettes allumées dès le départ (F6 les bascule).
- `ALACOD_MUTE=1` : aucun son.

Où regarder les parties automatiques (vidéos des bots, page de revue) : `docs/taches/rapports/m2-salles-murs-gungeon.md`, §4.
