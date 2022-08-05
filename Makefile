VERSION := $(shell git describe --always |sed -e "s/^v//")

devshell:
	docker-compose run --rm chirpstack-udp-forwarder bash

test:
	docker-compose run --rm chirpstack-udp-forwarder cargo test
