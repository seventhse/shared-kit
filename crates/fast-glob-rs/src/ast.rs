//! # Glob Pattern AST and Syntax Reference
//!
//! This module defines the Abstract Syntax Tree (AST) for parsed glob patterns
//! and provides a comprehensive reference for all supported syntax.
//!
//! ## Glob Pattern Syntax Reference
//!
//! The functionality is powered by industry-standard libraries like `micromatch`,
//! covering everything from basic wildcards to advanced extended globbing (extglob).
//!
//! ### Basic Wildcards
//!
//! These are the most fundamental and common parts of any glob implementation.
//!
//! | Syntax | Name | Description | Example |
//! | :--- | :--- | :--- | :--- |
//! | `*` | Asterisk | Matches **zero or more** characters, but **not** path separators (`/`). | `src/*.js` matches `src/index.js`, not `src/api/v1.js`. |
//! | `?` | Question Mark | Matches a **single** character, but **not** path separators (`/`). | `src/?.js` matches `src/a.js`, not `src/ab.js`. |
//! | `**` | Globstar | Matches **zero or more** directory levels. Must be a complete path segment. | `src/**/*.js` matches `src/index.js` and `src/api/v1.js`. |
//!
//! ### Character Classes
//!
//! Used to match against a set of possible characters.
//!
//! | Syntax | Name | Description | Example |
//! | :--- | :--- | :--- | :--- |
//! | `[...]` | Character Set | Matches any **single** character within the brackets. | `[abc].js` matches `a.js`, `b.js`, `c.js`. |
//! | `[a-z]` | Character Range | Matches any **single** character within the specified range. | `[0-9].log` matches `1.log`. |
//! | `[!...]` or `[^...]` | Negated Set | Matches any **single** character **not** in the brackets. | `[!abc].js` matches `d.js`, not `a.js`. |
//!
//! ### Grouping and Alternation
//!
//! Used to combine multiple patterns.
//!
//! | Syntax | Name | Description | Example |
//! | :--- | :--- | :--- | :--- |
//! | `{a,b}` | Brace Expansion | **Pre-processing step**: Expands a pattern into multiple distinct patterns before matching begins. | `src/{a,b}.js` is expanded into `src/a.js` and `src/b.js`. |
//! | `@(a\|b)` | Exact Match (Extglob) | Matches **exactly one** of the given patterns. | `src/@(app\|lib).js` matches `src/app.js` or `src/lib.js`. |
//!
//! ### Extended Globbing (Extglob)
//!
//! Provides powerful, regular-expression-like capabilities.
//!
//! | Syntax | Name | Description | Example |
//! | :--- | :--- | :--- | :--- |
//! | `?(pattern)` | Zero or One | Matches the `pattern` **zero or one** time (optional). | `a?(b)c` matches `ac` and `abc`. |
//! | `*(pattern)` | Zero or More | Matches the `pattern` **zero or more** times. | `a*(b)c` matches `ac`, `abc`, `abbc`, etc. |
//! | `+(pattern)` | One or More | Matches the `pattern` **one or more** times. | `a+(b)c` matches `abc`, `abbc`, but not `ac`. |
//! | `!(pattern)` | Negative Match | Matches any string segment that does **not** match the `pattern`. | `src/!(vendor)/*.js` matches `src/app/main.js`, not `src/vendor/jquery.js`. |
//!
//! ### Pattern Modifiers
//!
//! Used to alter the behavior of an entire pattern.
//!
//! | Syntax | Name | Description | Example |
//! | :--- | :--- | :--- | :--- |
//! | `!` (prefix) | Negation Pattern | If a pattern starts with `!`, it **excludes** all matching paths from the final result set. | `['**/*.js', '!**/vendor/**']` |
//! | `\` | Escape Character | Treats the following special character (e.g., `*`, `?`) as a literal character. | `a\*.js` matches the literal filename `a*.js`. |
//!
//! ### Important Notes
//!
//! 1.  **Path Separators**: For cross-platform compatibility, **always** use a forward slash (`/`) as the path separator in patterns, even on Windows. The library handles path normalization internally.
//! 2.  **Brace Expansion vs. Extglob**: Note the difference between `{a,b}` and `@(a|b)`. `{a,b}` is a string replacement step done before matching, while `@(a|b)` is an "OR" logic applied during the matching process.
//! 3.  **Negation Pattern vs. Negative Match**: The scope of `!` (prefix) and `!(pattern)` is different.
//!     *   `!pattern` (prefix) applies to the entire path and filters the final results.
//!     *   `!(pattern)` applies to a path *segment* as part of the matching logic.
//!
//!

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterClasses {
    ast: Vec<AstNode>,
    /// [!...] or [^...] is negative
    negative: bool,
    len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstNode {
    Literal(char),

    /// Basic syntax
    ///  *
    Star,
    // **
    GlobStart,
    // ?
    AnySea,

    // Character classes
    CharacterClasses(CharacterClasses),

    //
}
