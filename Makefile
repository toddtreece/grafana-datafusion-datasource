RS := $(shell find ./ -name '*.rs')

ui:
	@mkdir -p dist
	cd crates/datafusion-test-datasource && yarn && yarn watch 

cp: 
	cp -r crates/datafusion-test-datasource/dist/* dist/

dist/gpx_datafusion_linux_amd64: $(RS)
	cargo build
	cp target/debug/gpx_datafusion $@
	make cp

start: stop dist/gpx_datafusion_linux_amd64 
	grf start 7.5.7

stop:
	grf stop

logs:
	docker logs -f `docker ps --last 1 --format "{{.ID}}"` | grep datafusion

.PHONY := start stop logs ui cp
.DEFAULT_GOAL := start