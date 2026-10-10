# Les recettes utilisent des tableaux bash (test_multiplayer) : pas de /bin/sh (dash sur Ubuntu).
SHELL := /bin/bash

PROFILE ?= dev

LOBBY ?= "test"
# Joueurs d'une session matchbox : le local puis un `remote` par pair (voir test_multiplayer).
PLAYERS ?= localhost remote
NUMBER_PLAYER ?= 2
NAME ?= "Player"
TIMEOUT ?= 10

CURRENT_TAG := $(shell git describe --tags --exact-match HEAD 2>/dev/null)


RANDOM_SEED := $(echo $RANDOM)

LOG_DIR := ./logs
LOG_PREFFIX := game_run
FILTERED_LOG_DIR := ./logs/filtered
GREP_FILTER := 'ggrs{'
MATCHBOX_URL := wss://allumette.estran.studio


ifeq ($(CURRENT_TAG),)
	LATEST_TAG := $(shell git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
    SHORT_SHA := $(shell git rev-parse --short HEAD)
	VERSION := $(LATEST_TAG)-$(SHORT_SHA)
else
	VERSION := $(CURRENT_TAG)
endif

# Version compilée dans les binaires de test et d'outillage (`crates/game/build.rs` →
# `env!("APP_VERSION")`, affichage et chemin des assets wasm seulement). Elle doit être STABLE :
# avec `$(VERSION)` (tag + sha), chaque commit rebâtissait `game` et tout ce qui en dépend (le
# binaire de test de `scenario`, 14 Go) et laissait une génération de plus dans le target.
# `v0.0.0` = la valeur par défaut de build.rs : `make gen`, `cargo test` nu et `make
# test_scenarios` partagent ainsi un seul build. Les cibles de jeu (`make zombies`, `make
# release`) gardent `$(VERSION)`.
TEST_VERSION ?= v0.0.0


ifeq ($(PROFILE), dev)
	export MODE_DIR := debug
	# ?= : un CARGO_TARGET_DIR déjà dans l'environnement (worktree de tâche) est respecté
	export CARGO_TARGET_DIR ?= ./target
endif

ifeq ($(PROFILE), prod)
	export MODE_DIR := release
	export RELEASE := --release
endif

ifeq ($(TARGET), native)
	# ?= : un CARGO_TARGET_DIR déjà dans l'environnement (worktree de tâche) est respecté
	export CARGO_TARGET_DIR ?= ./target
endif


ifeq ($(TARGET), web)
	export RUSTFLAGS := --cfg=web_sys_unstable_apis
	export CARGO_TARGET_DIR := ./target_wasm
	ifeq ($(PROFILE), prod)
		export CARGO_TARGET_DIR := ./target_wasm_prod
	endif
endif


# ALL

all: test format

# CI check target (runs locally or in CI)
check: format test check_forbidden check_rollback_registration
	@echo "✓ All checks passed"

.PHONY: check


# Misc

clean:
	@echo "Cleaning the project..."
	@cargo clean
	@CARGO_TARGET_DIR=./target_wasm cargo clean


format:
	@echo "Vérification du formatage (cargo fmt --check)..."
	cargo fmt --all -- --check

# Formate le workspace (à lancer avant de commiter)
fmt:
	cargo fmt --all

format_fix:
	cargo fmt


# Test

test: test_scenarios
	@echo "Running tests with profile"
	cargo test

check_forbidden:
	@echo "Checking for forbidden patterns..."
	./scripts/check-forbidden.sh

# Strict (jamais en avertissement) : un appel direct rollback_component_with*/
# rollback_resource_with* hors de crates/utils/src/rollback.rs échappe au checksum GGRS.
check_rollback_registration:
	@echo "Checking rollback registration goes through RollbackTraceApp..."
	./scripts/check-rollback-registration.sh

# Lint du contenu (crates/content, T1.5) : références cassées, ids dupliqués, valeurs hors
# plage, kinds inconnus, littéraux nus dans des champs Fixed. Sans lancer le moteur.
# Code de sortie 1 (messages sur stderr) si un des deux jeux a une erreur.
lint:
	@echo "alacod lint games/zombies"
	cargo run -q -p content --bin alacod --profile headless -- lint games/zombies
	@echo "alacod lint games/testbed"
	cargo run -q -p content --bin alacod --profile headless -- lint games/testbed
	@echo "alacod lint games/throne"
	cargo run -q -p content --bin alacod --profile headless -- lint games/throne

.PHONY: lint

# Générateur de scénarios (alacod-gen ; v0 T2.10, v1 T1.13, crates/scenario/src/generate.rs) :
# un scénario par arme du registre de GAME et deux par ennemi à `test:` (immobile, mobile), dans
# tests/scenarios/generated/GAME/, joué et affiché en tableau
# (attentes, trace). Code de sortie 1 si une attente échoue, un scénario n'atteint pas sa
# dernière frame, ou une trace diffère de la référence. GEN_BLESS=1 les blesse au lieu de
# les comparer : make gen GAME=zombies GEN_BLESS=1.
GAME ?= zombies
gen:
	cargo run -q -p scenario --bin alacod-gen --profile headless -- games/$(GAME) --play $(if $(filter 1,$(GEN_BLESS)),--bless)

.PHONY: gen

# Scénarios de jeu headless (tests/scenarios/*.ron), comparés à leur trace de référence.
# BLESS=1 réécrit les traces de référence ; SCENARIO=<nom> n'en joue qu'un.
test_scenarios:
	ALACOD_BLESS=$(BLESS) ALACOD_SCENARIO=$(SCENARIO) APP_VERSION=$(TEST_VERSION) cargo test -p scenario --profile headless --test scenarios -- --nocapture

# Benchmarks : lance test_scenarios puis affiche les métriques de performance.
bench:
	ALACOD_BENCH_STRICT=1 $(MAKE) test_scenarios
	./scripts/scenario-metrics.py

# alacod-sim (T2.11) : centaines de parties bots headless par graine, métriques JSON dans
# target/metrics/sim-<commit>.json (lu par scenario-metrics.py / scenario-review.py) et affichées
# à l'écran. SEEDS=1..5 BOTS=4 WAVE=3 par défaut, pour rester court ; PROFILES=fonceur,fonceur,...
# doit avoir BOTS entrées.
SEEDS ?= 1..5
BOTS ?= 4
PROFILES ?= fonceur,fonceur,prudent,immobile
WAVE ?= 3
MAX_FRAMES ?= 20000
sim:
	mkdir -p $(CARGO_TARGET_DIR)/metrics
	cargo run -q -p scenario --bin alacod-sim --profile headless -- --game zombies --bots $(BOTS) --profiles $(PROFILES) --seeds $(SEEDS) --until-wave $(WAVE) --max-frames $(MAX_FRAMES) --json $(CARGO_TARGET_DIR)/metrics/sim-$$(git rev-parse --short HEAD).json
	./scripts/scenario-metrics.py

# Vidéos des scénarios (target/videos/<commit>/) : une par scénario + montage en grille.
# SCENARIO=<nom> pour un seul, SCENARIO=a,b ou SCENARIO="a b" pour plusieurs (D23) ;
# EVERY=N une image toutes les N frames (défaut 2) ; DRY_RUN=1 liste sans compiler ni capturer.
videos:
	EVERY=$(or $(EVERY),2) DRY_RUN=$(DRY_RUN) ./scripts/scenario-video render $(SCENARIO)
	$(if $(DRY_RUN),,./scripts/scenario-video montage)

# La même partie vue par chaque joueur, côte à côte : make views SCENARIO=four_players_shooting
views:
	EVERY=$(or $(EVERY),2) ./scripts/scenario-video views $(SCENARIO)

# Avant/après côte à côte : make compare_video SCENARIO=idle BASE=main [HEAD=<réf>]
compare_video:
	EVERY=$(or $(EVERY),2) ./scripts/scenario-video compare $(SCENARIO) $(BASE) $(or $(HEAD),HEAD)

# Page de revue des vidéos (target/videos/index.html) servie sur http://localhost:8766
# TAILSCALE=1 : sur l'IP Tailscale de la machine (accessible depuis le tailnet uniquement)
review_videos:
	./scripts/scenario-review.py --serve 8766 --bind $(if $(TAILSCALE),$$(tailscale ip -4),127.0.0.1)

# Partie pilotée à distance (scripts/alacod-remote), en pause au départ.
# HEADLESS=1 : sans fenêtre, bien plus rapide.
remote:
ifeq ($(HEADLESS), 1)
	ALACOD_HEADLESS=1 ALACOD_REMOTE=1 APP_VERSION=$(VERSION) cargo run --profile headless -p zombies --no-default-features -- --local-port 7000 --players localhost
else
	ALACOD_REMOTE=1 $(MAKE) ldtk_map_explorer
endif

# Joue au clavier et enregistre la session en scénario : make record_session NAME=ma_session
# (écrit tests/scenarios/<NAME>.ron à la fermeture ; puis make test_scenarios SCENARIO=<NAME> BLESS=1)
record_session:
	ALACOD_RECORD=$(CURDIR)/tests/scenarios/$(NAME).ron $(MAKE) ldtk_map_explorer

# Affiche un scénario avec rendu : make play_scenario SCENARIO=shoot_around
play_scenario:
	APP_VERSION=$(VERSION) cargo run -p scenario --features render --bin play_scenario -- tests/scenarios/$(SCENARIO).ron


# Env


# Dependencies

dep_web:
	rustup target add wasm32-unknown-unknown
	cargo install -f wasm-bindgen-cli --version 0.2.129

dep_format:
	rustup component add rustfmt
	rustup component add clippy

dep: dep_web dep_format

# Dev run

map_preview:
	cargo run --example map_preview $(ARGS) --features native

map_generation:
	cargo run --example map_generation $(ARGS)

map_generation_test:
	cargo run --example map_generation -- ./games/zombies/assets/exemples/test_map.ldtk ./games/zombies/assets/exemples/test_map_generated.ldtk $RANDOM_SEED

map_generation_diff_test:
	cargo run --example map_generation -- ./games/zombies/assets/exemples/test_map.ldtk ./games/zombies/assets/exemples/test_map_generated_1.ldtk $RANDOM_SEED
	cargo run --example map_generation -- ./games/zombies/assets/exemples/test_map.ldtk ./games/zombies/assets/exemples/test_map_generated_2.ldtk $RANDOM_SEED
	diff ./games/zombies/assets/exemples/test_map_generated_1.ldtk ./games/zombies/assets/exemples/test_map_generated_2.ldtk

character_tester:
	APP_VERSION=$(VERSION) cargo run --example character_tester $(ARGS) --features native -- $(GARGS) --local-port 7000 --players localhost

character_tester_matchbox:
	APP_VERSION=$(VERSION) cargo run --example character_tester $(ARGS) --features native -- --number-player $(NUMBER_PLAYER) --matchbox $(MATCHBOX_URL) --lobby $(LOBBY) --players $(PLAYERS) --cid $(CID) --name $(NAME)

# Lance un jeu du dossier games/ en local (un joueur) : make zombies / make testbed / make throne
zombies:
	APP_VERSION=$(VERSION) cargo run -p zombies $(ARGS) --features native -- $(GARGS) --local-port 7000 --players localhost

testbed:
	APP_VERSION=$(VERSION) cargo run -p testbed $(ARGS) --features native -- $(GARGS) --local-port 7000 --players localhost

throne:
	APP_VERSION=$(VERSION) cargo run -p throne $(ARGS) --features native -- $(GARGS) --local-port 7000 --players localhost

ldtk_map_explorer:
	APP_VERSION=$(VERSION) cargo run -p zombies $(ARGS) --features native -- $(GARGS) --local-port 7000 --players localhost

ldtk_map_explorer_matchbox:
	APP_VERSION=$(VERSION) cargo run -p zombies $(ARGS) --features native -- --number-player $(NUMBER_PLAYER) --matchbox $(MATCHBOX_URL) --lobby $(LOBBY) --players $(PLAYERS) --cid $(CID) --name $(NAME)

host_website:
	cd website && APP_VERSION=$(VERSION) npm run dev

cp_asset:
	mkdir -p ./website/static/$(VERSION)/assets/
	cp -r ./games/zombies/assets/* ./website/static/$(VERSION)/assets/

build_map_preview_web:
	APP_VERSION=$(VERSION) cargo build --example map_preview --target wasm32-unknown-unknown --no-default-features --features render,bevy_ecs_tilemap/atlas $(RELEASE)
	wasm-bindgen --out-dir ./website/static/$(VERSION)/map_preview --out-name wasm --target web $(CARGO_TARGET_DIR)/wasm32-unknown-unknown/$(MODE_DIR)/examples/map_preview.wasm
ifeq ($(PROFILE), prod)
	wasm-opt -Oz --vacuum ./website/static/$(VERSION)/map_preview/wasm_bg.wasm -o ./website/static/$(VERSION)/map_preview/wasm_bg.wasm
endif

build_character_tester_web:
	APP_VERSION=$(VERSION) cargo build --example character_tester --target wasm32-unknown-unknown --no-default-features --features render $(RELEASE)
	wasm-bindgen --out-dir ./website/static/$(VERSION)/character_tester --out-name wasm --target web $(CARGO_TARGET_DIR)/wasm32-unknown-unknown/$(MODE_DIR)/examples/character_tester.wasm
ifeq ($(PROFILE), prod)
	wasm-opt -Oz --vacuum ./website/static/$(VERSION)/character_tester/wasm_bg.wasm -o ./website/static/$(VERSION)/character_tester/wasm_bg.wasm
endif

build_ldtk_map_explorer_web:
	APP_VERSION=$(VERSION) cargo build -p zombies --target wasm32-unknown-unknown --no-default-features --features render $(RELEASE)
	wasm-bindgen --out-dir ./website/static/$(VERSION)/map_explorer --out-name wasm --target web $(CARGO_TARGET_DIR)/wasm32-unknown-unknown/$(MODE_DIR)/zombies.wasm
ifeq ($(PROFILE), prod)
	wasm-opt -Oz --vacuum ./website/static/$(VERSION)/map_explorer/wasm_bg.wasm -o ./website/static/$(VERSION)/map_explorer/wasm_bg.wasm
endif


build_wasm_apps:
	WEB_BUILD_ID=$(VERSION) WEB_PROFILE=$(if $(filter prod,$(PROFILE)),release,dev) node scripts/build-web-games.mjs zombies throne

build_website: build_wasm_apps
	cd website && npm ci && APP_VERSION=$(VERSION) npm run build

build_docker_website: build_wasm_apps
	docker build --build-arg APP_VERSION=$(VERSION) -f ./website/Dockerfile ./website -t ghcr.io/bascanada/alacod:latest

build_docker_builder:
	docker build --platform linux/amd64 -f ./Dockerfile.builder ./ -t ghcr.io/bascanada/alacod-builder:$(VERSION)

export_docker_website:
	docker create --name $(VERSION) ghcr.io/bascanada/alacod:latest && \
    docker cp $(VERSION):/usr/share/nginx/html ./build

# Publish
push_docker_website:
	docker push ghcr.io/bascanada/alacod:latest

push_docker_builder:
	docker push ghcr.io/bascanada/alacod-builder:$(VERSION)

print_version:
	@echo "Current Tag: $(CURRENT_TAG)"
	@echo "Version: $(VERSION)"


diff_log:
	@mkdir -p $(FILTERED_LOG_DIR)
	@# Filter logs by GGRS pattern
	@cat $(LOG_DIR)/$(LOG_PREFFIX)_$(CID_1).log | grep $(GREP_FILTER) > $(FILTERED_LOG_DIR)/$(CID_1)_raw.log
	@cat $(LOG_DIR)/$(LOG_PREFFIX)_$(CID_2).log | grep $(GREP_FILTER) > $(FILTERED_LOG_DIR)/$(CID_2)_raw.log
	@# Find the last frame in each log and take the minimum
	@LAST_FRAME_1=$$(grep -oE 'f=[0-9]+' $(FILTERED_LOG_DIR)/$(CID_1)_raw.log | tail -1 | cut -d= -f2); \
	LAST_FRAME_2=$$(grep -oE 'f=[0-9]+' $(FILTERED_LOG_DIR)/$(CID_2)_raw.log | tail -1 | cut -d= -f2); \
	if [ "$$LAST_FRAME_1" -lt "$$LAST_FRAME_2" ]; then \
		MIN_FRAME=$$LAST_FRAME_1; \
	else \
		MIN_FRAME=$$LAST_FRAME_2; \
	fi; \
	echo "Comparing logs up to frame $$MIN_FRAME ($(CID_1): $$LAST_FRAME_1, $(CID_2): $$LAST_FRAME_2)"; \
	perl -ne 'print if /f=(\d+)/ && $$1 <= '"$$MIN_FRAME" $(FILTERED_LOG_DIR)/$(CID_1)_raw.log > $(FILTERED_LOG_DIR)/$(CID_1).log; \
	perl -ne 'print if /f=(\d+)/ && $$1 <= '"$$MIN_FRAME" $(FILTERED_LOG_DIR)/$(CID_2)_raw.log > $(FILTERED_LOG_DIR)/$(CID_2).log; \
	diff $(FILTERED_LOG_DIR)/$(CID_1).log $(FILTERED_LOG_DIR)/$(CID_2).log

test_multiplayer:
	@N=$(or $(N),2); \
	PIDS=""; \
	CIDS=""; \
	PLAYER_NAMES=("alice" "bob" "charlie" "diana" "emma" "frank"); \
	echo "Starting multiplayer test with $$N players (same lobby: $(LOBBY))..."; \
	for ((i=1; i<=N; i++)); do \
		PLAYER_NAME=$${PLAYER_NAMES[$$((i-1))]}; \
		REMOTES=""; for ((j=1; j<N; j++)); do REMOTES="$$REMOTES remote"; done; \
		echo "Starting player $$i ($$PLAYER_NAME) in lobby $(LOBBY)..."; \
		make $(TARGET)_matchbox CID=$$PLAYER_NAME NAME="$$PLAYER_NAME" LOBBY=$(LOBBY) NUMBER_PLAYER=$$N PLAYERS="localhost$$REMOTES" & \
		PID=$$!; \
		PIDS="$$PIDS $$PID"; \
		CIDS="$$CIDS $$PLAYER_NAME"; \
		if [ $$i -lt $$N ]; then \
			echo "Waiting $(TIMEOUT) seconds before starting next player..."; \
			sleep $(TIMEOUT); \
		fi; \
	done; \
	echo "Waiting for all instances to complete..."; \
	wait $$PIDS; \
	echo "All instances completed"; \
	echo "Running log diffs..."; \
	FIRST_CID=$$(echo $$CIDS | awk '{print $$1}'); \
	for CID in $$CIDS; do \
		if [ "$$CID" != "$$FIRST_CID" ]; then \
			echo "Comparing $$FIRST_CID vs $$CID..."; \
			make diff_log CID_1=$$FIRST_CID CID_2=$$CID || true; \
		fi; \
	done

.PHONY: test_multiplayer

# Nightly CI pipeline (full)
nightly:
	bash scripts/nightly.sh

# Nightly CI pipeline (quick local test)
nightly_quick:
	bash scripts/nightly.sh --quick

.PHONY: nightly nightly_quick

# Immutable clone builds and release catalog (solo opt-in for previews).
.PHONY: build_web_games
build_web_games:
	node scripts/build-web-games.mjs $(WEB_GAMES)
