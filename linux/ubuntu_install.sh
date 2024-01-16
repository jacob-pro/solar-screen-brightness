#!/usr/bin/env bash

# Install Script for Solar Screen Brightness on Ubuntu
# 1. Installs Rust Cargo (via Rustup) if necessary
# 2. Installs any required packages if necessary
# 3. Compiles and installs the latest applications (ssb and ssb-cli) from GitHub
# 4. Creates desktop and autostart entries

set -e

DESKTOP_ENTRY="${HOME}/.local/share/applications/solar-screen-brightness.desktop"
AUTOSTART_DIR="${HOME}/.config/autostart"
AUTOSTART_ENTRY="${AUTOSTART_DIR}/solar-screen-brightness.desktop"
SSB_BINARY="${HOME}/.cargo/bin/ssb"
SSB_DATA_DIR="${HOME}/.local/share/solar-screen-brightness"
ICON_PATH="${SSB_DATA_DIR}/icon.png"
SSB_CONFIG_DIR="${HOME}/.config/solar-screen-brightness"

function install_package_if_not_exists () {
  if [ $(dpkg-query -W -f='${Status}' ${1} 2>/dev/null | grep -c "ok installed") -eq 0 ]; then
    echo "Attempting to install ${1}"
    sudo apt install -y ${1};
  else
    echo "Package ${1} is already installed"
  fi
}

function refresh_desktop() {
  echo "Updating Desktop"
  xdg-desktop-menu forceupdate
}

function install() {
  if ! command -v cargo &> /dev/null; then
    echo "Rust Cargo could not be found. Attempting to install Rust..."
    curl https://sh.rustup.rs -sSf | sh -s -- -y
    source $HOME/.cargo/env
  else
    echo "Cargo found: $(cargo --version)"
  fi

  install_package_if_not_exists "libssl-dev"
  install_package_if_not_exists "libudev-dev"
  install_package_if_not_exists "libgtk-3-dev"
  install_package_if_not_exists "libxdo-dev"

  echo "Installing latest Solar Screen Brightness..."
  cargo install --git https://github.com/jacob-pro/solar-screen-brightness

  ICON_URL="https://github.com/jacob-pro/solar-screen-brightness/blob/06d57b9054ae29c5e73e491f5d073a4986f2224a/assets/icon-256.png?raw=true"

  echo "Downloading icon to ${ICON_PATH}"
  curl -sS -L --create-dirs --output "${ICON_PATH}" "${ICON_URL}"

  # Quotes https://askubuntu.com/a/1023258
  echo "Creating desktop entry ${DESKTOP_ENTRY}"
  cat <<EOT > ${DESKTOP_ENTRY}
[Desktop Entry]
Type=Application
Name=Solar Screen Brightness
Exec="${SSB_BINARY}"
Icon=${ICON_PATH}
Terminal=false
EOT

  echo "Creating autostart entry ${AUTOSTART_ENTRY}"
  mkdir -p ${AUTOSTART_DIR}
  cat <<EOT > ${AUTOSTART_ENTRY}
[Desktop Entry]
Type=Application
Name=Solar Screen Brightness
Exec="${SSB_BINARY} --minimised"
Icon=${ICON_PATH}
Terminal=false
EOT

  refresh_desktop

  echo "Successfully installed $(ssb-cli --version)"
}

function uninstall() {
  echo "Removing desktop entry"
  rm -f ${DESKTOP_ENTRY}
  echo "Removing startup entry"
  rm -f ${AUTOSTART_ENTRY}
  refresh_desktop

  if command -v ssb &> /dev/null; then
    echo "Uninstalling solar-screen-brightness"
    cargo uninstall solar-screen-brightness
  fi

  echo "Deleting config"
  rm -rf ${SSB_CONFIG_DIR}
  echo "Deleting data"
  rm -rf ${SSB_DATA_DIR}
}

if [ "$1" = "install" ]; then
    install
elif [ "$1" = "uninstall" ]; then
    uninstall
else
    echo "Usage: ./ubuntu-install.sh [install or uninstall]"
fi
