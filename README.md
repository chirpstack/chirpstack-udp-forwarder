# ChirpStack UDP Forwarder

The ChirpStack UDP Forwarder is an UDP forwarder for the [ChirpStack Concentratord](https://www.chirpstack.io/concentratord/)
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


# Concentratord configuration.
[concentratord]

  # Event API URL.
  event_url="ipc:///tmp/concentratord_event"

  # Command API URL.
  command_url="ipc:///tmp/concentratord_command"
```

## Links

* [ChirpStack homepage](https://www.chirpstack.io/)

## License

ChirpStack UDP Forwarder is distributed under the MIT license. See
[LICENSE](https://github.com/brocaar/chirpstack-udp-bridge/blob/master/LICENSE).
