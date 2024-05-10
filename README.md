# Symlonk

## Symlink config file

### Tokens

```
NEWLINE := "\n"
EQ := "="
PATH_SEPARATOR := ":"
HEADER_SEPARATOR := "---"
IDENTIFIER := (LETTER | "_") (LETTER | DIGIT | "_")*
FILE_NAME_CHAR := LETTER | DIGIT | "." | "_" | "-"
RELATIVE_PATH := FILE_NAME_CHAR (FILE_NAME_CHAR | "/")*
# ABSOLUTE_PATH := (FILE_NAME_CHAR | "/" )+
EOF := EOF
``

- `"a"`: the literal string containing one 'a' character
- `P*`: repeat the pattern `P` 0 or more times
- `P+`: repeat the pattern `P` 1 or more times
- `(P)`: same as `P`
- `P | Q`: matches if either pattern `P` or `Q` matches

### Syntax

```
Config := [Header NEWLINE] Declaration*
Header := HEADER_SEPARATOR NEWLINE HeaderSetting* HEADER_SEPARATOR NEWLINE
HeaderSetting := IDENTIFIER EQ IDENTIFIER NEWLINE
Declaration := RELATIVE_PATH PATH_SEPARATOR RELATIVE_PATH NEWLINE
```

### Header keys

- `destination_dir`: prefix added to all created symlink paths (default: "~")
- `source_dir`: prefix added to symlink target paths (**required**)
