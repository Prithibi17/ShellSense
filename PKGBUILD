# Maintainer: Prithibi <prithibi@cachyos>
pkgname=shellsense
pkgver=0.1.0
pkgrel=1
pkgdesc="IntelliSense-like terminal command assistant and autocomplete layer for Linux"
arch=('x86_64')
url="https://github.com/prithibi/shellsense"
license=('MIT')
depends=('gcc-libs' 'glibc')
makedepends=('cargo' 'rust')
optdepends=(
    'fish: native interactive autocomplete'
    'bash: readline keybindings and completion'
    'zsh: ZLE widget completion'
    'ollama: optional local AI intent interpretation backend'
    'paru: AUR package management integration'
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
    install -Dm755 "target/release/shellsense" "$pkgdir/usr/bin/shellsense"
    install -Dm755 "target/release/shellsensd" "$pkgdir/usr/bin/shellsensd"
    ln -sf "/usr/bin/shellsense" "$pkgdir/usr/bin/ss"

    # Shell integrations
    install -Dm644 "fish/shellsense.fish" "$pkgdir/usr/share/fish/vendor_conf.d/shellsense.fish"
    install -Dm644 "bash/shellsense.bash" "$pkgdir/usr/share/shellsense/shellsense.bash"
    install -Dm644 "zsh/shellsense.zsh" "$pkgdir/usr/share/shellsense/shellsense.zsh"

    # Systemd user service
    install -Dm644 "systemd/shellsense.service" "$pkgdir/usr/lib/systemd/user/shellsense.service"

    # Default configuration
    install -Dm644 "config/config.toml" "$pkgdir/etc/shellsense/config.toml"

    # License
    install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
