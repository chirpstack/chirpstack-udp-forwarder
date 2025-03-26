{ pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixos-24.11.tar.gz") {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.cacert
    pkgs.rustup
    pkgs.protobuf
    pkgs.cargo-cross
    pkgs.cargo-deb
  ];
}
