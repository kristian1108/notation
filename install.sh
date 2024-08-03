#!/bin/bash

check_os() {
  case "$(uname -s)" in
    Darwin)
      OS="darwin"
      ;;
    Linux)
      OS="linux"
      ;;
    *)
      echo "Unsupported OS"
      exit 1
      ;;
  esac
}

create_notation_toml() {
  local notation_dir="$1"
  local toml_file="${notation_dir}/Notation.toml"

  if [ ! -f "${toml_file}" ]; then
    cat <<EOL > "${toml_file}"
[notion]
# this is the notation secret from your installed connection
secret = ""
# this is the title of the page that will host your new documentation
parent_page = ""
EOL
    echo "Created Notation.toml in ${notation_dir}"
  else
    echo "Notation.toml already exists in ${notation_dir}"
  fi
}

get_latest_release() {
  local repo=$1
  curl -s "https://api.github.com/repos/${repo}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
}

download_asset() {
  local url=$1
  local output_path=$2
  sudo curl -L -o "${output_path}" "${url}"
}

suggest_path() {
  local notation_dir="$1"

  echo
  echo
  echo "Successfully installed Notation! Next steps..."
  echo
  echo "(1) You need to add the notation directory to your PATH. Run this command:"
  echo "export PATH=\$PATH:$notation_dir"
  echo
  echo "(2) You can also edit the config in $notation_dir/Notation.toml"
  echo
  echo "Enjoy!"
}

main() {
  REPO="kristian1108/notation"
  check_os

  LATEST_RELEASE=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest")
  if [[ "$OS" == "darwin" ]]; then
      ASSET_URL=$(echo "${LATEST_RELEASE}" | grep -Eo 'https://[^\"]*darwin[^\"]*')
    else
      ASSET_URL=$(echo "${LATEST_RELEASE}" | grep -Eo 'https://[^\"]*x86[^\"]*')
    fi

  if [[ -z "${ASSET_URL}" ]]; then
    echo "No asset found for ${OS}"
    exit 1
  fi

  FILENAME=$(basename "${ASSET_URL}")

  NOTATION_DIR="$HOME/.notation"
  mkdir -p "${NOTATION_DIR}"

  OUTPUT_PATH="${NOTATION_DIR}/notation"

  echo "Downloading ${FILENAME}..."
  download_asset "${ASSET_URL}" "${OUTPUT_PATH}"
  echo "Downloaded ${FILENAME} to ${OUTPUT_PATH}"

  sudo chmod +x "${OUTPUT_PATH}"

  suggest_path "${NOTATION_DIR}"
  create_notation_toml "${NOTATION_DIR}"
}

main
