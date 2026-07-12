# 🐠 Ayu

A modern, statically typed programming language that transpiles to Luau.

## 🔷 `Hello, World!` Example

```ayu
extern fn print(msg: str)

fn main() {
    print("Hello, World!")
}
```

## 👀 Why Ayu?

Luau is a great language with many strengths, but Ayu explores a different approach. Its goal is to bring a more modern language design, stronger safety through static typing, and an ergonomic developer experience.

Rather than replacing Luau, Ayu builds on top of it, allowing developers to write expressive, maintainable code that transpiles to clean Luau.

## 🛠️ Current Status

Ayu is still in early development. The language design is still evolving, so the project is not accepting outside contributions yet.

## ✨ Language Features

### 🌐 Interoperability with Luau

Ayu uses `extern` declarations to interact with existing Luau globals.

```ayu
extern fn print(msg: str)
extern fn tostring as to_string(x: int) -> str

fn main() {
    print(to_string(1337))
}
```

### 🧱 Statically typed

Everything must have a type.

```ayu
extern fn print(msg: str)
extern fn tostring(num: int) -> str

fn add(a: int, b: int) -> int {
    return a + b
}

fn main() {
    let result: int = add(100, 200)
    
    print(tostring(result))
}
```

### 📜 Readable output

Ayu generates clean, formatted Luau output that is easy to inspect and debug.
