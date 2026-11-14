{
  description = "FORGE Rust MIR verifier development environment";

  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixos-unstable/nixexprs.tar.zst";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, rust-overlay, ... }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
    in {
      devShells = nixpkgs.lib.genAttrs systems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        in {
          default = pkgs.mkShell {
            packages = [ rust pkgs.just pkgs.pkg-config pkgs.z3 pkgs.llvmPackages.llvm ];
            LD_LIBRARY_PATH = nixpkgs.lib.makeLibraryPath [ pkgs.z3 ]
              + ":${rust}/lib:${rust}/lib/rustlib/${pkgs.stdenv.hostPlatform.rust.rustcTarget}/lib";
          };
        });
    };
}
