# syntax=docker/dockerfile:1
FROM ghcr.io/projectious-work/dev-box:base-latest AS runtime
# For local builds, use: FROM dev-box-base:latest

ENV DEBIAN_FRONTEND=noninteractive

# Python 3.13 + pip + venv
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 \
    python3-pip \
    python3-venv \
    && rm -rf /var/lib/apt/lists/*

# uv — fast Python package manager (pinned version)
ARG UV_VERSION=0.7.12
COPY --from=ghcr.io/astral-sh/uv:0.7.12 /uv /usr/local/bin/uv
COPY --from=ghcr.io/astral-sh/uv:0.7.12 /uvx /usr/local/bin/uvx

ENV PATH="/root/.local/bin:${PATH}"

WORKDIR /workspace
CMD ["sleep", "infinity"]
