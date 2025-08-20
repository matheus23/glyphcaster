{
  description = "gtk";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-24.11";
    nixpkgs-unstable.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    command-utils.url = "github:expede/nix-command-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      nixpkgs-unstable,
      flake-utils,
      command-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        unstable = import nixpkgs-unstable { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          name = "gtk";
          nativeBuildInputs = with pkgs; [
            direnv
            glib
            cairo
            pango
            # atkmm
            # gdk-pixbuf
            gtk4
            graphene
            gtksourceview5
            pkg-config
            bashInteractive # In an effort to fix the terminal in NixOS: (https://www.reddit.com/r/NixOS/comments/ycde3d/vscode_terminal_not_working_properly/)
          ];

          shellHook = '''';
        };
      }
    );
}
