fenix:

let
  file = ./rust-toolchain.toml;
  sha256 = "eMJethw5ZLrJHmoN2/l0bIyQjoTX1NsvalWSscTixpI=";
in
{
  fromFile = { pkgs, system }:
    let
      inherit ((pkgs.lib.importTOML file).toolchain) channel;
      toolchain = fenix.packages.${system}; in
    toolchain.combine [
      (toolchain.fromToolchainFile { inherit file sha256; })
      toolchain.targets.wasm32-unknown-unknown.${channel}.rust-std
    ];

  fromTarget = { pkgs, buildPlatform, targetPlatform ? null }:
    let
      inherit ((pkgs.lib.importTOML file).toolchain) channel;
      toolchain = fenix.packages.${buildPlatform};
    in
    if
      isNull targetPlatform
    then
      fenix.packages.${buildPlatform}.${channel}.toolchain
    else
      toolchain.combine [
        toolchain.${channel}.rustc
        toolchain.${channel}.cargo
        toolchain.targets.wasm32-unknown-unknown.${channel}.rust-std
        toolchain.targets.${targetPlatform}.${channel}.rust-std
      ];
}
