# Homebrew cask for Veil.
#
# This file lives in your tap repo `Loa212/homebrew-veil` at `Casks/veil.rb`.
# It's checked in here as the template; copy it to the tap when you publish.
#
# After each release, update `version` and `sha256` (the release workflow can
# automate this later). To get the sha256 of a built dmg:
#   shasum -a 256 Veil_<version>_aarch64.dmg
cask "veil" do
  version "0.1.0"
  sha256 "REPLACE_WITH_DMG_SHA256"

  url "https://github.com/Loa212/veil/releases/download/v#{version}/Veil_#{version}_aarch64.dmg"
  name "Veil"
  desc "macOS soft lockscreen (menubar overlay + Touch ID/PIN)"
  homepage "https://github.com/Loa212/veil"

  # Veil is Apple-silicon only for now and needs macOS 15+.
  depends_on arch: :arm64
  depends_on macos: ">= :sequoia"

  app "Veil.app"

  # Clean up the app-support + login item on uninstall.
  zap trash: [
    "~/Library/Application Support/com.veil.app",
    "~/Library/Caches/com.veil.app",
    "~/Library/Preferences/com.veil.app.plist",
  ]
end
