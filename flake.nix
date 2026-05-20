{
  description = "Rank and encode";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        dbName = "rankode";
        pgData = "./pgdata";
        pgPort = "5433";
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "rankode";
          version = "0.1.0";
          src = ./.;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.cargo
            pkgs.rustc
            pkgs.clippy
            pkgs.ffmpeg
            pkgs.postgresql_16
            pkgs.rustfmt
            pkgs.rust-analyzer
            pkgs.tree-sitter
          ];

          shellHook = ''
            export PGDATA="${pgData}"
            export PGPORT="${pgPort}"
            export PGHOST="/tmp"
            export DATABASE_URL="postgresql://localhost:${pgPort}/${dbName}"

            # Initialise le cluster si nécessaire
            if [ ! -d "$PGDATA" ]; then
              echo "Initialisation du cluster PostgreSQL..."
              initdb --auth=trust --no-locale --encoding=UTF8
            fi

            # Démarre PostgreSQL si pas déjà en cours
            if ! pg_isready -q; then
              echo "Démarrage de PostgreSQL..."
              pg_ctl start -l "$PGDATA/postgresql.log" -o "-k /tmp"
              # Crée la base si elle n'existe pas
              createdb ${dbName} 2>/dev/null || true
            fi
          '';
        };
      }
    );
}
