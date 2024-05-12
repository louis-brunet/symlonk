# Symlonk

## Symlink config file

<!-- TODO: executable name -->
- (Optional) Generate a json schema:
    ```bash
    cargo run -- create schema > path/to/my-generated-schema.json
    ```

- Write a configuration file
    ```toml
    #:schema path/to/my-generated-schema.json 

    [symlinks]
    "path/from/destination" = "path/from/source"

    [config]
    extends = "../root-symlonk.toml"
    source_dir = "."
    destination_dir = "~/path/to/symlinks"
    ```

<!-- ### Tokens -->
<!---->
<!-- ``` -->
<!-- NEWLINE := "\n" -->
<!-- EQ := "=" -->
<!-- PATH_SEPARATOR := ":" -->
<!-- HEADER_SEPARATOR := "---" -->
<!-- IDENTIFIER := (LETTER | "_") (LETTER | DIGIT | "_")* -->
<!-- FILE_NAME_CHAR := LETTER | DIGIT | "." | "_" | "-" -->
<!-- RELATIVE_PATH := FILE_NAME_CHAR (FILE_NAME_CHAR | "/")* -->
<!-- # ABSOLUTE_PATH := (FILE_NAME_CHAR | "/" )+ -->
<!-- EOF := EOF -->
<!-- `` -->
<!---->
<!-- - `"a"`: the literal string containing one 'a' character -->
<!-- - `P*`: repeat the pattern `P` 0 or more times -->
<!-- - `P+`: repeat the pattern `P` 1 or more times -->
<!-- - `(P)`: same as `P` -->
<!-- - `P | Q`: matches if either pattern `P` or `Q` matches -->
<!---->
<!-- ### Syntax -->
<!---->
<!-- ``` -->
<!-- Config := [Header NEWLINE] Declaration* -->
<!-- Header := HEADER_SEPARATOR NEWLINE HeaderSetting* HEADER_SEPARATOR NEWLINE -->
<!-- HeaderSetting := IDENTIFIER EQ IDENTIFIER NEWLINE -->
<!-- Declaration := RELATIVE_PATH PATH_SEPARATOR RELATIVE_PATH NEWLINE -->
<!-- ``` -->
<!---->
<!-- ### Header keys -->
<!---->
<!-- - `destination_dir`: prefix added to all created symlink paths (default: "~") -->
<!-- - `source_dir`: prefix added to symlink target paths (**required**) -->
