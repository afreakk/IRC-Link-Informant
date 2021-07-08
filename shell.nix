{ pkgs ? (import <nixpkgs> {}) }:
pkgs.mkShell {
  buildInputs = with pkgs; [ cargo carnix openssl];
  nativeBuildInputs = [pkgs.pkg-config];
}
