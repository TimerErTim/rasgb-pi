import abc
from dataclasses import dataclass

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