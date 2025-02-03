#!/usr/bin/env -S /bin/sh -c '"$(dirname "$0")/venv/bin/python" "$0" "$@"'

import datetime
import math
import time

import cv2
import numpy as np

from lib.client import RasgbPiClient, FrameLocation, DisplayMetadata, FramePayload
from lib.compression import ZstdCompression, parse_compression, NoCompression


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

    # Create a blank white image
    img = np.zeros((display.height, display.width, 3), dtype=np.uint8)

    draw_clock_to_image(img, timestamp)

    byte_list = img.flatten().tolist()
    return FramePayload(img.shape[1], img.shape[0], bytes(byte_list))

def draw_clock_to_image(img: cv2.Mat, timestamp: datetime.datetime):
    img_height, img_width, _ = img.shape

    hours = timestamp.hour % 12  # Convert to 12-hour format
    minutes = timestamp.minute
    seconds = timestamp.second

    day = timestamp.day
    month = timestamp.strftime("%b").upper()
    year = timestamp.year

    # Clock properties
    center = (img_width // 2, img_height // 2)
    radius = int(min(img_width, img_height) // 2.10)  # Adjust radius to fit inside the image

    # Draw clock circle
    # cv2.circle(img, center, radius, (255, 255, 255), thickness)

    # Draw clock center point
    # cv2.circle(img, center, 4, (255, 255, 255), -1)

    # Draw hour and minute ticks
    for i in range(60):  # 60 ticks (every minute)
        angle = math.radians(270 + (i * 6))  # Each tick is 6° apart
        outer_length = radius - 6  # Outer tick length
        inner_length = radius - 10  # Inner tick length
        if i % 60 == 0:  # half long tick for 12 hour mark
            inner_length -= 1
        elif i % 5 == 0:  # Longer ticks every 5 mins
            inner_length -= 4

        outer_point = (
            int(center[0] + outer_length * math.cos(angle)),
            int(center[1] + outer_length * math.sin(angle))
        )
        inner_point = (
            int(center[0] + inner_length * math.cos(angle)),
            # Longer ticks every 5 mins
            int(center[1] + inner_length * math.sin(angle))
        )
        cv2.line(img, outer_point, inner_point, (255, 255, 255), 2 if i % 5 == 0 else 1, lineType=cv2.LINE_8)

    # Compute angles for clock hands
    second_degrees = seconds * 6  # 6° per second
    minute_degrees = (minutes + seconds / 60.0) * 6  # 6° per full minute
    hours_degrees = hours * 30 + minutes * 0.5  # 30° per hour, 0.5° per minute

    # Hand radius
    base_radius = radius - 4
    hour_thickness = 6
    hour_radius = base_radius + hour_thickness // 2
    minute_thickness = 2
    minute_radius = base_radius + minute_thickness // 2 + 1
    second_thickness = 1
    second_radius = base_radius

    # Draw clock hands
    cv2.ellipse(img, center, (hour_radius, hour_radius), -90, 0, max(hours_degrees, 1), (255, 255, 255), hour_thickness, cv2.FILLED)

    cv2.ellipse(img, center, (minute_radius -1 , minute_radius - 1), -90, 0, minute_degrees, (0, 0, 0), minute_thickness + 3, cv2.FILLED)
    cv2.ellipse(img, center, (minute_radius, minute_radius), -90, 0, max(minute_degrees, 1), (0, 0, 255), minute_thickness, cv2.FILLED)

    cv2.ellipse(img, center, (second_radius, second_radius), -90, 0, second_degrees, (0, 0, 0), second_thickness + 1, cv2.FILLED)
    cv2.ellipse(img, center, (second_radius, second_radius), -90, 0, max(second_degrees, 1), (255, 0, 0), second_thickness, cv2.FILLED)


    # Draw date
    font = cv2.FONT_HERSHEY_SIMPLEX
    font_scale = 0.7
    font_color = (255, 255, 255)
    font_thickness = 1

    text = f"{year}"
    year_size = cv2.getTextSize(text, font, font_scale, font_thickness)[0]
    year_x = img_width // 2 - year_size[0] // 2
    year_y = img_height // 2 + year_size[1] // 2
    cv2.putText(img, text, (year_x, year_y), font, font_scale, font_color, font_thickness)

    text = f"{month:02}"
    month_size = cv2.getTextSize(text, font, font_scale, font_thickness)[0]
    month_x = img_width // 2 - month_size[0] // 2
    month_y = year_y - month_size[1] - 2
    cv2.putText(img, text, (month_x, month_y), font, font_scale, font_color, font_thickness)

    text = f"{day:02}"
    day_size = cv2.getTextSize(text, font, font_scale, font_thickness)[0]
    day_x = img_width // 2 - day_size[0] // 2
    day_y = month_y - day_size[1] - 2
    cv2.putText(img, text, (day_x, day_y), font, font_scale, font_color, font_thickness)

def parse_args():
    import argparse

    parser = argparse.ArgumentParser(
        prog="send_clock.py",
        description="Send current time as clock to a running RasGB-Pi server",
        exit_on_error=True
    )
    parser.add_argument("-s", "--peer", type=str, required=True, help="RasGB-Pi server address <host>:<port>")
    parser.add_argument("-c", "--channel", type=int, help="Channel to send the clock to")
    parser.add_argument("--compression", type=parse_compression, default=ZstdCompression(level=3),
                        help="Compression to use for sending pixel data (e.g. 'zstd:3', 'none')")

    args = parser.parse_args()
    return args

if __name__ == "__main__":
    main()
