"""Launch the actual extracted AppImage and require visible application text."""

import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time

from PIL import ImageGrab


def main():
    appimage = Path(sys.argv[1]).resolve()
    diagnostics = Path(sys.argv[2]).resolve()
    diagnostics.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="cgs-render-check-") as temporary:
        unpack = subprocess.run(
            [str(appimage), "--appimage-extract"],
            cwd=temporary, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
            text=True, timeout=120,
        )
        (diagnostics / "extract.log").write_text(unpack.stdout, encoding="utf-8")
        if unpack.returncode:
            raise RuntimeError("AppImage extraction failed")

        apprun = Path(temporary) / "squashfs-root" / "AppRun"
        environment = os.environ.copy()
        # Exercise the defaults from Rust, rather than supplying the workaround
        # externally and accidentally hiding a broken application startup.
        environment.pop("WEBKIT_DISABLE_DMABUF_RENDERER", None)
        environment.pop("WEBKIT_DISABLE_COMPOSITING_MODE", None)
        screenshot = diagnostics / "window.png"
        with (diagnostics / "startup.log").open("w", encoding="utf-8") as log:
            process = subprocess.Popen(
                [str(apprun)], cwd=apprun.parent, env=environment,
                stdout=log, stderr=subprocess.STDOUT, start_new_session=True,
            )
            try:
                deadline = time.monotonic() + 60
                while time.monotonic() < deadline:
                    if process.poll() is not None:
                        raise RuntimeError(f"Application exited early: {process.returncode}")
                    windows = subprocess.run(
                        ["xdotool", "search", "--onlyvisible", "--name", "Crocodile Gena Studio"],
                        capture_output=True, text=True, timeout=5,
                    )
                    if windows.returncode == 0:
                        screen = ImageGrab.grab()
                        screen.resize((screen.width * 2, screen.height * 2)).save(screenshot)
                        ocr = subprocess.run(
                            ["tesseract", str(screenshot), "stdout", "-l", "eng", "--psm", "11"],
                            capture_output=True, text=True, timeout=15,
                        )
                        (diagnostics / "visible-text.txt").write_text(ocr.stdout, encoding="utf-8")
                        # This app is undecorated: both the titlebar and sidebar
                        # are painted by WebKit. Require actual page text, not
                        # just an X11 window that may contain a black WebView.
                        visible = ocr.stdout.lower()
                        if "crocodile" in visible and "gena" in visible:
                            print("AppImage startup and visible frontend rendering passed.")
                            return
                    time.sleep(2)
                raise RuntimeError("No rendered frontend within 60 seconds; see Linux-rendering-diagnostics")
            finally:
                if process.poll() is None:
                    os.killpg(process.pid, signal.SIGTERM)
                    try:
                        process.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        os.killpg(process.pid, signal.SIGKILL)
                        process.wait(timeout=5)


if __name__ == "__main__":
    main()
