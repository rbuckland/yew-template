<h1 align="center">Yew-Template</h1>

<p align="center">
    <a href="https://crates.io/crates/yew-template"><img alt="Crates.io" src="https://img.shields.io/crates/v/yew-template"></a>
    <img alt="GitHub last commit" src="https://img.shields.io/github/last-commit/INSAgenda/yew-template?color=%23347d39" alt="last commit badge">
    <img alt="GitHub" src="https://img.shields.io/github/license/INSAgenda/yew-template">
    <img alt="GitHub closed issues" src="https://img.shields.io/github/issues-closed-raw/INSAgenda/yew-template">
    <img alt="docs.rs" src="https://img.shields.io/docsrs/yew-template">
</p>

<p align="center">A crate for using separate HTML files as <a href="https://yew.rs/">Yew</a> objects, with support for seamless localization.</p>

## Getting Started

### Hello World

```hbs
<div>
    <p>Hello {{name}}!</p>
</div>
```

```rust
use yew_template::template_html;

let html = template_html!("templates/hello.html", name="World");
```

The code above will actually compile to the following code:

```rust
let html = yew::html! {
    <div>
        <p>{"Hello World!"}</p>
    </div>
};
```

## Usage

- [Variables](#variables)
- [Attributes](#attributes)
- [Struct fields](#struct-fields)
- [Expressions](#expressions)
- [Example: Yew callbacks](#example-with-yew-callbacks)
- [Components](#components)
- [Optional variables](#optional-variables)
- [Optional elements](#optional-elements)
- [Iterators](#iterators)
- [Template inheritance](#template-inheritance)
- [Minimizing bloat](#minimizing-bloat)
- [Virtual elements](#virtual-elements)
- [Localization](#localization)
- [Config](#config)
- [Features](#features)
- [Security Notes](#security-notes)

### Variables

```rust
use yew_template::template_html;

let html = template_html!("templates/hello.html", name="World");
```

Pass with different variable name:

```rust
use yew_template::template_html;

let other_name = "Yew";
let html = template_html!("templates/hello.html", name=other_name);
```

### Attributes

Attributes support `format!`-like syntax with multiple interpolations:

```hbs
<div style="color: {{text_color}}; background: {{bg_color}};"></div>
```

```rust,ignore
use yew_template::template_html;

let html = template_html!("template.html", text_color="blue", bg_color="white");
```

### Struct fields

Pass a struct and access its fields directly in templates:

```hbs
<p>{{person.first_name}} {{person.last_name}}</p>
```

```rust,ignore
use yew_template::template_html;

struct Person { first_name: String, last_name: String }
let person = Person { first_name: "John".to_string(), last_name: "Doe".to_string() };
let html = template_html!("template.html", person);
```

### Expressions

```rust
use yew_template::template_html;

let name_reversed = String::from("dlroW");
let html = template_html!(
    "templates/hello.html",
    name = {
        let mut name = name_reversed.into_bytes();
        name.reverse();
        let name = String::from_utf8(name).unwrap();
        name
    }
);
```

Which will also display `Hello World!` as the Yew-code output is as follows:

```rust
let name_reversed = String::from("dlroW");
let html = yew::html! {
    <div>
        <p>
            {"Hello "}{{
            let mut name = name_reversed.into_bytes();
            name.reverse();
            let name = String::from_utf8(name).unwrap();
            name
            }}{"!"}
        </p>
    </div>
};
```

Note that the curly brackets around expressions are required for expressions.

### Example with Yew callbacks

```hbs
<div onclick={{onclick}}>
   <p>Hello {{name}}!</p>
</div>
```

```rust,ignore
let link = ctx.link();
let html = template_html!(
    "templates/hello.html",
    name="World",
    onclick={link.callback(|_| Msg::AddOne)}
);
```

### Components

While yew-template can be used only with raw HTML, it is also possible to use Yew components in your templates.
These do not follow the same syntax as in Yew's html macro, and need to be explicitly marked as components using the `comp` or `component` tag name.

```hbs
<comp name="SearchBar"/>
<!-- Which is equivalent to -->
<component name="SearchBar"/>
<!-- Or even -->
<Component name="SearchBar"/>
```

As you can see, the rust identifier for the component is passed as an attribute.

Other attributes and even children can be passed the regular way provided that your component supports them.

```hbs
<comp name="SearchBar" placeholder="Search..." onclick={{onclick}}>
    <span>child 1</span>
    <span>child 2</span>
</comp>
```

### Optional variables

Optional variables are marked with an `opt_` prefix or an `_opt` suffix, at your option.
Their value is expected to be an `Option<T>`.

Optional variables work with optional html elements. Mark an element with the `opt` attribute to make it optional. An optional element will only be rendered if *ALL* the optional variables it contains are `Some`. Note that variables contained by smaller optional elements are excluded from this requirement.

```hbs
<div>
    <p>Hello {{name}}!</p>
    <div opt>
        <h2>Age</h2>
        <p>You are {{opt_age}} years old!</p>
    </div>
</div>
```

In the example above, the `div` block will not be shown if `opt_age` is `None`.

Let's see how optional elements can be nested.

```hbs
<div>
    <p>Hello {{name}}!</p>
    <div opt>
        <h2>Age</h2>
        <p>You are {{opt_age}} years old!</p>
        <p opt>And you are born in {{opt_birth_city}}.</p>
    </div>
</div>
```

Here, both `opt_age` and `opt_birth_city` are optional. `opt_age` would be displayed even if `opt_birth_city` is `None`. However, if `opt_age` is `None`, `opt_birth_city` will not be displayed regardless of its value.

From the Rust side, there is no usage difference. Note that curly brackets are required (for now).

```rust
use yew_template::template_html;

let opt_age: Option<u8> = Some(20);
let opt_birth_city: Option<String> = None;
let html = template_html!(
    "templates/opt.html",
    name="John",
    opt_age,
    opt_birth_city
);
```

In the generated Yew code, `if let` expressions are used. As a result, optional variables based on expressions behave differently as they are only evaluated once for each optional element using them.

### Optional elements

Sometimes optional variables are not suitable for making an element optional. You might need a logic that is more complex than just checking if a variable is `Some` or `None`. In this case, you can use optional elements.

Elements can be given a `present-if` attribute. The value will be evaluated at runtime as a boolean expression. If the expression is `true`, the element will be rendered. Otherwise, it will be skipped.

```hbs
<div present-if={{condition}}>
    <p>1+1 = 3</p>
</div>
<div present-if=!{{condition}}> <!-- Negation is supported -->
    <p>1+1 != 3</p>
</div>
```

```rust
use yew_template::template_html;

let html = template_html!("templates/present_if.html", condition={ 1+1==3 });
```

### Iterators

Iterators use semantic variable names following patterns familiar from frameworks like Angular and Vue.
The syntax is placed on an outer parent HTML tag and all elements are duplicated; syntactically this looks like a for loop.

```hbs
<div>
    <h2>People:</h2>
    <ul iter.person={people}>
        <li>{{person.first_name}} {{person.last_name}}</li>
    </ul>
</div>
```

```rust
use yew_template::template_html;

#[derive(Clone, Copy)]
struct Person {
    first_name: &'static str,
    last_name: &'static str,
}

let people = vec![
    Person { first_name: "Alice", last_name: "Smith" },
    Person { first_name: "Bob", last_name: "Jones" },
];

let html = template_html!("templates/people_iter.html", people={people.iter()}, ...);
```

#### Field Access in Iterators

You can access fields of structs in iterators using dot notation:

```hbs
<ul>
    <li iter.item={items}>ID: {{item.id}}, Value: {{item.value}}</li>
</ul>
```

**Note**: When using field access with iterators, be mindful of Rust's ownership rules. Fields that implement `Copy` (like `i32`, `bool`, etc.) work seamlessly. For owned types like `String`, consider using references or ensuring proper ownership handling.

#### Loop Variables

Inside an `iter.*` block, a set of special `loop.*` variables are available:

| Variable | Description |
|---|---|
| `loop.index` | Current iteration (0-indexed) |
| `loop.index1` | Current iteration (1-indexed) |
| `loop.first` | `true` on the first iteration |
| `loop.last` | `true` on the last iteration |
| `loop.length` | Total number of items |
| `loop.depth` | Nesting depth, 0-indexed (0 for outermost, increments in nested loops) |
| `loop.depth1` | Nesting depth, 1-indexed (1 for outermost, increments in nested loops) |
| `loop.previtem` | Item from the previous iteration (`Option<T>`), or `None` on the first |
| `loop.nextitem` | Item from the next iteration (`Option<T>`), or `None` on the last |

```hbs
<ul iter.item={items}>
    <li>
        <span present-if={{loop.first}}>[first] </span>
        {{loop.index1}}/{{loop.length}}: {{item}}
        <span present-if={{loop.last}}> [last]</span>
    </li>
</ul>
```

`loop.first` and `loop.last` are booleans and work naturally with `present-if`. `loop.index` and `loop.index1` are `usize` values.

#### Nested Loops with Custom Aliases

Use `loop_var` to give each loop a distinct alias when nesting. This lets inner loops access outer loop variables:

```hbs
<ul iter.outer={outers} loop_var="outer">
  <li>Outer {{outer.index1}}
    <ul iter.inner={inners} loop_var="inner">
      <li>Inner {{inner.index1}} of {{inner.length}}, outer is {{outer.index1}}</li>
    </ul>
  </li>
</ul>
```

```rust,ignore
use yew_template::template_html;

let html = template_html!("nested.html", outers={...}, inners={...}, ...);
```

Without `loop_var`, the default alias is `"loop"`. Each custom alias maintains its own reference stack across nesting levels.

### Simplified Design

Embedding code in another language (e.g., SQL in Java, Bash in Python) is often considered a high-maintenance anti-pattern because it complicates syntax highlighting, hinders debugging, increases security risks (like SQL injection), and makes testing difficult. While useful for rapid prototyping or legacy code, it introduces technical debt, reduces readability, and hampers static analysis.

This is the reason to use `yew-template`, to reduce complexity.

**Key Reasons It Is an Anti-Pattern**
- **Increased Complexity & Reduced Readability** Mixing languages forces developers to context-switch, making the code harder to read and maintain.
- **Syntax and Debugging Issues** Embedded code often lacks proper IDE support, such as syntax highlighting, linting, and autocomplete, making errors harder to catch.
- **Security Vulnerabilities** Embedded strings (like SQL) are prone to injection attacks if not properly sanitized.
- **Testing Difficulties** It is challenging to unit test code that is embedded within a string in another language.

### Template Inheritance

Template inheritance lets you define a base layout once and override specific regions in child templates — the same pattern as Jinja's `{% extends %}` / `{% block %}`.

**Base template** — define named `<block>` regions with optional default content:

```hbs
<!-- templates/layouts/base.html -->
<div class="page">
  <header><block name="header"><h1>Default Title</h1></block></header>
  <main><block name="content"><p>No content provided.</p></block></main>
  <footer><block name="footer"><p>Default Footer</p></block></footer>
</div>
```

**Child template** — declare the parent with `<extends src="…"/>` and supply `<block>` overrides:

```hbs
<!-- templates/layouts/child.html -->
<extends src="templates/layouts/base.html"/>
<block name="header"><h1>{{title}}</h1></block>
<block name="content"><p>Hello, {{name}}!</p></block>
<!-- footer not overridden → renders base default -->
```

```rust
use yew_template::template_html;

let title = "My Page";
let name  = "World";
let html = template_html!("templates/layouts/child.html", title, name, ...);
```

Rules:
- `<extends src="path"/>` must be self-closing.  The path follows the same convention as the first argument to `template_html!` (relative to `template_directory`).
- Only `<block>` elements at the root level of a child template are considered; any other content outside a `<block>` is ignored.
- Blocks not overridden by the child keep the base default content.
- Inheritance chains are supported: a parent template can itself extend another template.
- A base template can also be used directly with `template_html!` — blocks then render their default content transparently.

### Minimizing bloat

The whole point of using this crate is making your code more readable than when using Yew directly. However, you will still find yourself writing lines of code that do not carry that much meaning. We already saw that `variable_ident=variable_ident` can be shortened to `variable_ident`. But it could even be completely omitted! Add `...` at the end of your macro call to tell that undefined variables should be retrieved from local variables with the same name. Taking the "Hello world" example:

```hbs
<div>
    <p>Hello {{name}}!</p>
</div>
```

```rust
use yew_template::template_html;

let name = "World";
let html = template_html!("templates/hello.html", ...);
```

This behavior is disabled by default because missing variables are often mistakes. If you want to enable it without have to add `...` to every macro call, please set `auto_default` to true in your [config](#config).

### Virtual elements

Yew-template requires adding attributes like `iter.var={...}`, `opt`, or `present-if` to HTML elements. In rare cases where no suitable element exists and adding a wrapper would break your CSS, use virtual elements. The `<virtual>` tag is removed from the final HTML:

```hbs
<virtual opt>
    {{opt_name}}
</virtual>
```

```rust
use yew_template::template_html;

let opt_name = Some("John".to_string());
let html = template_html!("templates/virtual.html", opt_name);
```

On Yew side, this will be seen as:

```rust
let opt_name = Some("John".to_string());
let html = yew::html! {
   <>
      if let Some(opt_name) = opt_name { {opt_name} }
  </>
};
```

And Yew will produce the following HTML:

```html
John
```

### Localization

Yew-template supports localization. It is able to extract localization data from `.po` files and automatically embed them in the generated code. Enabling this feature is as simple as putting `.po` files in a directory.

The `i18n` cargo feature needs to be enabled (it is enabled by default).

By default, the locale directory is set to `locales`. You can change this by setting `locale_directory` in your [config](#config). Yew template will automatically generate an up-to-date `.pot` file in this directory. Use it in your translation software as a template to generate `.po` files.

When done translating, put your `.po` files in the locale directory. Support for the added locales will automatically be enabled.

In order to select the locale to be rendered at runtime, you need to pass a `locale` variable to template-html macro calls. This variable will be matched against the filenames of the `.po` files in the locale directory (exluding the `.po` extension). If no match is found, the string will be left as it appears in your template.

Instead of using a `locale` variable, you can decide to evaluate any Rust expression. See the `locale_code` option in the [config](#config) section.

Yew-template prevents code injection from localized strings. This is done by escaping double quotes and backslashes. It is **SAFE** to delegate translation to unknown peers. However, these strings can include variable references, which could break compilation if referenced variables are not defined. Yew-template will take care of this issue in the future.

## Config

You can specify various settings in a `yew-template.toml` file at the crate root.
This requires the `config` cargo feature to be enabled (it is enabled by default).

This is the default configuration:

```toml
# Whether to attempt to capture local variables instead of aborting when arguments required by the template are missing.
auto_default = false

# Where to look for templates (relative to crate root)
template_directory = './'

# Where to look for locales (relative to crate root)
locale_directory = './locales/'

# Rust code to evaluate as locale. Should evaluate to a &str.
# If will be inserted in generated code like this: `match locale_code {`.
locale_code = 'locale.as_str()'

# Two strings marking the beginning and end of a variable in a template.
variable_separator = ["{{", "}}"]

```

## Features

All features are enabled by default. There currently two features:
- [`config`](#config): Allows you to use `yew-template.toml` settings
- [`i18n`](#localization): Enables support for localization

## Security Notes

- It is safe to display all kinds of strings. They will be escaped appropriately, preventing both HTML and Rust injection.
- Localized strings are harmless in the generated code, but they could break compilation.
- Do not use untrusted template files.
- Do not use untrusted `yew-template.toml` files.

License: MIT
