# polissage-distribution — archives téléchargeables Mac et Windows (B2)

Lire d'abord `docs/taches/README.md`, `docs/taches/plan-polissage.md` (§5 décisions 2, 9, 10),
`.github/workflows/release.yaml` (ne construit aujourd'hui que le WASM, runner `self-hosted`), `Makefile` (`APP_VERSION`,
`TARGET`, lignes 33-40, 74-80), `docs/jouer.md`, `docs/contrat-lancement-web.md`. Branche `polissage-distribution` depuis
`origin/main`. Estimation : 2 j. Clones : les quatre. Machine : Mac (arm64) avec affichage pour lancer l'archive ;
Windows n'est pas disponible sur le VPS [INFERENCE] : build par job GitHub Actions `windows-latest`, lancé et vérifié par
un humain si William a une machine Windows.

## À faire

1. Cible `make dist GAME=<clone>` : binaire `--release` (profil existant, `--no-default-features` interdit : il faut le
   rendu), dossier `assets/` du clone, `LISEZMOI.txt` (français : lancer, Gatekeeper « clic droit → Ouvrir » (décision 9),
   touches), version `APP_VERSION`, licences (`assets.yaml`). Un zip par clone et par plateforme.
2. Le jeu doit trouver ses `assets/` à côté du binaire, pas par le répertoire de travail du dépôt ; vérifier le chemin
   d'assets natif (`games/*/src/main.rs`) et corriger si besoin.
3. Workflow `.github/workflows/dist.yaml` (sur tag `v*` et `workflow_dispatch`) : jobs `macos-14` (arm64) et
   `windows-latest` (x86-64) qui construisent, zippent et publient en GitHub Release (décision 10). Ne pas toucher à
   `release.yaml` (WASM).
4. Lancement sans terminal : le binaire sans argument démarre sur le menu de B3 ; **tant que B3 n'est pas mergé**, un
   script `jouer-<clone>.command` / `.bat` dans l'archive passe les arguments par défaut (supprimé par B3).
5. Essai sur une machine **sans** toolchain Rust : décompresser, lancer, entrer dans une partie locale.

## Acceptation

- Quatre zips Mac arm64 produits en CI (et quatre Windows si décision 2 = Windows), tailles notées dans le rapport.
- Un clone lancé depuis son zip sur une machine sans Rust, partie locale jouable avec son et sprites.
- Non vérifié = dit (signature, antivirus Windows, Intel Mac).
- Rapport `docs/taches/rapports/polissage-distribution.md`.
