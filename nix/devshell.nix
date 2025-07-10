{ pkgs }:
pkgs.mkShell {
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    pkgs.wayland
  ];
}
