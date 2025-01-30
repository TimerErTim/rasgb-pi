from __future__ import annotations

import asyncio
import base64
import concurrent.futures
import json
import os
import threading
from dataclasses import dataclass
from typing import Optional, Callable

import cv2
import httpx

from lib.compression import Compression, ZstdCompression


class RasgbPiClient:
    def __init__(
            self,
            url: str,
            time_buffer_ms: int = 0,
            default_channel: Optional[int] = None,
            default_compression: Compression = ZstdCompression(level=3)
    ) -> None:
        self.url = url
        self.time_buffer_ms = time_buffer_ms
        self.default_channel = default_channel
        self.default_compression = default_compression

        self.http_client = httpx.AsyncClient()
        self.event_loop = asyncio.new_event_loop()
        self.blocking_executor = concurrent.futures.ThreadPoolExecutor(
            max_workers=max(2, len(os.sched_getaffinity(0)) - 2))

        # Run event loop in separate thread
        def run_event_loop():
            asyncio.set_event_loop(self.event_loop)
            self.event_loop.run_forever()

        threading.Thread(
            target=run_event_loop, daemon=True, name="HTTP event loop"
        ).start()

    async def _send(self, location: FrameLocation,
                    make_payload: Callable[[DisplayMetadata], FramePayload | None]) -> None:
        unix_micros = location.unix_micros + self.time_buffer_ms * 1_000
        channel = location.channel or self.default_channel
        path = f"{self.url}/frame/{unix_micros}/channel/{channel}" \
            if channel is not None else \
            f"{self.url}/frame/{unix_micros}"
        resp = await self.http_client.head(path)
        if resp.status_code >= 300:
            return

        metadata = DisplayMetadata(
            width=int(resp.headers["Display-Width"]),
            height=int(resp.headers["Display-Height"]),
            fps=float(resp.headers["Display-FPS"])
        )
        body = await self.event_loop.run_in_executor(
            self.blocking_executor,
            lambda: self._encode_payload(make_payload(metadata))
        )
        if body is None:
            return

        await self.http_client.post(
            path, content=body,
            headers={
                "Content-Type": "application/json"
            } | self.default_compression.http_headers()
        )

    def _encode_payload(self, payload: FramePayload | None) -> bytes | None:
        if payload is None:
            return None

        assert len(payload.pixels_rgb) == payload.width * payload.height * 3
        encoded_pixels = base64.encodebytes(payload.pixels_rgb).decode('utf-8').replace('\n', '')
        data = {
            "frame": {
                "width": payload.width,
                "height": payload.height,
                "pixels_b64": encoded_pixels
            }
        }
        body = self.default_compression.compress(json.dumps(data))
        return body

    def send_raw(self, location: FrameLocation, frame: bytes, width: int, height: int) -> None:
        request = self._send(location, lambda metadata: FramePayload(width, height, frame))
        asyncio.run_coroutine_threadsafe(
            request,
            loop=self.event_loop
        )

    def send_mat(
        self,
        location: FrameLocation,
        frame: cv2.Mat,
        resize: bool = True,
        callback: Callable[[DisplayMetadata], None] = None
    ) -> None:
        def convert(metadata: DisplayMetadata) -> FramePayload:
            if callback is not None:
                callback(metadata)
            if resize:
                scale = min(float(metadata.width) / frame.shape[1], float(metadata.height) / frame.shape[0])
                resized_frame = cv2.resize(frame, (0, 0), fx=scale, fy=scale, interpolation=cv2.INTER_LANCZOS4)
                shipped_frame = resized_frame
            else:
                shipped_frame = frame
            byte_list = shipped_frame.flatten().tolist()
            return FramePayload(shipped_frame.shape[1], shipped_frame.shape[0], bytes(byte_list))

        request = self._send(location, convert)
        asyncio.run_coroutine_threadsafe(
            request,
            loop=self.event_loop
        )

    def send_generator(
        self,
        start_location: FrameLocation,
        make_frame: Callable[[DisplayMetadata], FramePayload | None],
        max_fps: float = None,
    ) -> Callable[[], None]:
        stopped = False
        async def run_loop():
            nonlocal stopped
            next_unix_micros = start_location.unix_micros
            def capture_frame(metadata: DisplayMetadata) -> FramePayload | None:
                nonlocal next_unix_micros
                next_unix_micros += 1_000_000 // min(metadata.fps, max_fps or metadata.fps)
                return make_frame(metadata)

            while not stopped:
                start_location.unix_micros = next_unix_micros
                await self._send(start_location, capture_frame)

        asyncio.run_coroutine_threadsafe(
            run_loop(),
            loop=self.event_loop
        )

        def stop():
            nonlocal stopped
            stopped = True
        return stop


@dataclass
class FrameLocation:
    unix_micros: int
    channel: Optional[int] = None


@dataclass(frozen=True)
class FramePayload:
    width: int
    height: int
    pixels_rgb: bytes


@dataclass
class DisplayMetadata:
    width: int
    height: int
    fps: float
