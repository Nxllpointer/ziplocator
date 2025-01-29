{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs }: let
    pkgs = import nixpkgs { system = "x86_64-linux"; };
  in {
    devShells.x86_64-linux.default = pkgs.mkShell {
      packages = with pkgs; with xorg; [
        cargo rustc rust-analyzer rustfmt pkg-config openssl cmake
      ];
      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; with xorg; [
        libX11 libXcursor libXi libxkbcommon libxcb libXrandr vulkan-loader
      ]);
    };
  };
}
