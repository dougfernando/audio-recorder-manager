# Icon Assets

This directory should contain SVG icon files from the Lucide icon set.

## Required Icons

Download these icons from: https://github.com/longbridge/gpui-component/tree/main/assets/icons

The following icons are currently used in the application:

- `bell.svg` - Used in header for notifications/status
- `book-open.svg` - Used in sidebar and history panel (empty state)
- `circle-check.svg` - Used for record action, success states, and confirmation
- `circle-x.svg` - Used for stop/cancel actions
- `eye.svg` - Used in sidebar for monitor panel
- `replace.svg` - Used in sidebar for recovery and settings reset
- `settings.svg` - Used in sidebar for settings panel
- `search.svg` - Used in recovery panel for scanning
- `check.svg` - Used in settings panel for save action

## Icon Naming Convention

Icon filenames must match the IconName enum in kebab-case:
- `IconName::CircleCheck` → `circle-check.svg`
- `IconName::BookOpen` → `book-open.svg`
- `IconName::Bell` → `bell.svg`

## Alternative: Download All Icons

You can download the entire icons folder from the gpui-component repository to have access to all available icons for future use.
