//! Compositor-Specific Shortcut Setup Instructions
//!
//! Provides setup instructions as strings for display in UI or terminal.
//! Each function returns instructions for a specific compositor.

use whis_core::Compositor;

/// Get setup instructions for the given compositor
pub fn get_instructions(compositor: &Compositor, shortcut: &str) -> String {
    match compositor {
        Compositor::Gnome => gnome_instructions(shortcut),
        Compositor::KdePlasma => kde_instructions(shortcut),
        Compositor::Sway => sway_instructions(shortcut),
        Compositor::Hyprland => hyprland_instructions(shortcut),
        Compositor::Wlroots => wlroots_instructions(shortcut),
        Compositor::Unknown(name) => unknown_instructions(name, shortcut),
        Compositor::Native | Compositor::X11 => native_instructions(shortcut),
    }
}

/// Get the config file path for the compositor (if applicable)
pub fn get_config_path(compositor: &Compositor) -> Option<&'static str> {
    match compositor {
        Compositor::Sway => Some("~/.config/sway/config"),
        Compositor::Hyprland => Some("~/.config/hypr/hyprland.conf"),
        _ => None,
    }
}

/// Get the command to copy (for the UI copy button)
pub fn get_config_snippet(compositor: &Compositor, shortcut: &str) -> Option<String> {
    match compositor {
        Compositor::Sway => Some(format!(
            "bindsym {} exec whis-desktop --toggle",
            shortcut.to_lowercase()
        )),
        Compositor::Hyprland => Some(format!(
            "bind = {}, exec, whis-desktop --toggle",
            shortcut.replace('+', ", ")
        )),
        _ => None,
    }
}

fn gnome_instructions(shortcut: &str) -> String {
    format!(
        r#"Configure in GNOME Settings:

1. Open Settings → Keyboard → Keyboard Shortcuts
2. Scroll to "Custom Shortcuts" and click +
3. Set:
   • Name: Whis Toggle Recording
   • Command: whis-desktop --toggle
   • Shortcut: {shortcut}

Or use the command line:
  gsettings set org.gnome.settings-daemon.plugins.media-keys custom-keybindings \
    "['/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/whis/']"
  dconf write /org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/whis/name "'Whis Toggle'"
  dconf write /org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/whis/command "'whis-desktop --toggle'"
  dconf write /org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/whis/binding "'<{shortcut}>'"
"#
    )
}

fn kde_instructions(_shortcut: &str) -> String {
    r#"Configure in KDE System Settings:

1. Open System Settings → Shortcuts
2. Click "Add New" → "Command or Script"
3. Set:
   • Name: Whis Toggle Recording
   • Command: whis-desktop --toggle
   • Trigger: Click and press your desired key combination
"#
    .to_string()
}

fn sway_instructions(shortcut: &str) -> String {
    let binding = shortcut.to_lowercase();
    format!(
        r#"Add to your Sway config (~/.config/sway/config):

bindsym {binding} exec whis-desktop --toggle

Then reload Sway:
  swaymsg reload
"#
    )
}

fn hyprland_instructions(shortcut: &str) -> String {
    let binding = shortcut.replace('+', ", ");
    format!(
        r#"Add to your Hyprland config (~/.config/hypr/hyprland.conf):

bind = {binding}, exec, whis-desktop --toggle

Then reload Hyprland:
  hyprctl reload
"#
    )
}

fn wlroots_instructions(_shortcut: &str) -> String {
    r#"Configure your compositor's keybindings to run:

whis-desktop --toggle

Check your compositor's documentation for keybinding syntax.
"#
    .to_string()
}

fn unknown_instructions(compositor_name: &str, _shortcut: &str) -> String {
    format!(
        r#"Configure {compositor_name} to run:

whis-desktop --toggle

Check your compositor's documentation for keybinding configuration.
"#
    )
}

fn native_instructions(_shortcut: &str) -> String {
    r#"Shortcuts should be automatically available.

If shortcuts are not working, try restarting the application.
If the issue persists, configure your system to run:

whis-desktop --toggle
"#
    .to_string()
}

