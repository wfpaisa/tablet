#!/usr/bin/env python3
"""Exercise the native shortcut host in an isolated X server without a daemon."""
import os
import ctypes
import ctypes.util
import json
import socket
from pathlib import Path
import subprocess
import time
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "target/native-smoke"
OUT.mkdir(parents=True, exist_ok=True)
runtime=OUT / "runtime"
runtime.mkdir(mode=0o700,exist_ok=True)
env = os.environ.copy()
env.pop("WAYLAND_DISPLAY", None)
env.update(
    DISPLAY=":87",
    XDG_CONFIG_HOME=str(OUT / "config"),
    XDG_RUNTIME_DIR=str(runtime),
    WINIT_UNIX_BACKEND="x11",
    __EGL_VENDOR_LIBRARY_FILENAMES="/usr/share/glvnd/egl_vendor.d/50_mesa.json",
)
with (OUT / "xvfb.log").open("w") as server_log, (OUT / "host.log").open("w") as app_log:
    server = subprocess.Popen(
        ["Xvfb", ":87", "-screen", "0", "1100x720x24", "-nolisten", "tcp", "-extension", "GLX"],
        stdout=server_log, stderr=server_log, env=env,
    )
    try:
        socket_path=runtime / "tabletd.sock"
        socket_path.unlink(missing_ok=True)
        daemon=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM)
        daemon.bind(str(socket_path))
        daemon.listen(1)
        daemon.settimeout(3)
        time.sleep(0.6)
        if server.poll() is not None:
            raise RuntimeError("Xvfb failed; see target/native-smoke/xvfb.log")
        app = subprocess.Popen(
            [str(ROOT / "target/debug/tablet-center"), "--shortcut-host"],
            env=env, stdin=subprocess.PIPE, stdout=app_log, stderr=app_log, text=True,
        )
        try:
            time.sleep(0.8)
            for label, message in [
                ("keypad", '{"name":"keypad","at":[550,360]}'),
                ("hidden", '{"name":"keypad"}'),
                ("reopened", '{"name":"app_shortcuts","at":[550,360]}'),
            ]:
                app.stdin.write(message + "\n")
                app.stdin.flush()
                time.sleep(1)
                if app.poll() is not None:
                    raise RuntimeError("Host exited; see target/native-smoke/host.log")
                destination = OUT / (label + ".png")
                subprocess.run(["import", "-display", ":87", "-window", "root", str(destination)], env=env, check=True)
                image = Image.open(destination).convert("RGB")
                lit = sum(image.convert("L").histogram()[41:])
                if label == "hidden":
                    assert lit == 0, f"Toggle left {lit} visible pixels"
                else:
                    assert lit > 30000, f"Menu was not drawn: {label}, {lit} visible pixels"
                print(f"{label}: {lit} visible pixels", flush=True)
            xlib=ctypes.CDLL(ctypes.util.find_library("X11"))
            xtest=ctypes.CDLL(ctypes.util.find_library("Xtst"))
            xlib.XOpenDisplay.argtypes=[ctypes.c_char_p]
            xlib.XOpenDisplay.restype=ctypes.c_void_p
            xlib.XFlush.argtypes=[ctypes.c_void_p]
            xlib.XCloseDisplay.argtypes=[ctypes.c_void_p]
            xtest.XTestFakeMotionEvent.argtypes=[ctypes.c_void_p,ctypes.c_int,ctypes.c_int,ctypes.c_int,ctypes.c_ulong]
            xtest.XTestFakeButtonEvent.argtypes=[ctypes.c_void_p,ctypes.c_uint,ctypes.c_int,ctypes.c_ulong]
            xtest.XTestFakeKeyEvent.argtypes=[ctypes.c_void_p,ctypes.c_uint,ctypes.c_int,ctypes.c_ulong]
            display=xlib.XOpenDisplay(b":87")
            assert display
            try:
                xtest.XTestFakeMotionEvent(display,0,421,291,0)
                for pressed in [1,0]:
                    xtest.XTestFakeButtonEvent(display,1,pressed,0)
                xlib.XFlush(display)
                connection,_=daemon.accept()
                with connection:
                    payload=json.loads(connection.makefile("r").readline())
                    assert payload == {"cmd":"run","action":{"type":"keystroke","keys":["KEY_LEFTCTRL","KEY_Z"],"label":"Undo"}},payload
                    connection.sendall(b'{"ok":true,"error":null,"device":null,"pen":null}'+bytes([10]))
                subprocess.run(["import","-display",":87","-window","root",str(OUT / "after-action.png")],env=env,check=True)
                assert max(Image.open(OUT / "after-action.png").convert("L").getextrema())==0
                print("Undo: one Run request on the isolated socket; window hidden",flush=True)
                app.stdin.write('{"name":"express_menu"}'+chr(10))
                app.stdin.flush()
                time.sleep(.7)
                xlib.XDefaultRootWindow.argtypes=[ctypes.c_void_p]
                xlib.XDefaultRootWindow.restype=ctypes.c_ulong
                xlib.XQueryTree.argtypes=[ctypes.c_void_p,ctypes.c_ulong,ctypes.POINTER(ctypes.c_ulong),ctypes.POINTER(ctypes.c_ulong),ctypes.POINTER(ctypes.POINTER(ctypes.c_ulong)),ctypes.POINTER(ctypes.c_uint)]
                xlib.XFetchName.argtypes=[ctypes.c_void_p,ctypes.c_ulong,ctypes.POINTER(ctypes.c_void_p)]
                xlib.XFree.argtypes=[ctypes.c_void_p]
                xlib.XSetInputFocus.argtypes=[ctypes.c_void_p,ctypes.c_ulong,ctypes.c_int,ctypes.c_ulong]
                def find_overlay(window):
                    name=ctypes.c_void_p()
                    if xlib.XFetchName(display,window,ctypes.byref(name)) and name:
                        title=ctypes.string_at(name)
                        xlib.XFree(name)
                        if title==b"Tablet Center":
                            return window
                    root=ctypes.c_ulong()
                    parent=ctypes.c_ulong()
                    children=ctypes.POINTER(ctypes.c_ulong)()
                    count=ctypes.c_uint()
                    if xlib.XQueryTree(display,window,ctypes.byref(root),ctypes.byref(parent),ctypes.byref(children),ctypes.byref(count)):
                        found=None
                        try:
                            for i in range(count.value):
                                found=find_overlay(children[i])
                                if found:
                                    break
                        finally:
                            if children:
                                xlib.XFree(children)
                        return found
                    return None
                window=find_overlay(xlib.XDefaultRootWindow(display))
                assert window, "Overlay viewport missing"
                # Xvfb has no window manager to grant keyboard focus.
                xlib.XSetInputFocus(display,window,1,0)
                xlib.XFlush(display)
                time.sleep(.1)
                for pressed in [1,0]:
                    xtest.XTestFakeKeyEvent(display,9,pressed,0)
                xlib.XFlush(display)
                time.sleep(.4)
                subprocess.run(["import","-display",":87","-window","root",str(OUT / "escape.png")],env=env,check=True)
                assert max(Image.open(OUT / "escape.png").convert("L").getextrema())==0
                print("Escape: closed",flush=True)
            finally:
                xlib.XCloseDisplay(display)
                daemon.close()
                socket_path.unlink(missing_ok=True)
            app.stdin.close()
            app.wait(timeout=5)
            assert app.returncode == 0, f"EOF exit: {app.returncode}"
            print("EOF: clean exit", flush=True)
        finally:
            if app.poll() is None:
                app.terminate()
                app.wait(timeout=5)
    finally:
        server.terminate()
        server.wait(timeout=5)
