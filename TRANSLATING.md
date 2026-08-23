# Translating GNOME Paths

GNOME Paths uses the standard GNU gettext translation system. Translations are maintained in the `po/` directory using `.po` (Portable Object) files.

---

## Directory Structure

```text
po/
├── LINGUAS            # List of active language codes (one per line)
├── POTFILES.in        # List of source files containing translatable strings
├── gnome-paths.pot    # Base translation template
├── en.po              # English translation catalog
└── pt_BR.po           # Brazilian Portuguese translation catalog
```

---

## Adding a New Language

1. **Pick the Language Code**:
   Identify the ISO 639 language code (e.g., `es` for Spanish, `de` for German, `fr` for French, `it` for Italian).

2. **Register the Language**:
   Open `po/LINGUAS` and add your language code on a new line:
   ```text
   en
   pt_BR
   es
   ```

3. **Initialize the PO File**:
   Copy the base template `po/gnome-paths.pot` to `po/<language_code>.po`:
   ```bash
   cp po/gnome-paths.pot po/es.po
   ```

4. **Translate the Strings**:
   Open `po/<language_code>.po` in a text editor or a translation tool like **Poedit** or **GNOME Translation Editor (gtranslator)**:
   ```po
   msgid "Select and Transform"
   msgstr "Seleccionar y transformar"
   ```

---

## Translation Guidelines

- **Placeholders**: Keep formatting placeholders like `{}`, `%s`, and `<b>...</b>` exactly as they appear in the original `msgid`.
- **Keyboard Shortcuts**: If a string contains shortcut hints like `(Ctrl+S)`, preserve the hotkey format.
- **Pango Markup**: Do not use raw unescaped `&` characters in XML markup strings; use `&amp;` or natural language conjunctions (e.g., "and" / "e" / "y").

---

## Testing Translations Locally

You can test your translations by running the application with the `LANGUAGE` environment variable:

```bash
# Test with Spanish
LANGUAGE=es cargo run

# Test with Brazilian Portuguese
LANGUAGE=pt_BR cargo run

# Test with English
LANGUAGE=en cargo run
```

---

## Updating Translation Files

When new strings are added to the codebase, regenerate the `.pot` template and merge with existing catalogs:

```bash
# Automated string extraction and catalog generation
python3 scratch/generate_po.py
```

Or using standard GNU gettext utilities:
```bash
# Update template
xgettext --from-code=UTF-8 --keyword=gettext -f po/POTFILES.in -o po/gnome-paths.pot

# Merge into existing translation
msgmerge --update po/es.po po/gnome-paths.pot
```

---

## Submitting Translations

1. Fork the repository on GitLab: `https://gitlab.gnome.org/lewisHeart/gnome-paths` (or GitHub).
2. Commit your new or updated `po/<language_code>.po` file.
3. Open a Merge Request / Pull Request.
