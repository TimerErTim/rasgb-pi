from __future__ import annotations

import asyncio
import base64
import concurrent.futures
import json
import os
import threading
import time
from dataclasses import dataclass
from sys import stderr
from typing import Optional, Callable

import cv2
import httpx

from lib.compression import Compression, ZstdCompression
from lib.resize import ResizeMethod, ResizeArea


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
            self.event_loop.set_exception_handler(lambda loop, context: print(context))
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
            self._encode_payload,
            make_payload(metadata)
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
        resize_method: Optional[ResizeMethod] = ResizeArea(),
        map_frame: Callable[[cv2.Mat, DisplayMetadata], cv2.Mat] = None
    ) -> None:
        def convert(metadata: DisplayMetadata) -> FramePayload:
            if resize_method is not None:
                scale = min(float(metadata.width) / frame.shape[1], float(metadata.height) / frame.shape[0])
                resized_frame = cv2.resize(frame, (0, 0), fx=scale, fy=scale,
                                           interpolation=resize_method.get_cv2_method())
                resized_frame = resized_frame
            else:
                resized_frame = frame

            if map_frame is not None:
                mapped_frame = map_frame(resized_frame, metadata)
            else:
                mapped_frame = resized_frame

            byte_list = mapped_frame.flatten().tolist()
            return FramePayload(mapped_frame.shape[1], mapped_frame.shape[0], bytes(byte_list))

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
            current_fps = 1.0
            def capture_frame(metadata: DisplayMetadata) -> FramePayload | None:
                nonlocal current_fps
                current_fps = min(metadata.fps, max_fps or metadata.fps)
                return make_frame(metadata)

            while not stopped:
                start_location.unix_micros += int(1_000_000 // current_fps)
                try:
                    await self._send(start_location, capture_frame)
                except Exception as e:
                    print("Error sending frame:", e, file=stderr)
                waiting_time = (start_location.unix_micros - time.time_ns() // 1000) / 1_000_000.0
                await asyncio.sleep(max(0.0, waiting_time))

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
