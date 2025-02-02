# This file is for building on NixOS;
# You need to have direnv allow and .envrc should be:
# use nix
{ pkgs ? import <nixpkgs> {}}:

pkgs.mkShell rec {
  # https://github.com/rust-windowing/winit/issues/3244#issuecomment-1827827667

  nativeBuildInputs = with pkgs; [
    pkg-config
    alsa-lib
    openssl
  ];

  buildInputs = with pkgs; [
    libxkbcommon
    libGL
    wayland
  ];

  RUST_LOG="debug";
  LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath buildInputs}";
}

