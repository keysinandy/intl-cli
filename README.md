### **install**

```bash
cargo install intl-cli
```

### **CLI Usage Guide**

#### **Overview**

This is a CLI tool for handling internationalization (i18n) text and translation. It supports the following commands:

1. **extract**: Extract i18n text from files.
2. **tencent-translate**: Translate text using the Tencent Translation service.

---

### **Commands and Arguments**

#### **`extract` Command**

Used to extract i18n text from specified files.

#### Notice

> The `extract` command only extracts the strings contained within the `$t` function, such as `$t('some words')` and `$t("after {count} days", {count: 1})`.

**Usage**:

```bash
intl-cli extract [OPTIONS]
```

**Options**:
| Short | Long | Description | Default |
|-------|---------------|------------------------------------------|-------------------------------------|
| `-o` | `--output` | Output file path | `output.json` |
| `-e` | `--excludes` | Glob patterns for files to exclude | `["**/node_modules/**", "**/.git/**"]` |
| `-i` | `--includes` | Glob patterns for files to include | `["*.{ts,tsx}"]` |
| `-d` | `--delete_unreached` | Delete unreached key-value pairs in output | None (default: `false`) |

**Example**:

```bash
intl-cli extract -o extracted.json -i "*.{ts,tsx}" -e "**/node_modules/**" --delete_unreached
```

---

#### **`tencent-translate` Command**

Translate text using the Tencent Translation service.

**Usage**:

```bash
intl-cli tencent-translate [OPTIONS]
```

**Options**:
| Short | Long | Description | Default |
|-------|---------------|------------------------------------------|----------------------------------|
| `-i` | `--input` | Input file path | `output.json` |
| `-o` | `--output` | Output file path | None |
| `-s` | `--source` | Source language | `zh` |
| `-t` | `--target` | Target language | `en` |
| `-p` | `--project_id`| Tencent Translation service Project ID | `0` |
| `-d` | `--secret_id` | Tencent Translation service Secret ID | None |
| `-k` | `--secret_key`| Tencent Translation service Secret Key | None |
| `-w` | `--write_all` | Translate and write all content from input to output | None (default: `false`) |

**Example**:

```bash
intl-cli tencent-translate -i input.json -o translated.json -s zh -t en --secret_id YOUR_SECRET_ID --secret_key YOUR_SECRET_KEY --write_all
```

---

### **Global Options**

| Short | Long        | Description              |
| ----- | ----------- | ------------------------ |
| `-h`  | `--help`    | Show help message        |
| `-V`  | `--version` | Show version information |

---

### **Example Commands**

1. Extract i18n text and save to `i18n.json`, ignoring `node_modules` and `.git` directories:

   ```bash
   intl extract -o i18n.json -i "*.{ts,tsx}" -e "**/node_modules/**" --delete_unreached
   ```

2. Use the Tencent Translation service to translate `i18n.json` into English and save to `translated.json`:
   ```bash
   intl-cli tencent-translate -i i18n.json -o translated.json --secret_id YOUR_SECRET_ID --secret_key YOUR_SECRET_KEY --write_all
   ```

---

### **Notes**

- When using the `tencent-translate` command, you must provide `secret_id` and `secret_key`, otherwise the Tencent Translation service cannot be invoked.
- The `write_all` option determines whether to translate and write all content from the input to the output.
