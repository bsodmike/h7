TARGET=thumbv7em-none-eabihf
.DEFAULT_GOAL := all

all: core mkapp applib apps

core:
	cd h7 && cargo make build-release

mkapp:
	cd h7-mkapp && cargo build --release

applib:
	cd h7-applib && cargo build --release --features alloc,c-api

apps:
	for app in $(shell ls -d h7-apps/*/); do cd $${app} && cargo make && cd ../..; done

clean:
	rm -rf dist
	rm -rf h7/gen
	rm -rf h7-applib/dist
	for app in $(shell ls -d h7-apps/*/); do rm -rf $${app}/dist; done

clean-target:
	rm -rf $(shell find -type d | grep "target$$")

dist: clean all
	mkdir -p dist
	mkdir -p dist/rom
	cp $${CARGO_TARGET_DIR:-h7/target}/${TARGET}/release/h7 dist/rom/h7.elf
	mkdir -p dist/applib
	cp $${CARGO_TARGET_DIR:-h7-applib/target}/${TARGET}/release/libh7_applib.a dist/applib/libh7.a
	cp h7-applib/h7-app.ld dist/applib/
	cp h7-applib/dist/h7.h dist/applib/
	mkdir -p dist/apps
	for app in $(shell find h7-apps/ -maxdepth 1 -type d | cut -c 9-); do mkdir -p dist/apps/$${app} && cp h7-apps/$${app}/dist/* dist/apps/$${app}; done

dist-docker:
	docker run -it --volume=$$(pwd):/home/circleci/project olback/cortex-m:latest make dist
