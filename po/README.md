# GNOME Paths Translations (i18n)

This directory contains GNU gettext translations for **GNOME Paths**, configured for seamless integration with **Weblate** and GNOME translation workflows.

## Supported Languages
- `en` - English (Base / Source)
- `pt_BR` - Portuguese (Brazil)

## Structure
- `POTFILES.in` - List of source files scanned for translatable strings.
- `gnome-paths.pot` - The master Portable Object Template (POT) file.
- `pt_BR.po` - Brazilian Portuguese translation catalog.
- `en.po` - English translation catalog.
- `LINGUAS` - List of active languages.

## Weblate Integration
Weblate can directly sync with this repository. Whenever new strings are added:
1. Update `po/gnome-paths.pot`.
2. Merge POT into PO files (`msgmerge -U po/pt_BR.po po/gnome-paths.pot`).
3. Translators submit translations via Weblate or pull requests.
