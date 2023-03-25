{ pkgs, ... }:

{
  packages = [
    pkgs.dbus
  ];
  languages.rust.enable = true;
}
