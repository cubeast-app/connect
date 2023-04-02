{ pkgs, ... }:

{
  packages = [
    pkgs.dbus
    pkgs.yarn
    pkgs.libsoup
    pkgs.glib
    pkgs.gdk-pixbuf
    pkgs.pango
    pkgs.gtk3
    pkgs.librsvg
    pkgs.harfbuzz
    pkgs.cairo
    pkgs.atk
    pkgs.webkitgtk

    pkgs.libappindicator
    pkgs.libappindicator-gtk3
    pkgs.libindicator-gtk3
    pkgs.libindicator
    pkgs.libayatana-appindicator
    pkgs.libayatana-indicator

  ];

  languages.rust.enable = true;
  languages.javascript.enable = true;
  languages.typescript.enable = true;
}
