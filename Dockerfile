FROM ubuntu:26.04 as builder

ENV DEBIAN_FRONTEND=noninteractive

ENV TZ=Europe/Berlin
ENV LANG=en_US.UTF-8
ENV LC_ALL=en_US.UTF-8

RUN    apt-get update \ 
    && apt-get install -y --no-install-recommends  \
       git \
       bash \
       curl \
       build-essential \
       zig \
       ffmpeg \
       7zip \
       jq \ 
       poppler-utils \
       fd-find \
       ripgrep \
       fzf \
       zoxide \
       imagemagick \
       pkg-config \
       libfreetype6-dev \ 
       libfontconfig1-dev \
       wget \
       ca-certificates \
       gnupg \
       software-properties-common \
       nodejs \
       npm \
       python3 \
       python3-pip \
       golang-go \
       clang \
       lldb \
       llvm-dev \
       unzip \
       zip \
       locales \
       tzdata 

RUN    sed -i '/en_US.UTF-8/s/^# //g' /etc/locale.gen \ 
    && locale-gen en_US.UTF-8 \
    && update-locale LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8 \
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain
RUN    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# build directory
WORKDIR /tmp

# Build Helix 
# not pre-built binary because it is built with 
# a helix default runtime directory pointing to ~/.config/helix/runtime
# which is not suitable for a containerized dev-environment with the 
# helix config shared with the host as it fills the config with runtime files.
ENV HELIX_DEFAULT_RUNTIME=/usr/share/helix/runtime
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN git clone https://github.com/helix-editor/helix \
    && cd helix \
    && CARGO_BUILD_JOBS=2 cargo install \
       --profile opt \
       --config 'build.rustflags=["-C", "target-cpu=native"]' \
       --path helix-term \
       --locked

# Build fancy-cat with zig
# RUN mkdir /tmp/fancy-cat \
#     && cd /tmp/fancy-cat \ 
#     && git clone --recursive https://github.com/freref/fancy-cat.git . \
#     && zig build --release=small

# RUN apt-get install -y \
#     unzip \
#     python3 \
#     clang \
#     llvm-dev \
#     libclang-dev

# # Build tdf with rust
# RUN mkdir /tmp/tdf \
#     && cd /tmp/tdf \ 
#     && git clone https://github.com/itsjunetime/tdf.git . \
#     && cargo build --release




# Go tools - copy only binaries
RUN go install golang.org/x/tools/gopls@latest
RUN go install github.com/go-delve/delve/cmd/dlv@latest
RUN go install github.com/open-policy-agent/regal@latest
RUN go install github.com/docker/docker-language-server/cmd/docker-language-server@latest
RUN go install github.com/wader/jq-lsp@latest

# Rust tools
RUN /root/.cargo/bin/rustup component add rust-analyzer && \
    /root/.cargo/bin/cargo install taplo-cli

# Terraform LS - platform-aware with "v" prefix stripped
RUN ARCH=$(dpkg --print-architecture) && \
    if [ "$ARCH" = "amd64" ]; then PLATFORM="linux_amd64"; \
    elif [ "$ARCH" = "arm64" ]; then PLATFORM="linux_arm64"; \
    else echo "Unsupported architecture: $ARCH" && exit 1; fi && \
    TERRAFORM_LS_VERSION=$(curl -s https://api.github.com/repos/hashicorp/terraform-ls/releases/latest | grep tag_name | cut -d '"' -f 4 | sed 's/^v//') && \
    echo "Detected platform: $PLATFORM, Downloading terraform-ls version: ${TERRAFORM_LS_VERSION}" && \
    wget https://releases.hashicorp.com/terraform-ls/${TERRAFORM_LS_VERSION}/terraform-ls_${TERRAFORM_LS_VERSION}_${PLATFORM}.zip && \
    unzip terraform-ls_${TERRAFORM_LS_VERSION}_${PLATFORM}.zip -d /tmp/terraform-ls && \
    chmod +x /tmp/terraform-ls/terraform-ls

# Lua LS - platform-aware
RUN ARCH=$(dpkg --print-architecture) && \
    if [ "$ARCH" = "amd64" ]; then SUFFIX="linux-x64"; \
    elif [ "$ARCH" = "arm64" ]; then SUFFIX="linux-arm64"; \
    else echo "Unsupported architecture: $ARCH" && exit 1; fi && \
    LUA_LS_VERSION=$(curl -s https://api.github.com/repos/LuaLS/lua-language-server/releases/latest | grep tag_name | cut -d '"' -f 4 | sed 's/v//') && \
    echo "Detected platform: $SUFFIX, Downloading lua-language-server version: ${LUA_LS_VERSION}" && \
    wget https://github.com/LuaLS/lua-language-server/releases/download/${LUA_LS_VERSION}/lua-language-server-${LUA_LS_VERSION}-${SUFFIX}.tar.gz && \
    mkdir /tmp/luals && \ 
    tar -xzf lua-language-server-*.tar.gz -C /tmp/luals  && \
    chmod +x /tmp/luals/bin/lua-language-server

# Marksman - platform-aware
RUN ARCH=$(dpkg --print-architecture) && \
    if [ "$ARCH" = "amd64" ]; then SUFFIX="linux-x64"; \
    elif [ "$ARCH" = "arm64" ]; then SUFFIX="linux-arm64"; \
    else echo "Unsupported architecture: $ARCH" && exit 1; fi && \
    MARKSMAN_VERSION=$(curl -s https://api.github.com/repos/artempyanykh/marksman/releases/latest | jq -r .tag_name) && \
    echo "Detected platform: $SUFFIX, Downloading marksman version: ${MARKSMAN_VERSION}" && \
    wget https://github.com/artempyanykh/marksman/releases/download/${MARKSMAN_VERSION}/marksman-${SUFFIX} -O /tmp/marksman && \
    chmod +x /tmp/marksman


# # Ruff - standalone binary from releases (faster, no Python deps in final)
# RUN ARCH=$(dpkg --print-architecture) && \
#     if [ "$ARCH" = "amd64" ]; then SUFFIX="x86_64-unknown-linux-gnu"; \
#     elif [ "$ARCH" = "arm64" ]; then SUFFIX="aarch64-unknown-linux-gnu"; \
#     else echo "Unsupported architecture: $ARCH" && exit 1; fi && \
#     RUFF_VERSION=$(curl -s "https://api.github.com/repos/astral-sh/ruff/releases/latest" |  grep tag_name | cut -d '"' -f 4) && \
#     echo "Detected platform: $SUFFIX, Downloading ruff version: ${RUFF_VERSION}" && \
#     wget -qO /tmp/ruff.tar.gz "https://github.com/astral-sh/ruff/releases/latest/download/ruff-${RUFF_VERSION}-${SUFFIX}.tar.gz" && \
#     true

    # tar xf /tmp/ruff.tar.gz -C /tmp/ ruff && \
    # chmod +x /tmp/ruff && \
    # rm /tmp/ruff.tar.gz


# Build yazi filemanagaer
RUN mkdir /tmp/yazi \
    && cd /tmp/yazi \ 
    && git clone https://github.com/sxyazi/yazi.git . \
    && cargo build --release --locked

# Install TeXLab LaTeX language server
RUN cargo install --git https://github.com/latex-lsp/texlab --locked --tag "v5.24.0"

# Install Zellij terminal workspace
RUN cargo install --locked zellij

# Install OpenCode AI CLI
RUN curl -fsSL https://opencode.ai/install | bash

# RUN    apt-get update && apt-get install -y  file && \
#     echo "File installed : $(file --version)" 


# build-essential autoconf automake libtool zlib1g-dev libbz2-dev \
#     && mkdir /tmp/file-tool \
#     && cd /tmp/file-tool \
#     && git clone https://github.com/file/file.git . \
#     && ./autogen.sh \
#     && ./configure --prefix=/tmp/file-tool/build \
#     && make -j$(nproc) \
#     && make install

# Formatters
RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/hougesen/kdlfmt/releases/latest/download/kdlfmt-installer.sh | sh
RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/astral-sh/ruff/releases/download/0.14.10/ruff-installer.sh | sh

# CMD ["cp", "zig-out/bin/fancy-cat", "/output/"]



FROM ubuntu:26.04

ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=Europe/Berlin
ENV LANG=en_US.UTF-8
ENV LC_ALL=en_US.UTF-8

RUN    apt-get update \ 
    && apt-get install -y --no-install-recommends \
       curl \
       git \
       bash \
       lazygit \
       fontconfig \  
       ffmpeg \
       7zip \
       jq \
       poppler-utils \ 
       fd-find \
       ripgrep \
       fzf \
       zoxide \
       imagemagick \  
       ca-certificates \
       locales \
       tzdata \
       nodejs \
       python3 \
       npm \
       clangd \
       lldb \
       file
 
RUN    rm -rf /var/lib/apt/lists/* \
    && apt-get clean

RUN    sed -i '/en_US.UTF-8/s/^# //g' /etc/locale.gen \
    && locale-gen en_US.UTF-8 \
    && update-locale LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8 \
    && rm -rf /var/lib/apt/lists/*


# Tooling from builder stage
COPY --from=builder /root/.cargo/bin/zellij /usr/bin/zellij
COPY --from=builder /tmp/yazi/target/release/yazi /usr/local/bin/yazi
COPY --from=builder /tmp/yazi/target/release/ya /usr/local/bin/ya
COPY --from=builder /root/go/bin/jq-lsp /usr/local/bin

COPY --from=builder /tmp/helix/runtime/ /usr/share/helix/runtime/
COPY --from=builder /tmp/helix/target/opt/hx /usr/bin/hx
COPY --from=builder /root/.opencode/bin/* /usr/bin/

# Language servers
COPY --from=builder /root/.cargo/bin/texlab /usr/local/bin/texlab



# Copy ONLY the final binaries/executables from builder
COPY --from=builder /tmp/terraform-ls/* /usr/local/bin/
COPY --from=builder /tmp/luals/bin /usr/local/bin/
COPY --from=builder /tmp/marksman /usr/local/bin/marksman
COPY --from=builder /root/go/bin/* /usr/local/bin/
COPY --from=builder /root/.cargo/bin/rust-analyzer /usr/local/bin/
COPY --from=builder /root/.cargo/bin/taplo /usr/local/bin/
COPY --from=builder /root/.local/bin/* /usr/local/bin/

# https://github.com/helix-editor/helix/wiki/Language-Server-Configurations
# Node.js LSP servers
RUN apt-get update \ 
    && apt-get install -y --no-install-recommends \
       build-essential \
    && npm install -g \
        "awk-language-server@>=0.5.2" \
        bash-language-server \
        vscode-langservers-extracted \
        dockerfile-language-server-nodejs \
        dot-language-server \
        yaml-language-server \
        @tailwindcss/language-server \
        bibtex-tidy \
        ansible-language-server \
        typescript typescript-language-server \
    && apt remove -y build-essential \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean 


# Node.js LSPs (npm globals - keep minimal node runtime)
# COPY --from=builder /usr/lib/node_modules /usr/lib/node_modules
# COPY --from=builder /usr/bin/bash-language-server /usr/bin/
# COPY --from=builder /usr/bin/vscode-css-language-server /usr/bin/
# COPY --from=builder /usr/bin/vscode-html-language-server /usr/bin/
# COPY --from=builder /usr/bin/vscode-json-language-server /usr/bin/
# COPY --from=builder /usr/bin/yaml-language-server /usr/bin/
# COPY --from=builder /usr/bin/bibtex-tidy /usr/bin/
# COPY --from=builder /usr/bin/docker-file /usr/bin/
# COPY --from=builder /usr/bin/ansible-language-server /usr/bin/



RUN git config --global --add safe.directory /workspace
WORKDIR /workspace

CMD ["zellij"]


# X awk-language-server
# bash-language-server
# bibtex-tidy
# clangd              
# lldb-dap
# X vscode-css-language-server
# docker-langserver
# X dot-language-server
# gopls
# dlv (go debugger)
# terraform-ls
# vscode-html-language-server
# X typescript-language-server
# X jq-lsp
# X vscode-json-language-server
# lua-language-server
# marksman
# x perlnavigator
# ruff
# X regols
# X ruby-lsp
# rust-analyzer
# taplo
# yaml-language-server
# ansible-language-server