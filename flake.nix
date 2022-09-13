{
  description = "Crafting Interpreters";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-22.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }: 
    flake-utils.lib.eachDefaultSystem (system: 

      let 
        pkgs = import nixpkgs { inherit system; };

        gcc = pkgs.gcc;
        openjdk = pkgs.openjdk;
        maven = pkgs.maven;
        antlr = pkgs.antlr;
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = [
            gcc
            openjdk
            maven
            antlr
          ];
        };
      }
    );
}
