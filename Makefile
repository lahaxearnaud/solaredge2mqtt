SHELL := /bin/bash
help:            ## Show this help.
	@fgrep -h "##" $(MAKEFILE_LIST) | fgrep -v fgrep | sed -e 's/\\$$//' | sed -e 's/##//'
.DEFAULT_GOAL:= help

start: ## Start app
	cargo run

watch: ## Start cargo watch
	cargo watch -x run

build: ## Build release app
	cargo build -r --all-features

build-docker: build ## Build docker image
	docker build . -t lahaxearnaud/solaredge2mqtt

publish-docker: build-docker ## Publish docker image
	docker push lahaxearnaud/solaredge2mqtt

lint: ## Lint code
	cargo clippy --fix
