# Shik

**A functional, dynamically-typed scripting language for shell automation — with a minimalist syntax designed to be written left-to-right in the terminal.**

[![version](https://img.shields.io/badge/version-0.7.1-blue)](https://github.com/pungy/shik/releases) [![license](https://img.shields.io/badge/license-MIT-green)](LICENSE) [![platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey)](https://github.com/pungy/shik/releases)

Shik is a scripting language built around one idea: the thought in your head and the code you type should be the same shape. Data flows left to right through function pipelines. Everything is function application — no operators, no special syntax, no imports. A full standard library for files, strings, lists, objects, and shell commands is available from the first line.

Shik is for people who write small automation scripts every few days — moving files, counting things, pulling shell output into structured data — and who are tired of fighting the tools instead of solving the problem.

![Demo](https://raw.githubusercontent.com/pungy/shik/main/shik-demo.gif)

---


## Try it

```bash
# Via cargo
cargo install shik

# macOS / Linux
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/pungy/shik/releases/latest/download/shik-installer.sh | sh

# Windows (PowerShell)
powershell -ExecutionPolicy ByPass -c "irm https://github.com/pungy/shik/releases/latest/download/shik-installer.ps1 | iex"
```

```bash
shik              # REPL — try typing help inside
shik script.shk   # run a file
```

## Learn

Read the [origin story](https://blog.pungy.me/articles/shik), which will give you a quick overview of the **Shik**.

Read the [Shik Book](https://blog.pungy.me/articles/shik-book/getting-started) to get a full understanding of the language.

## Roadmap

Current version: **v0.7.1**

Planned, roughly in priority order:

- Shebang support (`#!/usr/bin/env shik`)
- Object destructuring
- Regular expressions
- Multiple statements per line with `,`
- Networking
- Lambda shorthand (`#(- #1 #2)` instead of `fn [a b] - a b`)
- JSON parsing
- User-facing error handling (`try`/`catch` or similar)
- Threading

---

## Contributing

Shik is in active development.

Issues and PRs welcome at [github.com/pungy/shik](https://github.com/pungy/shik).

**License:** MIT
