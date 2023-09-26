{
  description = "Rust development environment for macOS with rustfmt and libiconv";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

  outputs = { self, nixpkgs }: {
    devShell.x86_64-darwin = nixpkgs.legacyPackages.x86_64-darwin.mkShell {
      buildInputs = with nixpkgs.legacyPackages.x86_64-darwin; [
        darwin.apple_sdk.frameworks.Security
        zlib
        rustc
        cargo
        rustfmt
        libiconv
      ];

      # This line makes the libiconv library available to the linker
      LIBRARY_PATH = "${nixpkgs.legacyPackages.x86_64-darwin.libiconv}/lib";
    };
  };
}
