RS := $(shell find ./ -name '*.rs')
PLUGIN_DIR := crates/datafusion-test-datasource

ui:
	cd ${PLUGIN_DIR} && yarn && yarn build
	cp -R ${PLUGIN_DIR}/dist/* dist/
	cp -R ${PLUGIN_DIR}/README.md dist/
	cp -R ${PLUGIN_DIR}/LICENSE dist/
	cp -R ${PLUGIN_DIR}/dashboards dist/dashboards

dist/gpx_datafusion_linux_amd64: $(RS)
	cargo build --target x86_64-unknown-linux-gnu
	cp target/x86_64-unknown-linux-gnu/debug/gpx_datafusion $@

start: clean ui dist/gpx_datafusion_linux_amd64
	docker-compose up -d
	@echo "grafana started: http://localhost:3000"

stop:
	docker-compose down -v --remove-orphans

logs:
	docker-compose logs -f

bash:
	docker-compose exec grafana bash

clean: stop
	rm -rf dist
	mkdir -p dist

setup:
	@cargo --version || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
	cargo target add x86_64-unknown-linux-gnu

.PHONY := start stop logs ui clean setup
.DEFAULT_GOAL := start