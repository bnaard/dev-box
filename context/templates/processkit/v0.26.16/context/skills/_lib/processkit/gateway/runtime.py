"""Runtime metadata and defaults for processkit MCP gateway entrypoints."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Mapping


DEFAULT_STDIO_TRANSPORT = "stdio"
DEFAULT_DAEMON_TRANSPORT = "streamable-http"
DEFAULT_DAEMON_HOST = "127.0.0.1"
DEFAULT_DAEMON_PORT = 8000
DEFAULT_DAEMON_PATH = "/mcp"

ENV_TRANSPORT = "PROCESSKIT_GATEWAY_TRANSPORT"
ENV_DAEMON_HOST = "PROCESSKIT_GATEWAY_HOST"
ENV_DAEMON_PORT = "PROCESSKIT_GATEWAY_PORT"
ENV_DAEMON_PATH = "PROCESSKIT_GATEWAY_PATH"
ENV_DAEMON_LAZY = "PROCESSKIT_GATEWAY_LAZY"
ENV_DAEMON_PROXY = "PROCESSKIT_GATEWAY_PROXY"

DAEMON_ENV_DEFAULTS: dict[str, str] = {
    ENV_TRANSPORT: DEFAULT_DAEMON_TRANSPORT,
    ENV_DAEMON_HOST: DEFAULT_DAEMON_HOST,
    ENV_DAEMON_PORT: str(DEFAULT_DAEMON_PORT),
    ENV_DAEMON_PATH: DEFAULT_DAEMON_PATH,
}


def _truthy(value: str | None) -> bool:
    if value is None:
        return False
    return value.strip().lower() in {"1", "true", "yes", "on"}


def normalize_daemon_path(path: str | None) -> str:
    """Return a stable streamable-http mount path."""
    if path is None or not path.strip():
        return DEFAULT_DAEMON_PATH
    value = path.strip()
    if not value.startswith("/"):
        value = f"/{value}"
    return value.rstrip("/") or "/"


def daemon_port_from_env(value: str | None) -> int:
    """Parse and validate the configured daemon port."""
    if value is None or not value.strip():
        return DEFAULT_DAEMON_PORT
    try:
        port = int(value)
    except ValueError as exc:
        raise ValueError(f"{ENV_DAEMON_PORT} must be an integer") from exc
    if not 1 <= port <= 65535:
        raise ValueError(f"{ENV_DAEMON_PORT} must be between 1 and 65535")
    return port


@dataclass(frozen=True)
class DaemonConfig:
    """Streamable-http daemon binding and environment configuration."""

    host: str = DEFAULT_DAEMON_HOST
    port: int = DEFAULT_DAEMON_PORT
    path: str = DEFAULT_DAEMON_PATH
    transport: str = DEFAULT_DAEMON_TRANSPORT
    env: dict[str, str] = field(default_factory=dict)

    def __post_init__(self) -> None:
        object.__setattr__(self, "path", normalize_daemon_path(self.path))

    @property
    def url(self) -> str:
        return f"http://{self.host}:{self.port}{self.path}"

    def as_env(self) -> dict[str, str]:
        values = {
            ENV_TRANSPORT: self.transport,
            ENV_DAEMON_HOST: self.host,
            ENV_DAEMON_PORT: str(self.port),
            ENV_DAEMON_PATH: self.path,
        }
        values.update(self.env)
        return values

    def as_dict(self) -> dict[str, Any]:
        return {
            "transport": self.transport,
            "host": self.host,
            "port": self.port,
            "path": self.path,
            "url": self.url,
            "env": self.as_env(),
        }


def daemon_config_from_env(
    env: Mapping[str, str] | None = None,
) -> DaemonConfig:
    """Build daemon config from environment-like values."""
    values = env or {}
    host = values.get(ENV_DAEMON_HOST, DEFAULT_DAEMON_HOST).strip()
    return DaemonConfig(
        host=host or DEFAULT_DAEMON_HOST,
        port=daemon_port_from_env(values.get(ENV_DAEMON_PORT)),
        path=normalize_daemon_path(values.get(ENV_DAEMON_PATH)),
        transport=values.get(ENV_TRANSPORT, DEFAULT_DAEMON_TRANSPORT).strip()
        or DEFAULT_DAEMON_TRANSPORT,
        env={
            key: value
            for key, value in values.items()
            if key in {ENV_DAEMON_LAZY, ENV_DAEMON_PROXY}
        },
    )


def runtime_metadata_from_env(
    env: Mapping[str, str] | None = None,
) -> RuntimeMetadata:
    """Build runtime metadata from environment-like values."""
    values = env or {}
    transport = values.get(ENV_TRANSPORT, DEFAULT_STDIO_TRANSPORT).strip()
    transport = transport or DEFAULT_STDIO_TRANSPORT
    lazy = _truthy(values.get(ENV_DAEMON_LAZY))
    proxy = _truthy(values.get(ENV_DAEMON_PROXY))

    if transport == DEFAULT_DAEMON_TRANSPORT:
        return RuntimeMetadata.for_daemon(
            daemon_config_from_env(values),
            lazy=lazy,
        )
    if proxy:
        return RuntimeMetadata.for_proxy(
            daemon_config_from_env(values),
            lazy=lazy,
        )
    return RuntimeMetadata(
        transport=transport,
        lazy=lazy,
        proxy=proxy,
    )


@dataclass(frozen=True)
class RuntimeMetadata:
    """Describe the current gateway runtime mode."""

    import_mode: str = "eager"
    lazy: bool = False
    lazy_daemon: bool | None = None
    transport: str = DEFAULT_STDIO_TRANSPORT
    daemon: bool = False
    proxy: bool = False
    host: str | None = None
    port: int | None = None
    path: str | None = None

    def __post_init__(self) -> None:
        lazy_daemon = (
            self.lazy
            if self.lazy_daemon is None
            else self.lazy_daemon
        )
        lazy = self.lazy or lazy_daemon
        object.__setattr__(self, "lazy", lazy)
        object.__setattr__(self, "lazy_daemon", lazy_daemon)
        if self.path is not None:
            object.__setattr__(self, "path", normalize_daemon_path(self.path))

    @classmethod
    def for_stdio(cls, *, lazy: bool = False) -> "RuntimeMetadata":
        return cls(transport=DEFAULT_STDIO_TRANSPORT, lazy=lazy)

    @classmethod
    def for_daemon(
        cls,
        config: DaemonConfig | None = None,
        *,
        lazy: bool = False,
    ) -> "RuntimeMetadata":
        daemon_config = config or DaemonConfig()
        return cls(
            transport=daemon_config.transport,
            daemon=True,
            lazy=lazy,
            host=daemon_config.host,
            port=daemon_config.port,
            path=daemon_config.path,
        )

    @classmethod
    def for_proxy(
        cls,
        config: DaemonConfig | None = None,
        *,
        lazy: bool = False,
    ) -> "RuntimeMetadata":
        daemon_config = config or DaemonConfig()
        return cls(
            transport=DEFAULT_STDIO_TRANSPORT,
            daemon=False,
            proxy=True,
            lazy=lazy,
            host=daemon_config.host,
            port=daemon_config.port,
            path=daemon_config.path,
        )

    def as_dict(self) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "import_mode": self.import_mode,
            "lazy": self.lazy,
            "lazy_daemon": self.lazy_daemon,
            "transport": self.transport,
            "daemon": self.daemon,
            "proxy": self.proxy,
        }
        if self.host is not None:
            payload["host"] = self.host
        if self.port is not None:
            payload["port"] = self.port
        if self.path is not None:
            payload["path"] = self.path
        return payload
