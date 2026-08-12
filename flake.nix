{
  description = "Development environment for node-pipewire";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";

  outputs = { nixpkgs, ... }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              nodejs_24
              cargo
              rustc
              rustfmt
              clang
              llvmPackages.libclang.lib
              pkg-config
              pipewire
              cargo-audit
              shellcheck
            ];

            LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
            PKG_CONFIG_PATH = "${pkgs.pipewire.dev}/lib/pkgconfig";
            LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath [ pkgs.pipewire ]}";
          };
        });
    };
}
