#!/usr/bin/env -S /bin/sh -c '"$(dirname "$0")/venv/bin/python" "$0" "$@"'

from __future__ import annotations

import base64
import dataclasses
import glob
import sys
import threading
import time
import asyncio
from itertools import repeat, cycle
from typing import Iterator

import alive_progress
import cv2

import httpx


def main():
    args = parse_args()
    video_path: str = args.file
    server: str = args.peer
    host = server.rsplit(":", 1)[0]
    port = server.rsplit(":", 1)[1] if len(server.rsplit(":", 1)) == 2 else "8081"
    is_loop = args.loop

    stream_videos_to_server(glob.iglob(video_path, recursive=True), host, port, is_loop=is_loop)


def stream_videos_to_server(video_files: Iterator[str], host: str, port: str, is_loop = False):
    display_width, display_height, display_fps = retrieve_display_stats(f"{host}:{port}")
    videos = cycle(video_files) if is_loop else video_files
    video = load_video(next(videos), display_fps)

    unix_start = time.time_ns() // 1000
    offset_i = 0
    video_i = lambda: i + offset_i
    for i, _ in enumerate(repeat(None)):
        frame = video.next_frame()
        if frame is None:
            unix_start += video_i() * 1_000_000 / video.fps
            offset_i -= video_i()
            video.close()
            video = None
            while video is None:
                video_file = next(videos, None)
                if video_file:
                    print()
                    print("Loading new video sequence...")
                    video = load_video(video_file, display_fps)
                else:
                    print("Finished sending frames")
                    break  # Exit loop if no more frames
            frame = video.next_frame()

        if i % video.frame_divisor != 0:
            continue

        frame_time = unix_start + video_i() * 1_000_000 / video.fps
        time.sleep(max(0.0, (frame_time - time.time_ns() // 1000) / 1_000_000.0))
        send_video_frame(
            int(frame_time),
            (display_width, display_height),
            (video.width, video.height),
            frame,
            f"{host}:{port}"
        )


def retrieve_display_stats(peer: str) -> [int, int, float]:
    resp = httpx.get(f"http://{peer}/meta")
    payload = resp.json()
    return payload["display"]["width"], payload["display"]["height"], payload["display"]["fps"]

def send_video_frame(
    unix_micros: int,
    display_dims: (int, int),
    video_dims: (int, int),
    frame: cv2.Mat,
    peer: str
) -> None:
    scale = min(float(display_dims[0]) / video_dims[0], float(display_dims[1]) / video_dims[1])

    # Optionally resize the frame
    resized_frame = cv2.resize(frame, (0, 0), fx=scale, fy=scale, interpolation=cv2.INTER_AREA)
    rgb_image = cv2.cvtColor(resized_frame, cv2.COLOR_BGR2RGB)
    byte_list = rgb_image.flatten().tolist()

    pixel_data = bytes(byte_list)

    send_frame(unix_micros + 1_000_000, resized_frame.shape[1], resized_frame.shape[0], pixel_data, peer)

http_client = httpx.AsyncClient()
event_loop = asyncio.new_event_loop()
# Run event loop in separate thread
def run_event_loop():
    asyncio.set_event_loop(event_loop)
    event_loop.run_forever()

threading.Thread(
    target=run_event_loop, daemon=True, name="HTTP event loop"
).start()
def send_frame(unix_micros: int, width: int, height: int, frame: bytes, peer: str) -> None:
    encoded_pixels = base64.encodebytes(frame).decode('utf-8').replace('\n', '')

    asyncio.run_coroutine_threadsafe(
        http_client.post(f"http://{peer}/frame",
            json={
                "unix_micros": int(unix_micros),
                "frame": {
                    "width": width,
                    "height": height,
                    "pixels_b64": encoded_pixels
                }
            }
        ),
        loop=event_loop
    )


def retrieve_video_stats(video: cv2.VideoCapture) -> [int, int, float]:
    video_fps = video.get(cv2.CAP_PROP_FPS)
    video_width = int(video.get(cv2.CAP_PROP_FRAME_WIDTH))
    video_height = int(video.get(cv2.CAP_PROP_FRAME_HEIGHT))

    return video_width, video_height, video_fps

def print_video_metadata(path: str, video: cv2.VideoCapture):
    # Retrieve video metadata
    fps = video.get(cv2.CAP_PROP_FPS)
    width = int(video.get(cv2.CAP_PROP_FRAME_WIDTH))
    height = int(video.get(cv2.CAP_PROP_FRAME_HEIGHT))
    frame_count = int(video.get(cv2.CAP_PROP_FRAME_COUNT))
    seconds_duration = frame_count / fps

    hours, remainder = divmod(seconds_duration, 3600)
    minutes, seconds = divmod(remainder, 60)

    print(f"Video Metadata:")
    print(f"  File: {path}")
    print(f"  FPS: {fps}")
    print(f"  Resolution: {width}x{height}")
    print(f"  Duration: {int(hours)}:{int(minutes):02d}:{seconds:05.2f}")


def open_video(path: str, print_metadata: bool = False) -> cv2.VideoCapture | None:
    video = cv2.VideoCapture(path)

    if not video.isOpened():
        print(f"Error: Cannot open video {path}")
        return None

    if print_metadata:
        print_video_metadata(path, video)

    return video

def load_video(path: str, display_fps: float) -> VideoContext | None:
    capture = open_video(path, print_metadata=True)

    if not capture:
        return None

    return VideoContext(capture, display_fps)


def parse_args():
    import argparse

    parser = argparse.ArgumentParser(
        prog="send_video.py",
        description="Send video frames to a running RasGB-Pi server",
        exit_on_error=True
    )
    parser.add_argument("file", type=str, help="Path or glob to the video file")
    parser.add_argument("-s", "--peer", type=str, required=True, help="RasGB-Pi server address <host>:<port>")
    parser.add_argument("--loop", action="store_true", help="Loop the sequence")

    args = parser.parse_args()
    return args


class VideoContext:
    def __init__(self, capture: cv2.VideoCapture, display_fps: float):
        self.capture = capture
        self.width, self.height, self.fps = retrieve_video_stats(capture)
        self.frame_divisor = max(int(self.fps // display_fps), 1)
        self._progress = alive_progress.alive_bar(
            total=int(self.capture.get(cv2.CAP_PROP_FRAME_COUNT)),
            spinner=None,
            unit=" Frames",
            stats="<-> {eta} ({rate})"
        )
        self._progress_context = self._progress.__enter__()

    def close(self):
        self.capture.release()
        self._progress.__exit__(None, None, None)

    def next_frame(self):
        ret, frame = self.capture.read()
        if not ret:
            return None

        self._progress_context()
        return frame


if __name__ == '__main__':
    main()
