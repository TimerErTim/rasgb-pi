#!/usr/bin/env -S /bin/sh -c '"$(dirname "$0")/venv/bin/python" "$0" "$@"'

from __future__ import annotations

import glob
import time
from itertools import repeat, cycle
from typing import Iterator

import alive_progress
import cv2

import httpx
import zstd

from lib.client import RasgbPiClient, FrameLocation, DisplayMetadata
from lib.compression import NoCompression, ZstdCompression


def main():
    args = parse_args()
    video_path: str = args.file
    server: str = args.peer
    host = server.rsplit(":", 1)[0]
    port = server.rsplit(":", 1)[1] if len(server.rsplit(":", 1)) == 2 else "8081"
    is_loop = args.loop
    channel = args.channel

    client = RasgbPiClient(f"http://{host}:{port}", time_buffer_ms=1000, default_compression=ZstdCompression(level=3), default_channel=channel)

    stream_videos_to_server(glob.iglob(video_path, recursive=True), client, is_loop=is_loop)


def stream_videos_to_server(video_files: Iterator[str], client: RasgbPiClient, is_loop = False):
    videos = cycle(video_files) if is_loop else video_files
    video = load_video(next(videos))

    unix_start = time.time_ns() // 1000
    offset_i = 0
    display_fps = video.fps
    video_i = lambda: i + offset_i
    video_frame_divisor = lambda: max(int(display_fps // video.fps), 1)
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
                    video = load_video(video_file)
                else:
                    print("Finished sending frames")
                    break  # Exit loop if no more frames
            frame = video.next_frame()

        if i % video_frame_divisor() != 0:
            continue

        frame = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
        frame_time = unix_start + video_i() * 1_000_000 / video.fps
        time.sleep(max(0.0, (frame_time - time.time_ns() // 1000) / 1_000_000.0))
        def update_fps(mat, metadata: DisplayMetadata):
            nonlocal display_fps
            display_fps = metadata.fps
            return mat
        client.send_mat(FrameLocation(unix_micros=int(frame_time)), frame, map_frame=update_fps)


def retrieve_display_stats(peer: str) -> [int, int, float]:
    resp = httpx.get(f"http://{peer}/meta")
    payload = resp.json()
    return payload["display"]["width"], payload["display"]["height"], payload["display"]["fps"]


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

def load_video(path: str) -> VideoContext | None:
    capture = open_video(path, print_metadata=True)

    if not capture:
        return None

    return VideoContext(capture)


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
    parser.add_argument("-c", "--channel", type=int, help="Channel to send the video to")

    args = parser.parse_args()
    return args


class VideoContext:
    def __init__(self, capture: cv2.VideoCapture):
        self.capture = capture
        self.width, self.height, self.fps = retrieve_video_stats(capture)
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
