#!/usr/bin/python3
from __future__ import annotations

import os
import subprocess
import sys
from dataclasses import dataclass

import gi

gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, GdkPixbuf  # noqa: E402


@dataclass(frozen=True)
class DesktopOption:
    name: str
    description: str
    screenshot: str
    packages: str
    remove_packages: str
    session: str
    display_manager: str
    install_flatpak: bool


OPTIONS = [
    DesktopOption("KDE Plasma", "Feature-rich, polished, and highly configurable.", "/usr/share/droidianos-desktop-chooser/previews/kde-plasma.png", "task-kde-desktop", "openbox lightdm lightdm-gtk-greeter gdm3", "startplasma-x11", "sddm", False),
    DesktopOption("GNOME", "Modern, simple, and touch-friendly.", "/usr/share/droidianos-desktop-chooser/previews/gnome.png", "task-gnome-desktop", "openbox lightdm lightdm-gtk-greeter sddm", "gnome-session", "gdm3", False),
    DesktopOption("Cinnamon", "Traditional workflow with a familiar layout.", "/usr/share/droidianos-desktop-chooser/previews/cinnamon.png", "task-cinnamon-desktop", "openbox lightdm lightdm-gtk-greeter sddm", "cinnamon-session", "gdm3", False),
    DesktopOption("XFCE", "Lightweight, stable, and fast on modest hardware.", "/usr/share/droidianos-desktop-chooser/previews/xfce.png", "task-xfce-desktop", "openbox lightdm lightdm-gtk-greeter gdm3 sddm", "startxfce4", "lightdm", False),
    DesktopOption("LXQt", "Very light Qt-based desktop with a clean panel setup.", "/usr/share/droidianos-desktop-chooser/previews/lxqt.png", "task-lxqt-desktop", "openbox lightdm lightdm-gtk-greeter gdm3", "startlxqt", "sddm", False),
    DesktopOption("LXDE", "Classic ultra-light desktop for low-resource systems.", "/usr/share/droidianos-desktop-chooser/previews/lxde.png", "task-lxde-desktop", "openbox lightdm lightdm-gtk-greeter gdm3 sddm", "startlxde", "lightdm", False),
    DesktopOption("Openbox", "Minimal window manager for a very small footprint.", "/usr/share/droidianos-desktop-chooser/previews/openbox.png", "openbox lxterminal feh", "", "openbox-session", "lightdm", True),
]


class DesktopChooser(Gtk.Window):
    def __init__(self) -> None:
        super().__init__(title="Choose your desktop")
        self.set_default_size(980, 560)
        self.set_border_width(18)
        self.connect("destroy", Gtk.main_quit)

        root = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        self.add(root)

        title = Gtk.Label(label="Pick the desktop environment you want installed after setup.")
        title.set_xalign(0.0)
        title.get_style_context().add_class("title-3")
        root.pack_start(title, False, False, 0)

        subtitle = Gtk.Label(label="Each choice includes a screenshot preview and a short description.")
        subtitle.set_xalign(0.0)
        subtitle.set_line_wrap(True)
        root.pack_start(subtitle, False, False, 0)

        scroller = Gtk.ScrolledWindow()
        scroller.set_policy(Gtk.PolicyType.AUTOMATIC, Gtk.PolicyType.AUTOMATIC)
        root.pack_start(scroller, True, True, 0)

        box = Gtk.ListBox()
        box.set_selection_mode(Gtk.SelectionMode.SINGLE)
        scroller.add(box)

        for option in OPTIONS:
            row = self._build_row(option)
            row.option = option  # type: ignore[attr-defined]
            box.add(row)

        box.connect("row-activated", self._on_activate)
        self.listbox = box

        button_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        button_box.set_halign(Gtk.Align.END)
        root.pack_start(button_box, False, False, 0)

        cancel = Gtk.Button(label="Cancel")
        cancel.connect("clicked", lambda *_: self._quit())
        button_box.pack_start(cancel, False, False, 0)

        select = Gtk.Button(label="Install Selected")
        select.get_style_context().add_class("suggested-action")
        select.connect("clicked", lambda *_: self._install_selected())
        button_box.pack_start(select, False, False, 0)

    def _build_row(self, option: DesktopOption) -> Gtk.ListBoxRow:
        row = Gtk.ListBoxRow()
        outer = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
        outer.set_border_width(10)
        row.add(outer)

        image = Gtk.Image()
        try:
            pixbuf = GdkPixbuf.Pixbuf.new_from_file_at_scale(option.screenshot, 260, 146, True)
            image.set_from_pixbuf(pixbuf)
        except Exception:
            image.set_from_icon_name("image-missing", Gtk.IconSize.DIALOG)
        outer.pack_start(image, False, False, 0)

        text = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        outer.pack_start(text, True, True, 0)

        name = Gtk.Label(label=option.name)
        name.set_xalign(0.0)
        name.get_style_context().add_class("title-2")
        text.pack_start(name, False, False, 0)

        desc = Gtk.Label(label=option.description)
        desc.set_xalign(0.0)
        desc.set_line_wrap(True)
        desc.set_max_width_chars(60)
        text.pack_start(desc, False, False, 0)

        pkg = Gtk.Label(label=f"Installs: {option.packages}")
        pkg.set_xalign(0.0)
        pkg.get_style_context().add_class("dim-label")
        text.pack_start(pkg, False, False, 0)

        return row

    def _selected_option(self) -> DesktopOption | None:
        row = self.listbox.get_selected_row()
        if row is None:
            return None
        return getattr(row, "option", None)

    def _on_activate(self, _listbox: Gtk.ListBox, row: Gtk.ListBoxRow) -> None:
        self.listbox.select_row(row)
        self._install_selected()

    def _install_selected(self) -> None:
        option = self._selected_option()
        if option is None:
            self._info("Desktop chooser", "Please select a desktop first.")
            return

        self._show_preview(option)
        display_manager = self._choose_display_manager(option)
        flatpak = " flatpak" if option.install_flatpak and self._ask_flatpak() else ""

        if not self._confirm(
            "Install desktop",
            f"Install {option.name} now?\n\nThis will install:\n{option.packages}{flatpak}\n\nThe initial GUI stack will be replaced with the selected desktop.",
        ):
            return

        cleanup = ""
        if option.remove_packages:
            cleanup = f" && apt-get remove -y {option.remove_packages}"

        cmd = (
            "apt-get update && "
            f"apt-get install -y {option.packages}{flatpak}"
            f"{cleanup} && "
            f"update-alternatives --set x-session-manager /usr/bin/{option.session} || true && "
            f"update-alternatives --set x-window-manager /usr/bin/{option.session} || true && "
            f"apt-get install -y {display_manager} && "
            f"systemctl enable {display_manager}.service || true"
        )
        status = subprocess.run(["pkexec", "sh", "-c", cmd])
        if status.returncode != 0:
            self._error("Desktop chooser", "Desktop installation failed.")
            return

        marker = os.path.expanduser("~/.config/droidianos/desktop-chooser-complete")
        os.makedirs(os.path.dirname(marker), exist_ok=True)
        with open(marker, "w", encoding="utf-8") as handle:
            handle.write(f"chosen={option.name}\ndisplay_manager={display_manager}\nflatpak={1 if flatpak else 0}\n")

        self._info("Desktop chooser", f"Installed {option.name} successfully.")
        self._quit()

    def _show_preview(self, option: DesktopOption) -> None:
        dialog = Gtk.Dialog(title=f"{option.name} preview", transient_for=self, flags=0)
        dialog.add_button("Continue", Gtk.ResponseType.OK)
        dialog.set_default_size(920, 520)
        content = dialog.get_content_area()
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        box.set_border_width(14)
        content.add(box)

        try:
            pixbuf = GdkPixbuf.Pixbuf.new_from_file_at_scale(option.screenshot, 860, 484, True)
            image = Gtk.Image.new_from_pixbuf(pixbuf)
        except Exception:
            image = Gtk.Image.new_from_icon_name("image-missing", Gtk.IconSize.DIALOG)
        box.pack_start(image, True, True, 0)

        label = Gtk.Label(label=f"{option.name}: {option.description}")
        label.set_xalign(0.0)
        label.set_line_wrap(True)
        box.pack_start(label, False, False, 0)

        dialog.show_all()
        dialog.run()
        dialog.destroy()

    def _ask_flatpak(self) -> bool:
        dialog = Gtk.MessageDialog(
            transient_for=self,
            flags=0,
            message_type=Gtk.MessageType.QUESTION,
            buttons=Gtk.ButtonsType.YES_NO,
            text="Install Flatpak support too?",
        )
        dialog.format_secondary_text("Only choose this if you want Flatpak available after setup.")
        response = dialog.run()
        dialog.destroy()
        return response == Gtk.ResponseType.YES

    def _choose_display_manager(self, option: DesktopOption) -> str:
        mapping = {
            "KDE Plasma": "sddm",
            "GNOME": "gdm3",
            "Cinnamon": "gdm3",
            "XFCE": "lightdm",
            "LXQt": "sddm",
            "LXDE": "lightdm",
            "Openbox": "lightdm",
        }
        default_dm = mapping.get(option.name, option.display_manager)
        choice = Gtk.Dialog(title="Choose display manager", transient_for=self, flags=0)
        choice.add_button("Use Default", Gtk.ResponseType.OK)
        choice.add_button("Cancel", Gtk.ResponseType.CANCEL)
        choice.set_default_size(620, 320)
        content = choice.get_content_area()
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        box.set_border_width(14)
        content.add(box)
        label = Gtk.Label(
            label=(
                f"{option.name} defaults to {default_dm}.\n"
                "You can keep it, or choose a different display manager for this install."
            )
        )
        label.set_xalign(0.0)
        label.set_line_wrap(True)
        box.pack_start(label, False, False, 0)

        combo = Gtk.ComboBoxText()
        for dm in ("lightdm", "gdm3", "sddm"):
            combo.append_text(dm)
        combo.set_active({"lightdm": 0, "gdm3": 1, "sddm": 2}.get(default_dm, 0))
        box.pack_start(combo, False, False, 0)

        choice.show_all()
        response = choice.run()
        selected = combo.get_active_text() or default_dm
        choice.destroy()
        if response != Gtk.ResponseType.OK:
            return default_dm
        return selected

    def _confirm(self, title: str, text: str) -> bool:
        dialog = Gtk.MessageDialog(
            transient_for=self,
            flags=0,
            message_type=Gtk.MessageType.QUESTION,
            buttons=Gtk.ButtonsType.OK_CANCEL,
            text=title,
        )
        dialog.format_secondary_text(text)
        response = dialog.run()
        dialog.destroy()
        return response == Gtk.ResponseType.OK

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

    def _error(self, title: str, text: str) -> None:
        dialog = Gtk.MessageDialog(
            transient_for=self,
            flags=0,
            message_type=Gtk.MessageType.ERROR,
            buttons=Gtk.ButtonsType.OK,
            text=title,
        )
        dialog.format_secondary_text(text)
        dialog.run()
        dialog.destroy()

    def _quit(self) -> None:
        Gtk.main_quit()


def main() -> int:
    win = DesktopChooser()
    win.show_all()
    Gtk.main()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
