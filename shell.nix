{ pkgs ? import <nixpkgs> {}}:

pkgs.mkShell rec {
  # https://github.com/rust-windowing/winit/issues/3244#issuecomment-1827827667

  buildInputs = with pkgs; [
    libxkbcommon
    libGL
    wayland
  ];

  RUST_LOG="debug";
  OPENSSL_DIR="${pkgs.lib.getBin pkgs.openssl.dev}";
  PKG_CONFIG_PATH="${pkgs.lib.getBin pkgs.alsa-lib.dev}/lib/pkgconfig";
  OPENSSL_LIB_DIR="${pkgs.lib.getBin pkgs.openssl.out}/lib";
  LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath buildInputs}";
}

