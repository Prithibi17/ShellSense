# Maintainer: Prithibi <prithibi@cachyos>
pkgname=terminal-assistant
pkgver=0.1.0
pkgrel=1
pkgdesc="Fast, native Linux terminal command assistant and autocomplete daemon for CachyOS / Arch"
arch=('x86_64')
url="https://github.com/prithibi/terminal-assistant"
license=('MIT')
depends=('gcc-libs' 'glibc')
makedepends=('cargo' 'rust')
optdepends=(
    'fish: interactive fish shell autocomplete and bindings'
    'ollama: local AI intent interpretation backend'
    'quickshell: Wayland floating suggestion popup'
    'paru: AUR package management'
    'pacman: official repository package management'
)
source=()
sha256sums=()

build() {
    cargo build --release
}

check() {
    cargo test --release
}

package() {
    install -Dm755 "target/release/terminal-assistant" "$pkgdir/usr/bin/terminal-assistant"
    install -Dm755 "target/release/terminal-assistantd" "$pkgdir/usr/bin/terminal-assistantd"
    install -Dm644 "fish/terminal-assistant.fish" "$pkgdir/usr/share/fish/vendor_conf.d/terminal-assistant.fish"
    install -Dm644 "systemd/terminal-assistant.service" "$pkgdir/usr/lib/systemd/user/terminal-assistant.service"
    install -Dm644 "config/config.toml" "$pkgdir/etc/terminal-assistant/config.toml"
}
