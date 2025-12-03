{
  pkgs ? import <nixpkgs> { },
}:
pkgs.mkShell rec {
  libs = with pkgs; [
    # wayland
    libxkbcommon
    libGL
    wayland

    libx11
    libxcursor
    libxi

    clang
  ];

  nativeBuildInputs =
    with pkgs;
    [
      rustc
      cargo
      rust-analyzer
      python3Packages.fonttools
    ]
    ++ libs;

  # add libs to path
  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath libs}";
}
