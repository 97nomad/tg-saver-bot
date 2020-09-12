let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
  ruststable = (nixpkgs.latest.rustChannels.nightly.rust.override {
    extensions = [
      "rust-src" "rust-analyzer-preview" "rustfmt-preview"
    ];
  });
in
with nixpkgs;
stdenv.mkDerivation {
  name = "tg-saver-bot";
  buildInputs = [
    ruststable
    pkg-config
    openssl
    cmake
    zlib
  ];
}
