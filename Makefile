devshell:
	docker-compose run --rm chirpstack-udp-forwarder bash

test:
	docker-compose run --rm chirpstack-udp-forwarder cargo test
