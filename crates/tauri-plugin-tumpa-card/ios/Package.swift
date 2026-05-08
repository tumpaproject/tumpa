// swift-tools-version:5.3
// SPDX-License-Identifier: GPL-3.0-or-later

import PackageDescription

let package = Package(
  name: "tauri-plugin-tumpa-card",
  platforms: [
    // .v13 matches the Tauri runtime's supported iOS floor and keeps
    // swift-tools-version at 5.3. NFC smartcard APIs (NFCTagReaderSession
    // + NFCISO7816Tag) are available from iOS 13+. The host app's
    // deployment target is independent and set in tauri.conf.json /
    // Info.plist.
    .iOS(.v13),
  ],
  products: [
    .library(
      name: "tauri-plugin-tumpa-card",
      type: .static,
      targets: ["tauri-plugin-tumpa-card"])
  ],
  dependencies: [
    .package(name: "Tauri", path: "../.tauri/tauri-api")
  ],
  targets: [
    .target(
      name: "tauri-plugin-tumpa-card",
      dependencies: [
        .byName(name: "Tauri")
      ],
      path: "Sources")
  ]
)
