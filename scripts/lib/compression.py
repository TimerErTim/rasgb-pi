import abc
import re
from dataclasses import dataclass

import argparse
import zstd


class Compression(abc.ABC):
    @abc.abstractmethod
    def compress(self, data: str) -> bytes:
        raise NotImplementedError()

    @abc.abstractmethod
    def http_headers(self) -> dict:
        raise NotImplementedError()


@dataclass(frozen=True)
class ZstdCompression(Compression):
    level: int

    def compress(self, data: str) -> bytes:
        return zstd.compress(data.encode("utf-8"), self.level)

    def http_headers(self) -> dict:
        return {"Content-Encoding": "zstd"}

@dataclass(frozen=True)
class NoCompression(Compression):
    def compress(self, data: str) -> bytes:
        return data.encode()

    def http_headers(self) -> dict:
        return {}

def parse_compression(value):
    """Parse compression argument of format 'algorithm:level' or 'none'."""
    if value.lower() == "none":
        return NoCompression()

    match = re.fullmatch(r"([a-zA-Z0-9]+):(\d+)", value)
    if match is None:
        raise argparse.ArgumentTypeError(f"Invalid compression format: {value}")

    algo, level = match.groups()
    level = int(level)

    if algo == "zstd":
        return ZstdCompression(level)

    raise argparse.ArgumentTypeError(f"Invalid compression format: {value}")
