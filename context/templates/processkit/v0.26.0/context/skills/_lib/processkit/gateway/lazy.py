"""Lazy gateway tool registration helpers."""

from __future__ import annotations

from typing import Any

from mcp.server.fastmcp.tools.base import Tool
from mcp.server.fastmcp.utilities.func_metadata import (
    ArgModelBase,
    FuncMetadata,
)
from pydantic import ConfigDict

from .catalog import annotations_from_metadata


class PassthroughArguments(ArgModelBase):
    """Accept arbitrary tool arguments for catalog-backed lazy wrappers."""

    model_config = ConfigDict(arbitrary_types_allowed=True, extra="allow")

    def model_dump_one_level(self) -> dict[str, Any]:
        return dict(self.model_extra or {})


PASSTHROUGH_METADATA = FuncMetadata(arg_model=PassthroughArguments)


def make_lazy_tool(registry: Any, item: dict[str, Any]) -> Tool:
    """Create a FastMCP Tool that defers execution to ``registry``."""
    name = item["name"]

    async def invoke(**arguments: Any) -> Any:
        return await registry.call_tool(name, arguments)

    base = Tool.from_function(
        invoke,
        name=name,
        description=item.get("description") or "",
        annotations=annotations_from_metadata(item.get("annotations")),
    )
    return base.model_copy(update={
        "title": item.get("title"),
        "parameters": item.get("parameters") or {
            "type": "object",
            "properties": {},
        },
        "fn_metadata": PASSTHROUGH_METADATA,
        "is_async": True,
        "annotations": annotations_from_metadata(item.get("annotations")),
    })
