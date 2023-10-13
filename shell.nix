{pkgs ? import <nixpkgs> {}}:
with pkgs;
  mkShell rec {
    nativeBuildInputs = [pkg-config];
    buildInputs = [
      udev
      xorg.libX11
      vulkan-loader
    ];
    LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
  }
