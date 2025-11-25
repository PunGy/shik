# Shik Language

## Overview
Shik is a functional, dynamically-typed scripting language designed for shell automation with a minimalist syntax designed to be easily written in the terminal.

## Language Features
- Pipeline operator (`$>`) for function composition
- First-class functions and lambdas
- Pattern matching capabilities
- Rich standart library for working with system
- Lazy evaluation where appropriate

## Example
```shik
;; get all files in lang-dev with ext name .dk
;; Any string without whitespaces could be written with `:` symbol in the start
glob :../lang-dev/**/*.dk $>
  ;; Redirect it to function list mapping function.
  ;; Replace all :src/file.dk with ["src/file.dk", "src/file.ts"].
  map (fn [f] [f, ext.replace f :ts]) $>
  ;; Iterate each entry, and for each file copy if the file exists to "./"
  iterate (iterate (fn [f] copy.if-exists f :./))
```

