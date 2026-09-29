PROFILE ?= dev

LOBBY ?= "test"
NUMBER_PLAYER ?= 2
NAME ?= "Player"
TIMEOUT ?= 10

CURRENT_TAG := $(shell git describe --tags --exact-match HEAD 2>/dev/null)


RANDOM_SEED := $(echo $RANDOM)

LOG_DIR := ./logs
LOG_PREFFIX := game_run
FILTERED_LOG_DIR := ./logs/filtered
GREP_FILTER := 'ggrs{'
MATCHBOX_URL := wss://allumette.bascanada.org


ifeq ($(CURRENT_TAG),)
	LATEST_TAG := $(shell git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
    SHORT_SHA := $(shell git rev-parse --short HEAD)
	VERSION := $(LATEST_TAG)-$(SHORT_SHA)
else
	VERSION := $(CURRENT_TAG)
endif


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
check: format test check_forbidden
	@echo "✓ All checks passed"

.PHONY: check


# Misc

clean:
	@echo "Cleaning the project..."
	@cargo clean
	@CARGO_TARGET_DIR=./target_wasm cargo clean


format:
	@echo "Running fmy..."
	cargo fmt --all -- --emit=files

format_fix:
	cargo fmt


# Test

test: test_scenarios
	@echo "Running tests with profile"
	cargo test

check_forbidden:
	@echo "Checking for forbidden patterns..."
	./scripts/check-forbidden.sh

# Scénarios de jeu headless (tests/scenarios/*.ron), comparés à leur trace de référence.
# BLESS=1 réécrit les traces de référence ; SCENARIO=<nom> n'en joue qu'un.
test_scenarios:
	ALACOD_BLESS=$(BLESS) ALACOD_SCENARIO=$(SCENARIO) APP_VERSION=$(VERSION) cargo test -p scenario --profile headless --test scenarios -- --nocapture

# Benchmarks : lance test_scenarios puis affiche les métriques de performance.
bench:
	ALACOD_BENCH_STRICT=1 $(MAKE) test_scenarios
	./scripts/scenario-metrics.py

# Vidéos des scénarios (target/videos/<commit>/) : une par scénario + montage en grille.
# SCENARIO=<nom> pour un seul ; EVERY=N une image toutes les N frames (défaut 2).
videos:
	EVERY=$(or $(EVERY),2) ./scripts/scenario-video render $(SCENARIO)
	./scripts/scenario-video montage

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
	ALACOD_HEADLESS=1 ALACOD_REMOTE=1 APP_VERSION=$(VERSION) cargo run --profile headless --example map_explorer --no-default-features -- --local-port 7000 --players localhost
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
	cargo run --example map_generation -- ./assets/exemples/test_map.ldtk ./assets/exemples/test_map_generated.ldtk $RANDOM_SEED

map_generation_diff_test:
	cargo run --example map_generation -- ./assets/exemples/test_map.ldtk ./assets/exemples/test_map_generated_1.ldtk $RANDOM_SEED
	cargo run --example map_generation -- ./assets/exemples/test_map.ldtk ./assets/exemples/test_map_generated_2.ldtk $RANDOM_SEED
	diff ./assets/exemples/test_map_generated_1.ldtk ./assets/exemples/test_map_generated_2.ldtk

character_tester:
	APP_VERSION=$(VERSION) cargo run --example character_tester $(ARGS) --features native -- $(GARGS) --local-port 7000 --players localhost

character_tester_matchbox:
	APP_VERSION=$(VERSION) cargo run --example character_tester $(ARGS) --features native -- --number-player $(NUMBER_PLAYER) --matchbox $(MATCHBOX_URL) --lobby $(LOBBY) --players localhost remote --cid $(CID) --name $(NAME)

ldtk_map_explorer:
	APP_VERSION=$(VERSION) cargo run --example map_explorer $(ARGS) --features native -- $(GARGS) --local-port 7000 --players localhost

ldtk_map_explorer_matchbox:
	APP_VERSION=$(VERSION) cargo run --example map_explorer $(ARGS) --features native -- --number-player $(NUMBER_PLAYER) --matchbox $(MATCHBOX_URL) --lobby $(LOBBY) --players localhost remote --cid $(CID) --name $(NAME)

host_website:
	cd website && APP_VERSION=$(VERSION) npm run dev

cp_asset:
	mkdir -p ./website/static/$(VERSION)/assets/
	cp -r ./assets/* ./website/static/$(VERSION)/assets/

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
	APP_VERSION=$(VERSION) cargo build --example map_explorer --target wasm32-unknown-unknown --no-default-features --features render $(RELEASE)
	wasm-bindgen --out-dir ./website/static/$(VERSION)/map_explorer --out-name wasm --target web $(CARGO_TARGET_DIR)/wasm32-unknown-unknown/$(MODE_DIR)/examples/map_explorer.wasm
ifeq ($(PROFILE), prod)
	wasm-opt -Oz --vacuum ./website/static/$(VERSION)/map_explorer/wasm_bg.wasm -o ./website/static/$(VERSION)/map_explorer/wasm_bg.wasm
endif


build_wasm_apps: cp_asset build_map_preview_web build_character_tester_web build_ldtk_map_explorer_web

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
	echo "Starting multiplayer test with $(N) players..."; \
	for ((i=1; i<=N; i++)); do \
		PLAYER_NAME=$${PLAYER_NAMES[$$((i-1))]}; \
		LOBBY_ID="$$i"; \
		echo "Starting player $$i ($$PLAYER_NAME)..."; \
		make $(TARGET)_matchbox CID=$$PLAYER_NAME NAME="$$PLAYER_NAME" LOBBY="test_$$LOBBY_ID" NUMBER_PLAYER=$$N & \
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
