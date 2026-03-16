# =============================================================================
# Stage 1: TeX Live builder — install from CTAN with basic scheme + packages
# =============================================================================
FROM debian:trixie-slim AS texlive-builder

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    perl \
    wget \
    fontconfig \
    && rm -rf /var/lib/apt/lists/*

# Install TeX Live from CTAN (basic scheme, then add packages)
RUN mkdir -p /tmp/texlive-installer && cd /tmp/texlive-installer \
    && wget -q https://mirror.ctan.org/systems/texlive/tlnet/install-tl-unix.tar.gz \
    && tar -xzf install-tl-unix.tar.gz --strip-components=1 \
    && echo "selected_scheme scheme-basic" > texlive.profile \
    && echo "TEXDIR /usr/local/texlive" >> texlive.profile \
    && echo "TEXMFLOCAL /usr/local/texlive/texmf-local" >> texlive.profile \
    && echo "TEXMFSYSCONFIG /usr/local/texlive/texmf-config" >> texlive.profile \
    && echo "TEXMFSYSVAR /usr/local/texlive/texmf-var" >> texlive.profile \
    && echo "option_doc 0" >> texlive.profile \
    && echo "option_src 0" >> texlive.profile \
    && echo "tlpdbopt_autobackup 0" >> texlive.profile \
    && echo "tlpdbopt_install_docfiles 0" >> texlive.profile \
    && echo "tlpdbopt_install_srcfiles 0" >> texlive.profile \
    && ./install-tl --profile=texlive.profile --no-interaction \
    && rm -rf /tmp/texlive-installer

# Create version-independent symlink
RUN ln -s /usr/local/texlive/*/bin/* /usr/local/texlive/bin

ENV PATH="/usr/local/texlive/bin:${PATH}"

# Install LaTeX packages (comprehensive set for academic and technical documents)
RUN tlmgr install --no-execute-actions \
    # LuaTeX engine and fonts
    luatex luaotfload luatexbase luacolor lua-ul fontspec unicode-math \
    lualatex-math luahbtex \
    # Core packages
    latex latex-bin latexmk collection-latexrecommended \
    # Page layout and geometry
    geometry fancyhdr lastpage multicol multirow adjustbox \
    # Typography
    microtype parskip setspace titlesec titletoc \
    # Tables
    tabularray booktabs longtable tabu array colortbl \
    # Colors and graphics
    xcolor graphicx graphics epstopdf svg \
    # TikZ/PGF
    pgf tikz-cd tikzmark pgfplots pgfplotstable \
    # Boxes
    tcolorbox pdfcol environ etoolbox \
    # Bibliography
    biblatex biber csquotes \
    # Code listings
    minted fvextra fancyvrb upquote lineno catchfile xstring framed float \
    # Math
    amsmath amscls mathtools amsthm \
    # Hyperlinks and references
    hyperref bookmark nameref cleveref url \
    # Fonts
    fontenc lm lm-math sourcecodepro sourcesanspro tex-gyre \
    freetype2 gnu-freefont \
    # Misc utilities
    caption subcaption enumitem listings wrapfig \
    footmisc appendix pdfpages pdflscape xkeyval iftex \
    babel babel-english \
    tools oberdiek amsfonts symbol zapfding \
    kvoptions kvsetkeys kvdefinekeys ltxcmds infwarerr \
    epstopdf-pkg grfext pdftexcmds auxhook intcalc letltxmacro \
    bitset bigintcalc atbegshi atveryend rerunfilecheck \
    uniquecounter refcount gettitlestring hycolor \
    stringenc pdfescape hobsub \
    bytefield siunitx markdown soul ulem \
    todonotes changes datetime2 tracklang \
    was collection-fontsrecommended \
    selnolig emoji \
    && mktexlsr \
    && updmap-sys \
    && fmtutil-sys --all

# =============================================================================
# Stage 2: Runtime — Rust image + TeX Live tree
# =============================================================================
FROM ghcr.io/projectious-work/dev-box:rust-latest AS runtime
# For local builds, use: FROM dev-box-rust:latest

ENV DEBIAN_FRONTEND=noninteractive

# Runtime dependencies for TeX Live
RUN apt-get update && apt-get install -y --no-install-recommends \
    perl \
    fontconfig \
    inkscape \
    poppler-utils \
    # latexindent perl dependencies
    libfile-homedir-perl \
    libyaml-tiny-perl \
    liblog-log4perl-perl \
    libunicode-linebreak-perl \
    && rm -rf /var/lib/apt/lists/*

# Copy TeX Live tree from builder
COPY --from=texlive-builder /usr/local/texlive /usr/local/texlive

ENV PATH="/usr/local/texlive/bin:${PATH}"

WORKDIR /workspace
CMD ["sleep", "infinity"]
