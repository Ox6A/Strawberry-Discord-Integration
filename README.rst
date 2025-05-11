Strawberry Music Player Discord Integration
====================
A simple Rust program which integrates the currently playing song on the Strawberry Music Player into Discord's Rich Presence feature.
This program utilises the MPRIS DBus media interface on Linux systems to fetch media metadata.

Installation
-----
- Create a new application in the Discord Developer portal, and add your Discord Application ID in the .env file. I would suggest adding the Strawberry logo as an app icon.
.. code:: sh

  git clone https://github.com/Ox6A/Strawberry-Discord-Integration.git
  cd Strawberry-Discord-Integration
  cargo build --release
  vim .env
  ./target/debug/strawberry_rpc
