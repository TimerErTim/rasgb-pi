#!/usr/bin/env -S /bin/sh -c '"$(dirname "$0")/venv/bin/python" "$0" "$@"'

from __future__ import annotations

import asyncio
import datetime
import math
import threading
import time
from dataclasses import dataclass
from sys import stderr
from typing import Optional, overload

import cv2
import pyowm
import numpy as np
from PIL import Image, ImageFont, ImageDraw
from pyowm.weatherapi25.observation import Observation

from lib.client import RasgbPiClient, FrameLocation, DisplayMetadata, FramePayload
from lib.compression import ZstdCompression, parse_compression, NoCompression


def main():
    args = parse_args()
    server: str = args.peer
    host = server.rsplit(":", 1)[0]
    port = server.rsplit(":", 1)[1] if len(server.rsplit(":", 1)) == 2 else "8081"

    client = RasgbPiClient(f"http://{host}:{port}", time_buffer_ms=1000, default_compression=args.compression, default_channel=args.channel)

    if args.location is not None or args.openweather_key is not None:
        if args.location is None or args.openweather_key is None:
            raise ValueError("Both --location and --openweather-key must be specified")
        weather_conf = WeatherConfig(open_weather_api_key=args.openweather_key, location=args.location)
    else:
        weather_conf = None

    renderer = ClockRenderer(weather=weather_conf)

    render_clock_to_server(client, renderer)

def render_clock_to_server(client: RasgbPiClient, clock_renderer: ClockRenderer):
    stop_sending = client.send_generator(
        FrameLocation(unix_micros=time.time_ns() // 1000),
        make_frame=clock_renderer.make_clock_frame,
        max_fps=5
    )
    try:
        while True:
            time.sleep(1000)
    except KeyboardInterrupt:
        print("quitting...")
    finally:
        stop_sending()

@dataclass
class WeatherConfig:
    open_weather_api_key: str
    location: str
    _current_observation: Observation = None

    def __post_init__(self):
        owm_mgr = pyowm.OWM(self.open_weather_api_key).weather_manager()
        event_loop = asyncio.new_event_loop()
        def run_event_loop():
            event_loop.set_exception_handler(lambda loop, context: print(context))
            asyncio.set_event_loop(event_loop)
            event_loop.run_forever()
        threading.Thread(
            target=run_event_loop, daemon=True, name="OWM event loop"
        ).start()

        async def update_observation():
            while True:
                try:
                    new_observation = owm_mgr.weather_at_place(self.location)
                    if new_observation is None:
                        raise ValueError("Observation is None")
                    self._current_observation = new_observation
                except Exception as e:
                    print(f"Failed to get observation at location '{self.location}': {e}", file=stderr)
                await asyncio.sleep(60)

        asyncio.run_coroutine_threadsafe(
            update_observation(),
            loop=event_loop
        )

    def current_observation(self) -> Observation | None:
        return self._current_observation



@dataclass
class ClockRenderer:
    weather: Optional[WeatherConfig] = None
    _owm_mgr: pyowm.owm.weather_manager = None

    def __post_init__(self):
        if self.weather is not None:
            self._owm_mgr = pyowm.OWM(self.weather.open_weather_api_key).weather_manager()

    def make_clock_frame(self, display: DisplayMetadata) -> FramePayload:
        # Create a blank white image
        img = np.zeros((display.height, display.width, 3), dtype=np.uint8)

        img = self.draw_clock_to_image(img)
        img = self.draw_weather_to_image(img)

        byte_list = img.flatten().tolist()
        return FramePayload(img.shape[1], img.shape[0], bytes(byte_list))

    def draw_clock_to_image(self, img: cv2.Mat) -> cv2.Mat:
        # Convert timestamp to datetime
        timestamp = datetime.datetime.fromtimestamp(time.time() - 1)

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

        # Draw hour and minute ticks
        for i in range(60):  # 60 ticks (every minute)
            angle = math.radians(270 + (i * 6))  # Each tick is 6° apart
            outer_length = radius - 2  # Outer tick length
            inner_length = radius - 6  # Inner tick length
            if i % 60 == 0:  # half long tick for 12 hour mark
                outer_length += 1
            elif i % 5 == 0:  # Longer ticks every 5 mins
                outer_length += 4

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
        base_radius = radius - 10
        hour_thickness = 6
        hour_radius = base_radius - hour_thickness // 2
        minute_thickness = 2
        minute_radius = base_radius - minute_thickness // 2
        second_thickness = 1
        second_radius = base_radius

        def circle_pos(angle: float, radius: float) -> tuple[int, int]:
            return (
                int(center[0] + radius * math.cos(math.radians(angle - 90))),
                int(center[1] + radius * math.sin(math.radians(angle - 90)))
            )

        cv2.circle(img, circle_pos(hours_degrees, hour_radius), 1, (255, 255, 255), hour_thickness, cv2.FILLED)
        cv2.circle(img, circle_pos(minute_degrees, minute_radius), 1, (0, 0, 255), minute_thickness, cv2.FILLED)
        cv2.circle(img, circle_pos(second_degrees, second_radius), 1, (255, 0, 0), second_thickness, cv2.LINE_8)

        # Draw date
        font = cv2.FONT_HERSHEY_SIMPLEX
        font_scale = 0.65
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

        return img

    def draw_weather_to_image(self, img: cv2.Mat) -> cv2.Mat:
        curr_obsv = self.weather.current_observation() if self.weather is not None else None
        if curr_obsv is None:
            return img

        weather = curr_obsv.weather
        temperature = weather.temperature('celsius')["temp"]
        detailed_weather_status = weather.detailed_status
        weather_status = weather.status

        img_height, img_width, _ = img.shape

        # Clock radius
        radius = int(min(img_width, img_height) // 2.10)

        # Draw data
        font = cv2.FONT_HERSHEY_SIMPLEX
        font_scale = 0.35
        font_color = (255, 255, 255)
        font_thickness = 1

        text = detailed_weather_status.lower()
        weather_size = cv2.getTextSize(text, font, font_scale, font_thickness)[0]
        distance_from_center = weather_size[1] + 14
        circle_width = 2 * math.sqrt(radius * radius - distance_from_center * distance_from_center)
        if weather_size[0] > circle_width - 32:
            text = weather_status.lower()
            weather_size = cv2.getTextSize(text, font, font_scale, font_thickness)[0]
        weather_x = img_width // 2 - weather_size[0] // 2
        weather_y = img_height // 2 + distance_from_center
        cv2.putText(img, text, (weather_x, weather_y), font, font_scale, font_color, font_thickness)


        pil_image = Image.fromarray(img)
        font = ImageFont.truetype("Pillow/Tests/fonts/DejaVuSans.ttf", 11)
        draw = ImageDraw.Draw(pil_image)

        # Draw non-ascii text onto image
        text = f"{temperature:02.1f}°C"
        temp_size = draw.textbbox((0, 0), text, font=font)  # Get text size (Pillow 8.0+)
        temp_width = temp_size[2] - temp_size[0]
        temp_height = temp_size[3] - temp_size[1]
        temp_x = img_width // 2
        temp_y = weather_y + temp_height // 2 + 1
        draw.fontmode = "1"
        draw.text((temp_x, temp_y), text, font=font, fill=font_color, anchor="mt")

        return np.asarray(pil_image, dtype=np.uint8, copy=False)




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
    parser.add_argument("--location", type=str, help="OpenWeatherMap location (e.g. 'Linz,AT', 'London,GB')")
    parser.add_argument("--openweather-key", type=str, help="OpenWeatherMap API key")

    args = parser.parse_args()
    return args

if __name__ == "__main__":
    main()
