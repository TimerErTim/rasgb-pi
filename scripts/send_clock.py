#!/usr/bin/env -S /bin/sh -c '"$(dirname "$0")/venv/bin/python" "$0" "$@"'

import datetime
import math
import time

import cv2
import numpy as np

from lib.client import RasgbPiClient, FrameLocation, DisplayMetadata, FramePayload
from lib.compression import ZstdCompression


def main():
    args = parse_args()
    server: str = args.peer
    host = server.rsplit(":", 1)[0]
    port = server.rsplit(":", 1)[1] if len(server.rsplit(":", 1)) == 2 else "8081"

    client = RasgbPiClient(f"http://{host}:{port}", time_buffer_ms=1000, default_compression=ZstdCompression(level=15))

    render_clock_to_server(client)

def render_clock_to_server(client: RasgbPiClient):
    stop_sending = client.send_generator(
        FrameLocation(unix_micros=time.time_ns() // 1000),
        make_frame=make_clock_frame,
        max_fps=10
    )
    try:
        while True:
            time.sleep(1000)
    except KeyboardInterrupt:
        print("quitting...")
    finally:
        stop_sending()

def make_clock_frame(display: DisplayMetadata) -> FramePayload:
    # Convert timestamp to datetime
    timestamp = datetime.datetime.fromtimestamp(time.time() - 1)

    hours = timestamp.hour % 12  # Convert to 12-hour format
    minutes = timestamp.minute
    seconds = timestamp.second

    # Create a blank white image
    img = np.zeros((display.height, display.width, 3), dtype=np.uint8)

    # Clock properties
    center = (display.width // 2, display.height // 2)
    radius = int(min(display.width, display.height) // 2.25)  # Adjust radius to fit inside the image
    thickness = 2

    # Draw clock circle
    cv2.circle(img, center, radius, (255, 255, 255), thickness)

    # Draw clock center point
    cv2.circle(img, center, 4, (255, 255, 255), -1)

    # Draw hour and minute ticks
    for i in range(60):  # 60 ticks (every minute)
        angle = math.radians(270 + (i * 6))  # Each tick is 6° apart
        outer_point = (
            int(center[0] + radius * math.cos(angle)),
            int(center[1] + radius * math.sin(angle))
        )
        inner_point = (
            int(center[0] + (radius - (8 if i % 60 == 0 else 15 if i % 5 == 0 else 5)) * math.cos(angle)),  # Longer ticks every 5 mins
            int(center[1] + (radius - (8 if i % 60 == 0 else 15 if i % 5 == 0 else 5)) * math.sin(angle))
        )
        cv2.line(img, outer_point, inner_point, (255, 255, 255), 2 if i % 5 == 0 else 1, lineType=cv2.LINE_8)

    # Add numbers to the clock
    font = cv2.FONT_HERSHEY_SIMPLEX
    font_scale = 0.3
    font_thickness = 1
    num_offset = int(radius * 0.85)  # Slightly outside the clock face

    for i in range(1, 13):
        angle = math.radians(270 + (i * 30))  # 30° per number
        num_x = int(center[0] + num_offset * math.cos(angle)) - int(
            font_scale * 0.2 * radius)  # Adjust x to center numbers better
        num_y = int(center[1] + num_offset * math.sin(angle)) + int(
            font_scale * 0.2 * radius)  # Adjust y to center numbers better

        #cv2.putText(img, str(i), (num_x, num_y), font, font_scale, (255, 255, 255), font_thickness, cv2.FILLED)

    # Compute angles for clock hands
    second_angle = math.radians(270 + (seconds * 6))  # 6° per second
    minute_angle = math.radians(270 + ((minutes + seconds / 60.0) * 6))  # 6° per full minute
    hour_angle = math.radians(270 + (hours * 30 + minutes * 0.5))  # 30° per hour, 0.5° per minute

    # Hand lengths
    second_length = int(radius * 0.75)
    minute_length = int(radius * 0.75)
    hour_length = int(radius * 0.5)

    # Compute hand end points
    def compute_hand(angle, length):
        return int(center[0] + length * math.cos(angle)), int(center[1] + length * math.sin(angle))

    # Draw clock hands
    cv2.line(img, center, compute_hand(hour_angle, hour_length), (255, 255, 255), 6, cv2.LINE_8)  # Hour hand (red)
    cv2.line(img, center, compute_hand(minute_angle, minute_length), (255, 255, 255), 4, cv2.LINE_8)  # Minute hand (green)
    cv2.line(img, center, compute_hand(second_angle, second_length), (255, 0, 0), 2, cv2.LINE_8)  # Second hand (blue)

    byte_list = img.flatten().tolist()
    return FramePayload(img.shape[1], img.shape[0], bytes(byte_list))


def parse_args():
    import argparse

    parser = argparse.ArgumentParser(
        prog="send_clock.py",
        description="Send current time as clock to a running RasGB-Pi server",
        exit_on_error=True
    )
    parser.add_argument("-s", "--peer", type=str, required=True, help="RasGB-Pi server address <host>:<port>")

    args = parser.parse_args()
    return args

if __name__ == "__main__":
    main()
