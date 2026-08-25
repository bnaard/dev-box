#!/usr/bin/env python3
"""Render selected paths as an ls-like table with tree and disk-usage totals."""

from __future__ import annotations

import argparse
import datetime as dt
import grp
import os
from pathlib import Path
import pwd
import stat
import sys


def human_size(value: int) -> str:
    units = ("B", "K", "M", "G", "T", "P")
    amount = float(value)
    for unit in units:
        if abs(amount) < 1024 or unit == units[-1]:
            return f"{amount:.0f}{unit}" if unit == "B" or amount >= 10 else f"{amount:.1f}{unit}"
        amount /= 1024
    raise AssertionError("unreachable")


def name_for(identifier: int, lookup: object) -> str:
    try:
        return lookup(identifier).pw_name if lookup is pwd.getpwuid else lookup(identifier).gr_name
    except KeyError:
        return str(identifier)


class Row:
    def __init__(self, tree: str, depth: int, info: os.stat_result, size: int) -> None:
        self.tree, self.depth = tree, depth
        self.mode, self.links = stat.filemode(info.st_mode), str(info.st_nlink)
        self.owner, self.group = name_for(info.st_uid, pwd.getpwuid), name_for(info.st_gid, grp.getgrgid)
        self.modified = dt.datetime.fromtimestamp(info.st_mtime).strftime("%Y-%m-%d %H:%M")
        self.size = human_size(size)


def collect(path: Path, prefix: str = "", depth: int = 0, branch: str = "") -> tuple[int, list[Row]]:
    try:
        info = path.lstat()
    except OSError as error:
        print(f"aibox-size-tree: {path}: {error}", file=sys.stderr)
        return 0, []
    label = branch + path.name + ("/" if stat.S_ISDIR(info.st_mode) else "")
    own = info.st_blocks * 512
    if not stat.S_ISDIR(info.st_mode) or path.is_symlink():
        return own, [Row(prefix + label, depth, info, own)]
    try:
        children = sorted(path.iterdir(), key=lambda item: (not item.is_dir(), item.name.casefold()))
    except OSError as error:
        print(f"aibox-size-tree: {path}: {error}", file=sys.stderr)
        children = []
    total, nested = own, []
    descendant_prefix = prefix
    if branch:
        descendant_prefix += "    " if branch == "└── " else "│   "
    for index, child in enumerate(children):
        last = index == len(children) - 1
        child_total, rows = collect(child, descendant_prefix, depth + 1,
                                    "└── " if last else "├── ")
        total += child_total
        nested.extend(rows)
    return total, [Row(prefix + label, depth, info, total), *nested]


def render(paths: list[Path]) -> None:
    groups = [collect(path)[1] for path in paths]
    actual = [row for group_rows in groups for row in group_rows]
    widths = {
        key: max(len(title), *(len(getattr(row, key)) for row in actual))
        for key, title in (("tree", "TREE"), ("mode", "MODE"), ("links", "LINKS"),
                           ("owner", "OWNER"), ("group", "GROUP"))
    }
    print(f"{'TREE':<{widths['tree']}}  {'MODE':<{widths['mode']}}  {'LINKS':>{widths['links']}}  "
          f"{'OWNER':<{widths['owner']}}  {'GROUP':<{widths['group']}}  {'MODIFIED':<16}  SIZE")
    for group_index, group_rows in enumerate(groups):
        if group_index:
            print()
        for row in group_rows:
            size = "  " * row.depth + row.size
            print(f"{row.tree:<{widths['tree']}}  {row.mode:<{widths['mode']}}  {row.links:>{widths['links']}}  "
                  f"{row.owner:<{widths['owner']}}  {row.group:<{widths['group']}}  {row.modified:<16}  {size}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("paths", nargs="+")
    args = parser.parse_args()
    render([Path(value) for value in args.paths])


if __name__ == "__main__":
    main()
