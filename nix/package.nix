{ pkgs }:
pkgs.rustPlatform.buildRustPackage (finalAttrs: {
  pname = "automatic-redshift";
  version = "0.1.0";

  src = ../.;

  cargoHash = "sha256-8wbBi1+IzUTZWFAISCmUmHIApCZw+Aue7o7Nnzql8aE=";

  meta = {
    mainProgram = "automatic-redshift";
  };
})
