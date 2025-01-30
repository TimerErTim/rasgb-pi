import abc
from dataclasses import dataclass

import cv2


class ResizeMethod(abc.ABC):
    @abc.abstractmethod
    def get_cv2_method(self) -> int:
        raise NotImplementedError()


@dataclass(frozen=True)
class ResizeLanczos4(ResizeMethod):
    def get_cv2_method(self):
        return cv2.INTER_LANCZOS4

@dataclass(frozen=True)
class ResizeArea(ResizeMethod):
    def get_cv2_method(self):
        return cv2.INTER_AREA