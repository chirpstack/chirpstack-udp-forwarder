VERSION := $(shell git describe --always |sed -e "s/^v//")

test:
	docker-compose run --rm chirpstack-udp-bridge cargo test
