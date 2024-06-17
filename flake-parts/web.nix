_: {
  perSystem = {
    config,
    lib,
    pkgs,
    system,
    inputs',
    self',
    ...
  }: let
    static-files = pkgs.runCommand "static-files" {} ''
      mkdir -p $out
      mkdir -p $out/public/

      cp -r ${../public}/* $out/public
      cp -r ${self'.packages.wasm}/* $out/public
    '';
  in {
    packages = {
      inherit static-files;
    };
  };
}
