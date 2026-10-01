# Static Site Generator

An extremely barebones program to generate content for my website from Markdown files and HTML templates.

## Usage

To generate a site, use the `ssg build` command, followed by the `--source` argument, and optionally the `--output` argument, defaulting to `./public`.

### Source structure

The directory given by the `source` argument must include the `site.toml` file containing site-wide options and the `assets`, `content`, and `templates` directories.

### Site Templates

Templates are HTML files with syntax supported by the [minijinja](https://github.com/mitsuhiko/minijinja) crate. The only context variables available currently are `page.title`, `page.description` (both are set in the Markdown front matter), `page.content` (the Markdown outside of the front matter), and `site` (including all options that are set in `site.toml`).

### Site Content

The site generates a directory and unique `index.html` for each Markdown file, with the exception of the file `index.md`, which will not create a directory.
For example, if the directory given as the `source` argument had this structure:
```
content/
├─index.md
├─about.md
└─projects/
　　├─index.md
　　└─ssg.md
```
it would generate:
```
public/
├─index.html
├─about/
│　└─index.html
├─projects/
│　├─index.html
│　└─ssg/
│　　　└─index.html
└─assets/
　　└─...
```

Markdown files use the CommonMark standard and must include a header with this information:
```yaml
---
title: String
template: PathBuf in ../templates/
description: String
---
```

### Site Assets

The `assets` directory should include any file that is referenced by the templates or Markdown content and should get copied to the `assets` folder of the site.

## Serving the site

To serve a site from a generated site, use the `ssg serve` command. It takes the `root` argument, which should point to a folder containing valid HTML files, as well as the `address` argument, which takes any address and serves the `root` site on it.

### Preview subcommand

The `ssg preview` subcommand takes the `source`, `output`, and `address` arguments and builds then serves the given `source` site at `address`.
