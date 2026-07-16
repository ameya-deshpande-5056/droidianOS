#!/usr/bin/python3
from __future__ import annotations

import os
import shutil
import subprocess

import gi

gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, Gdk  # noqa: E402


CSS_PATH = "/usr/share/droidianos-session-defaults/droidianos-ui.css"


def load_css() -> None:
    provider = Gtk.CssProvider()
    provider.load_from_path(CSS_PATH)
    screen = Gdk.Screen.get_default()
    if screen is None:
        return
    Gtk.StyleContext.add_provider_for_screen(screen, provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)


class InstallerHub(Gtk.Window):
    def __init__(self) -> None:
        super().__init__(title="Install droidianOS")
        self.set_default_size(1100, 720)
        self.set_border_width(24)
        self.connect("destroy", Gtk.main_quit)
        self.get_style_context().add_class("droidianos-window")

        root = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=24)
        self.add(root)

        left = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=16)
        left.set_size_request(470, -1)
        root.pack_start(left, False, False, 0)

        hero = Gtk.Frame()
        hero.get_style_context().add_class("droidianos-hero")
        left.pack_start(hero, False, False, 0)

        hero_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=16)
        hero_box.set_border_width(16)
        hero.add(hero_box)

        logo = Gtk.Image()
        logo.set_from_file("/usr/share/pixmaps/droidianos-logo.svg")
        hero_box.pack_start(logo, False, False, 0)

        hero_text = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        hero_box.pack_start(hero_text, True, True, 0)

        kicker = Gtk.Label(label="Live image entry point")
        kicker.set_xalign(0.0)
        kicker.get_style_context().add_class("droidianos-kicker")
        hero_text.pack_start(kicker, False, False, 0)

        title = Gtk.Label(label="droidianOS install hub")
        title.set_xalign(0.0)
        title.get_style_context().add_class("droidianos-display")
        hero_text.pack_start(title, False, False, 0)

        subtitle = Gtk.Label(
            label=(
                "This live image gives you a branded droidianOS entry point first.\n"
                "Install the system, review the full setup flow, or continue into the live session."
            )
        )
        subtitle.set_xalign(0.0)
        subtitle.set_line_wrap(True)
        subtitle.get_style_context().add_class("droidianos-subtitle")
        hero_text.pack_start(subtitle, False, False, 0)

        left.pack_start(Gtk.Separator(orientation=Gtk.Orientation.HORIZONTAL), False, False, 0)

        self._status = Gtk.Label(label="Ready to install.")
        self._status.set_xalign(0.0)
        self._status.set_line_wrap(True)
        self._status.get_style_context().add_class("droidianos-muted")
        left.pack_start(self._status, False, False, 0)

        button_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        left.pack_start(button_box, False, False, 0)

        install = Gtk.Button(label="Install droidianOS")
        install.get_style_context().add_class("suggested-action")
        install.connect("clicked", lambda *_: self._launch_installer())
        button_box.pack_start(install, False, False, 0)

        desktops = Gtk.Button(label="Choose desktop after install")
        desktops.get_style_context().add_class("droidianos-secondary")
        desktops.connect("clicked", lambda *_: self._launch_desktop_chooser())
        button_box.pack_start(desktops, False, False, 0)

        live = Gtk.Button(label="Continue in live session")
        live.get_style_context().add_class("droidianos-secondary")
        live.connect("clicked", lambda *_: self._continue_live())
        button_box.pack_start(live, False, False, 0)

        quit_btn = Gtk.Button(label="Exit")
        quit_btn.get_style_context().add_class("droidianos-danger")
        quit_btn.connect("clicked", lambda *_: self._quit())
        button_box.pack_start(quit_btn, False, False, 0)

        right = Gtk.Frame(label="What happens next")
        right.get_style_context().add_class("droidianos-panel")
        root.pack_start(right, True, True, 0)

        info = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=14)
        info.set_border_width(18)
        right.add(info)

        flow = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=10)
        info.pack_start(flow, False, False, 0)

        for idx, label_text in enumerate(["Live", "Install", "First boot", "Desktop"], start=1):
            step = Gtk.Label(label=f"{idx}. {label_text}")
            step.get_style_context().add_class("droidianos-step")
            if idx == 1:
                step.get_style_context().add_class("droidianos-step-active")
            elif idx < 4:
                step.get_style_context().add_class("droidianos-step-done")
            flow.pack_start(step, False, False, 0)

        for title_text, body_text in [
            (
                "1. Live session",
                "Boot into the branded droidianOS live environment and start from here.",
            ),
            (
                "2. Install system",
                "Install the base system, droidianOS packages, and the Waydroid setup hook.",
            ),
            (
                "3. First boot",
                "Reboot into the installed system, then finish first-login setup.",
            ),
            (
                "4. Pick desktop",
                "Choose your preferred desktop environment and optional Flatpak support.",
            ),
        ]:
            card = Gtk.Frame()
            card.get_style_context().add_class("droidianos-card")
            row = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
            row.set_border_width(12)
            card.add(row)

            heading = Gtk.Label(label=title_text)
            heading.set_xalign(0.0)
            heading.get_style_context().add_class("droidianos-subtitle")
            row.pack_start(heading, False, False, 0)

            body = Gtk.Label(label=body_text)
            body.set_xalign(0.0)
            body.set_line_wrap(True)
            body.get_style_context().add_class("droidianos-muted")
            row.pack_start(body, False, False, 0)

            info.pack_start(card, False, False, 0)

        note = Gtk.Label(
            label=(
                "The live session is the primary UX, so the user sees droidianOS branding "
                "before any underlying installer screens."
            )
        )
        note.set_xalign(0.0)
        note.set_line_wrap(True)
        info.pack_end(note, False, False, 0)

    def _launch_installer(self) -> None:
        launcher = shutil.which("debian-installer-launcher")
        if launcher is None:
            self._message(Gtk.MessageType.ERROR, "Installer backend launcher is missing from the live image.")
            return

        self._status.set_text("Starting installation backend...")
        subprocess.Popen([launcher])
        self._info("Installer launched", "The installation backend has been started from the droidianOS hub.")
        self._quit()

    def _launch_desktop_chooser(self) -> None:
        chooser = shutil.which("droidianos-desktop-chooser")
        if chooser is None:
            self._message(Gtk.MessageType.ERROR, "Desktop chooser is missing from the live image.")
            return
        subprocess.Popen([chooser])
        self._quit()

    def _continue_live(self) -> None:
        self._info("Live session", "The live session will remain open. You can launch the installer again from the desktop.")

    def _message(self, message_type: Gtk.MessageType, text: str) -> None:
        dialog = Gtk.MessageDialog(
            transient_for=self,
            flags=0,
            message_type=message_type,
            buttons=Gtk.ButtonsType.OK,
            text="droidianOS installer",
        )
        dialog.format_secondary_text(text)
        dialog.run()
        dialog.destroy()

    def _info(self, title: str, text: str) -> None:
        dialog = Gtk.MessageDialog(
            transient_for=self,
            flags=0,
            message_type=Gtk.MessageType.INFO,
            buttons=Gtk.ButtonsType.OK,
            text=title,
        )
        dialog.format_secondary_text(text)
        dialog.run()
        dialog.destroy()

    def _quit(self) -> None:
        Gtk.main_quit()


def main() -> int:
    load_css()
    win = InstallerHub()
    win.show_all()
    Gtk.main()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
