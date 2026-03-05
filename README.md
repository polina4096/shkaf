# shkaf

Rust tool to scaffold any projects from templates.

## Usage

```sh
shkaf new <template> <path>  # Scaffold a new project from a template
shkaf list                   # List available templates
```

## Templates

Templates are stored in `~/.config/shkaf/templates/`. Each template is a directory with the following structure:

```
templates/
└── my-template/
    ├── template.toml
    └── files/
        ├── ...
        └── ...
```

Files inside `files/` are rendered using [Handlebars](https://handlebarsjs.com/) syntax. Binary files (images, fonts, compiled assets, etc.) are detected automatically by checking if the file content is valid UTF-8 and copied as-is. The detection involves reading the entire file into memory and attempting to decode it as UTF-8, so it's generally not advised to have lots of large binary files in templates.

The automatic detection vs manual using a suffix-based system is a trade-off between convenience and control. Usually you'd never need to have a lot of large binary files in templates, so in practice there's never an issue.

### Available variables

| Variable       | Description                                          |
| -------------- | ---------------------------------------------------- |
| `package_name` | The project name (last component of the output path) |

### template.toml

```toml
[template]
name = "My Template"
description = "A starter template"
author = "yourname"

[commands]
pre = [
  "git init",
]

post = [
  "cargo fmt",
]
```

Pre commands run before files are rendered, post commands run after. Commands are also rendered with Handlebars, so variables can also be used in them.

## License

Distributed under the The Unlicense.
