# macOS display and interaction model

Widgitron has three surfaces:

| Surface | Purpose | How it opens on macOS | Closing behavior |
| --- | --- | --- | --- |
| Dashboard | Configure services, inspect data, control visibility | Application launch or menu bar **Dashboard** | The app remains in the menu bar |
| Sidebar | Read several modules together | Dashboard **Open Sidebar** or menu bar **Sidebar** | Close button; **Pin Display** also opens it on the next launch |
| Independent widget | Keep one module in its own movable window | Dashboard **Show Widget** | Widget close button or **Hide All Widgets** |

## Startup and state

- macOS starts with the dashboard and no independent widgets unless the user
  explicitly enabled a widget after migration. Service enablement and widget
  visibility are separate settings: a service may collect data while its
  independent window is hidden.
- The one-time migration recognizes the old state where all four widget windows
  were automatically marked visible. It clears that state, preserving widgets
  explicitly pinned above other windows. Other visibility selections remain.
- Default macOS widget assignments use opaque theme presets. Legacy transparent
  assignments are replaced once; custom nontransparent assignments remain.
- The light sidebar uses high-opacity surfaces and dark widget text. The
  sidebar's content uses the standard widget palette with text colors adapted
  for the light background.
- `active_widgets` controls independent windows. `sidebar_widgets` controls
  which modules appear in the sidebar. The sidebar is useful even when no
  independent widget is open.

## macOS controls

- The menu bar icon presents native actions for the dashboard, sidebar, hiding
  independent widgets, and quitting. It remains an entry point when the main
  window is closed.
- Independent widgets start as normal movable windows. Their move lock, keep
  above other windows, desktop fixation, and hide buttons are visible without
  hover. The lower right corner provides a visible resize handle. The dashboard
  also has a desktop fixation control, so a widget can be returned to normal
  window order even when another app covers it.
- Desktop-fixed widgets use an AppKit window below Finder desktop icons and
  stay on all Spaces. They are intended to remain visible during macOS Show
  Desktop. This setting persists per widget, independently of the move lock.
  Turning on always-on-top returns that widget to floating mode.
- The portable macOS sidebar stays open when focus moves to another window;
  it closes explicitly. **Pin Display** controls whether it also opens on the
  next launch. Its edge is selected in Settings. Edge reveal and drag-to-dock
  are Windows features.
- The interface language defaults to Simplified Chinese on macOS and can be
  switched to English in Settings. The choice is stored in `app_config.json`.

## Boundaries

Desktop fixation uses the existing live window at desktop level. It is not a
WidgetKit extension and does not appear in the macOS widget gallery. The
update installer opens a disk image for manual installation. Widget and
sidebar windows still use the existing custom frameless design.
