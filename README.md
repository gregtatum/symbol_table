# gregtatum_symbol_table

A fast and efficient symbol table for making it easy to work cheapy with strings.

Stores a unique list of strings, so that strings can be operated upon via stable
indexes, which are stored in the [`Symbol`] type. This makes for cheap comparisons
and easy storage of references to strings. The strings are accessed as [`Symbol`]s
that have a `string() -> &str`.

```rs
use gregtatum_symbol_table::SymbolTable;

let symbol_table = SymbolTable::new();

// Insert strings into the SymbolTable.
let hello_symbol = symbol_table.get("hello");
let world_symbol = symbol_table.get("world");

// Strings can easily be accessed.
assert_eq!(hello_symbol.string(), String::from("hello"));
```

String can be accessed via various convenient traits:

```rs
let hello_string: String = hello_symbol.into();
assert_eq!(hello_string, "hello");

let hello_string: &str = hello_symbol.as_ref();
assert_eq!(hello_string, "hello");

let hello_string: String = format!("{}", hello_symbol);
assert_eq!(hello_string, "hello");

// String equality works across Symbols and strings.
assert_eq!(hello, "hello");
assert_eq!(world, "world");
```

There are various convenient ways to get a symbol back, and work with them.

```rs
// The symbol can be looked up via a HashMap, and string comparison will cheaply
// compare the indexes, and avoid full string comparison.
assert_eq!(symbol_table.get("hello"), hello_symbol);

let hello_world = symbol_table.get("hello world");
let hello_slice = hello_world.slice(0..5).unwrap();

// Slices can easily be created, but string comparison is now a full comparison.
assert_eq!(hello_slice, hello_symbol);

// But slices can be turned back into full Symbols for cheap comparisons.
assert_eq!(hello_slice.deslice(), hello_symbol);
```
