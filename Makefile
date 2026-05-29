SHELL := /bin/sh
COMPOSE ?= docker compose
COMPOSE_FILE ?= deploy/compose/docker-compose.yml
PROJECT ?= zotero-s3-webdav
PROVISIONER_IMAGE ?= zotero-s3-webdav-provisioner:dev
DOCKER_BUILD_NETWORK ?= default

.PHONY: help fmt clippy test validate build docker-build up up-edge up-e2e provision down logs smoke clean

help:
	@printf '%s\n' \
	  'Targets:' \
	  '  fmt           Check rustfmt' \
	  '  clippy        Run clippy with warnings denied' \
	  '  test          Run Rust tests' \
	  '  validate      fmt + clippy + test' \
	  '  docker-build  Build the Rust provisioner image' \
	  '  up            Start SFTPGo and run provisioner' \
	  '  up-edge       Start SFTPGo + optional Caddy edge profile' \
	  '  up-e2e        Start SFTPGo + MinIO local test double' \
	  '  provision     Re-run provisioning once' \
	  '  smoke         Run a WebDAV smoke test via curl' \
	  '  down          Stop stack' \
	  '  logs          Tail logs'

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

validate: fmt clippy test

build:
	cargo build --workspace --locked

docker-build:
	docker build --network=$(DOCKER_BUILD_NETWORK) -f apps/provisioner/Dockerfile -t $(PROVISIONER_IMAGE) .

up:
	DOCKER_BUILD_NETWORK=$(DOCKER_BUILD_NETWORK) $(COMPOSE) --project-name $(PROJECT) --env-file .env -f $(COMPOSE_FILE) up -d --build

up-edge:
	DOCKER_BUILD_NETWORK=$(DOCKER_BUILD_NETWORK) $(COMPOSE) --project-name $(PROJECT) --env-file .env -f $(COMPOSE_FILE) -f deploy/compose/docker-compose.edge.yml up -d --build

up-e2e:
	DOCKER_BUILD_NETWORK=$(DOCKER_BUILD_NETWORK) $(COMPOSE) --project-name $(PROJECT) --env-file .env -f $(COMPOSE_FILE) -f deploy/compose/docker-compose.e2e.yml up -d --build

provision:
	$(COMPOSE) --project-name $(PROJECT) --env-file .env -f $(COMPOSE_FILE) run --rm provisioner

smoke:
	./deploy/scripts/smoke-webdav.sh

down:
	$(COMPOSE) --project-name $(PROJECT) --env-file .env -f $(COMPOSE_FILE) down

logs:
	$(COMPOSE) --project-name $(PROJECT) --env-file .env -f $(COMPOSE_FILE) logs -f --tail=200

clean:
	rm -rf target apps/provisioner/target
