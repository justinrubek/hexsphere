{inputs, ...}: {
  perSystem = {
    config,
    pkgs,
    system,
    inputs',
    self',
    ...
  }: let
    # packages required for building the rust packages
    bevyInputs = with pkgs; [
      pkgs.llvmPackages.bintools
      pkgs.udev
      pkgs.alsaLib
      pkgs.vulkan-loader
      pkgs.xorg.libX11
      pkgs.xorg.libXcursor
      pkgs.xorg.libXrandr
      pkgs.xorg.libXi
      pkgs.libxkbcommon
      pkgs.wayland
      pkgs.clang
    ];
    extraPackages =
      [
        pkgs.pkg-config
      ]
      ++ bevyInputs;
    withExtraPackages = base: base ++ extraPackages;

    craneLib = (inputs.crane.mkLib pkgs).overrideToolchain self'.packages.rust-toolchain;

    common-build-args = rec {
      src = inputs.nix-filter.lib {
        root = ../.;
        include = [
          "crates"
          "Cargo.toml"
          "Cargo.lock"
        ];
      };

      # TODO: change the name to reflect the project
      pname = "bevy-hex-planet";

      nativeBuildInputs = withExtraPackages [];
      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath nativeBuildInputs;
    };

    deps-only = craneLib.buildDepsOnly ({} // common-build-args);

    packages = {
      default = packages.cli;
      cli = craneLib.buildPackage ({
          pname = "game";
          cargoArtifacts = deps-only;
          cargoExtraArgs = "--bin game";
          meta.mainProgram = "game";
        }
        // common-build-args);

      cargo-doc = craneLib.cargoDoc ({
          cargoArtifacts = deps-only;
        }
        // common-build-args);

      wasm = craneLib.buildPackage (rec {
          pname = "hexsphere-wasm";
          cargoArtifacts = deps-only;
          buildInputs = [
            pkgs.xorg.libxcb
            pkgs.wasm-bindgen-cli
          ];
          nativeBuildInputs = withExtraPackages [pkgs.wasm-pack pkgs.wasm-bindgen-cli];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          buildPhase = ''
            # required to enable web_sys clipboard API
            export RUSTFLAGS=--cfg=web_sys_unstable_apis

            cargo build --release --target wasm32-unknown-unknown --manifest-path=crates/game/Cargo.toml

            ${pkgs.wasm-bindgen-cli}/bin/wasm-bindgen --out-dir $out/wasm --target web target/wasm32-unknown-unknown/release/game.wasm
          '';
          installPhase = ''
            echo 'Skipping installPhase'
          '';
          doCheck = false;
        }
        // common-build-args);
    };

    checks = {
      clippy = craneLib.cargoClippy ({
          cargoArtifacts = deps-only;
          cargoClippyExtraArgs = "--all-features -- --deny warnings";
        }
        // common-build-args);

      rust-fmt = craneLib.cargoFmt ({
          inherit (common-build-args) src;
        }
        // common-build-args);

      rust-tests = craneLib.cargoNextest ({
          cargoArtifacts = deps-only;
          partitions = 1;
          partitionType = "count";
        }
        // common-build-args);
    };
  in rec {
    inherit packages checks;

    apps = {
      cli = {
        type = "app";
        program = pkgs.lib.getBin self'.packages.cli;
      };
      default = apps.cli;
    };

    legacyPackages = {
      cargoExtraPackages = extraPackages;
    };
  };
}
