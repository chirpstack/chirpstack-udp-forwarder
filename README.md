# ChirpStack UDP Forwarder

The ChirpStack UDP Forwarder is an UDP forwarder for the [ChirpStack Concentratord](https://www.chirpstack.io/docs/chirpstack-concentratord/index.html)
and is compatible with the [Semtech UDP protocol](https://github.com/Lora-net/packet_forwarder/blob/master/PROTOCOL.TXT).

## Configuration

Configuration example:

```toml
# UDP Forwarder configuration.
[udp_forwarder]

  # Log level.
  #
  # Valid options are:
  #   * TRACE
  #   * DEBUG
  #   * INFO
  #   * WARN
  #   * ERROR
  #   * OFF
  log_level="INFO"

  # Log to syslog.
  #
  # When set to true, log messages are being written to syslog instead of stdout.
  log_to_syslog=false

  # Prometheus metrics bind.
  #
  # E.g. '0.0.0.0:9800', leave blank to disable the metrics endpoint.
  metrics_bind="0.0.0.0:9800"


  # Servers to forward the data to using UDP.
  # This section can be repeated.
  [[udp_forwarder.servers]]
    # Server (hostname:port).
    server="localhost:1700"

    # Keepalive interval (seconds).
    #
    # In this interval, the ChirpStack UDP Forwarder will send keepalive
    # frames to the server, which must be answered by an acknowledgement.
    keepalive_interval_secs=10

    # Max. allowed keepalive failures.
    #
    # After the max. number has been reached, the ChirpStack UDP Forwarder will
    # 're-connect' to the server, meaning it will also re-resolve the DNS in case
    # the server address is a hostname.
    keepalive_max_failures=12

	# Forward CRC OK.
	forward_crc_ok=true

	# Forward CRC invalid.
	forward_crc_invalid=false

	# Forward CRC missing.
	forward_crc_missing=false


# Concentratord configuration.
[concentratord]

  # Event API URL.
  event_url="ipc:///tmp/concentratord_event"

  # Command API URL.
  command_url="ipc:///tmp/concentratord_command"
```

## Links

* [ChirpStack homepage](https://www.chirpstack.io/)

## Building from source

### Requirements

Building ChirpStack UDP Forwarder requires:

* [Nix](https://nixos.org/download.html) (recommended) and
* [Docker](https://www.docker.com/)

#### Nix

Nix is used for setting up the development environment which is used for local
development and for creating the binaries.

If you do not have Nix installed and do not wish to install it, then you can
use the provided Docker Compose based Nix environment. To start this environment
execute the following command:

```bash
make docker-devshell
```

**Note:** You will be able to run the test commands and run `cargo build`, but
cross-compiling will not work within this environment (because it would try start
Docker within Docker).

#### Docker

Docker is used by [cross-rs](https://github.com/cross-rs/cross) for cross-compiling,
as well as some of the `make` commands.

### Starting the development shell

Run the following command to start the development shell:

```bash
nix-shell
```

Or if you do not have Nix installed, execute the following command:

```bash
make docker-devshell
```

### Running tests

Execute the following command to run the tests:

```bash
make test
```

### Building binaries

Execute the following commands to build the ChirpStack UDP Forwarder binaries
and packages:

```bash
# Only build binaries
make build

# Build binaries + distributable packages.
make dist
```

## License

ChirpStack UDP Forwarder is distributed under the MIT license. See
[LICENSE](https://github.com/brocaar/chirpstack-udp-forwarder/blob/master/LICENSE).
