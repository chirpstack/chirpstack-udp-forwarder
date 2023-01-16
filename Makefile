.PHONY: dist

# Update the version
version:
	test -n "$(VERSION)"
	sed -i 's/^version.*/version = "$(VERSION)"/g' ./Cargo.toml
	make test
	git add .
	git commit -v -m "Bump version to $(VERSION)"
	git tag -a v$(VERSION) -m "v$(VERSION)"

# Cleanup dist.
clean:
	rm -rf dist

# Run tests
test:
	docker-compose run --rm chirpstack-udp-forwarder cargo clippy --no-deps
	docker-compose run --rm chirpstack-udp-forwarder cargo test

# Enter the devshell.
devshell:
	docker-compose run --rm chirpstack-udp-forwarder bash
