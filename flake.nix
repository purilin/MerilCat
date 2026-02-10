{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        # 使用 devShells.default 是现在的标准写法
        devShells.default = pkgs.mkShell {
          # 1. 编译工具
          nativeBuildInputs = with pkgs; [
            cargo
            rustc
            rustfmt
            rustPackages.clippy
            pkg-config # 必须有它！
            rust-analyzer
          ];

          # 2. 运行时依赖库
          buildInputs = with pkgs; [
            openssl # 解决你的报错！
          ];

          # 3. 环境变量
          shellHook = ''
            export RUST_SRC_PATH="${pkgs.rustPlatform.rustLibSrc}";
            export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig";
          '';
        };
      }
    );
}
