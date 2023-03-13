//! A fast and efficient symbol table for making it easy to work cheaply with strings.
//!
//! Stores a unique list of strings, so that strings can be operated upon via stable
//! indexes, which are stored in the [`Symbol`] type. This makes for cheap comparisons
//! and easy storage of references to strings. The strings are accessed as [`Symbol`]s
//! that have a [`fn str() -> &str`](struct.Symbol.html#method.str).

use std::fmt;
use std::marker::PhantomData;
use std::ops::Range;

use elsa::{FrozenMap, FrozenVec};
use fxhash::FxBuildHasher;

/// A cheap reference to a [`String`] in the [`SymbolTable`]. The only lifetime constraint
/// is that it must outlive the StringTable. This makes it easy to operate on strings
/// and store references to pieces of them.
/// ```
/// use gregtatum_symbol_table::SymbolTable;
///
/// let symbol_table = SymbolTable::new();
/// let hello = symbol_table.get("hello");
/// assert_eq!(hello, "hello");
///
/// let hello_str: &str = hello.str();
/// let hello_string: String = hello.to_string();
/// ```
#[derive(Copy, Clone)]
pub struct Symbol<'strings> {
    index: usize,
    range: Option<(u32, u32)>,
    symbol_table: &'strings SymbolTable<'strings>,
}

impl<'strings> Symbol<'strings> {
    fn new(symbol_table: &'strings SymbolTable, index: usize) -> Symbol<'strings> {
        Symbol {
            index,
            range: None,
            symbol_table,
        }
    }

    /// Returns a reference to a string. It will be bound by the lifetime of the
    /// [`SymbolTable`]. Symbols can also be substrings, which returns a reference
    /// to the substring in the [`SymbolTable`].
    ///
    /// ```
    /// use gregtatum_symbol_table::SymbolTable;
    ///
    /// let symbol_table = SymbolTable::new();
    /// let hello_symbol = symbol_table.get("hello");
    ///
    /// // The direct `.str()` way of accessing.
    /// assert_eq!(hello_symbol.str(), "hello");
    ///
    /// // The string can also be accessed via the following traits:
    ///
    /// let hello_string: String = hello_symbol.into();
    /// assert_eq!(hello_string, "hello");
    ///
    /// let hello_string: &str = hello_symbol.as_ref();
    /// assert_eq!(hello_string, "hello");
    ///
    /// let hello_string: String = format!("{}", hello_symbol);
    /// assert_eq!(hello_string, "hello");
    /// ```
    pub fn str(&self) -> &'strings str {
        let string = self.symbol_table.str(self.index);
        if let Some(ref range) = self.range {
            string
                .get(range.0 as usize..range.1 as usize)
                // This should always be valid, since "slice" checks that the string slice
                // is a valid one.
                .expect("Failed to get the range of a Symbol")
        } else {
            string
        }
    }

    /// Gets a slice of a string. This is a fast way to get substrings, but can
    /// incur penalties for string equality. A slice can be converted into a full
    /// symbol by running [`deslice`](struct.Symbol.html#method.deslice).
    ///
    /// ```
    /// use gregtatum_symbol_table::SymbolTable;
    ///
    /// let symbol_table = SymbolTable::new();
    /// let hello_world = symbol_table.get("hello world");
    ///
    /// // Slices can easily be created.
    /// let hello_slice = hello_world.slice(0..5).unwrap();
    /// assert_eq!(hello_slice, "hello");
    /// ```
    pub fn slice(&self, range: Range<usize>) -> Option<Symbol> {
        let range = match self.range {
            Some(ref existing_range) => {
                // Ensure the range is within the existing slice.
                let start = existing_range.0 as usize + range.start;
                let end = start + range.end;
                if end > existing_range.1 as usize {
                    return None;
                }
                start..end
            }
            None => range,
        };

        // Get the original string.
        let string = self.symbol_table.str(self.index);

        string.get(range.clone()).map(|_| Symbol {
            index: self.index,
            range: Some((range.start as u32, range.end as u32)),
            symbol_table: self.symbol_table,
        })
    }

    /// Turns a string slice into a full symbol. This ensures equality checks are
    /// simple index equality checks rather than full string equality checks.
    ///
    /// ```
    /// use gregtatum_symbol_table::SymbolTable;
    ///
    /// let symbol_table = SymbolTable::new();
    /// let hello = symbol_table.get("hello");
    /// let hello_world = symbol_table.get("hello world");
    ///
    /// // Slices can easily be created, but string comparison is now a full comparison.
    /// let hello_slice = hello_world.slice(0..5).unwrap();
    /// assert_eq!(hello_slice, hello);
    ///
    /// // But slices can be turned back into full Symbols for cheap comparisons.
    /// assert_eq!(hello_slice.deslice(), hello);
    /// ```
    pub fn deslice(self) -> Symbol<'strings> {
        if self.range.is_some() {
            self.symbol_table.get(self.str())
        } else {
            self
        }
    }
}

impl<'strings> PartialEq<String> for Symbol<'strings> {
    fn eq(&self, other: &String) -> bool {
        self.str() == other
    }
}

impl<'strings> PartialEq<&str> for Symbol<'strings> {
    fn eq(&self, other: &&str) -> bool {
        self.str() == *other
    }
}

/// Cheap string equality checks. Slices may invoke full string checking.
impl<'strings> PartialEq for Symbol<'strings> {
    fn eq(&self, other: &Self) -> bool {
        if self.index == other.index {
            if self.range == other.range {
                return true;
            }
            // Even though the indexes match, the subranges could point to equivalent
            // strings. This requires a full string comparison.
            return self.str() == other.str();
        }
        if self.range.is_none() && other.range.is_none() {
            // The is no slice range, and the indexes differ, so they must be different.
            return false;
        }
        // Do a full string comparison.
        self.str() == other.str()
    }
}

impl<'strings> fmt::Display for Symbol<'strings> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.str())
    }
}

impl<'strings> fmt::Debug for Symbol<'strings> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.str())
    }
}

impl<'strings> AsRef<str> for Symbol<'strings> {
    fn as_ref(&self) -> &str {
        self.str()
    }
}

impl<'strings> From<Symbol<'strings>> for String {
    fn from(other: Symbol<'strings>) -> Self {
        other.str().into()
    }
}

/// An index into the symbol vector.
pub type SymbolIndex = usize;

/// Stores a unique list of strings, so that strings can be operated upon via stable
/// indexes, which are stored in the [`Symbol`] type. This makes for cheap comparisons
/// and easy storage of references to strings. The strings are accessed as [`Symbol`]s
/// that have a [`fn str() -> &str`](struct.Symbol.html#method.str) method.
///
/// ```
/// use gregtatum_symbol_table::SymbolTable;
///
/// let symbol_table = SymbolTable::new();
/// let hello = symbol_table.get("hello");
/// let world = symbol_table.get("world");
///
/// // Symbols behave like strings.
/// assert_eq!(hello, "hello");
/// assert_eq!(world, "world");
///
/// // The symbol will looked up via a HashMap, and string comparison will cheaply
/// // compare the indexes.
/// assert_eq!(symbol_table.get("hello"), hello);
///
/// let hello_world = symbol_table.get("hello world");
/// let hello_slice = hello_world.slice(0..5).unwrap();
/// // Slices can easily be created, but string comparison is now a full comparison.
/// assert_eq!(hello_slice, hello);
///
/// // But slices can be turned back into full Symbols for cheap comparisons.
/// assert_eq!(hello_slice.deslice(), hello);
/// ```
#[derive(Default)]
pub struct SymbolTable<'strings> {
    symbols: FrozenVec<String>,
    indexes: FrozenMap<String, Box<SymbolIndex>, FxBuildHasher>,
    // Enforces the self lifetime.
    lifetime: PhantomData<&'strings ()>,
}

impl<'strings> SymbolTable<'strings> {
    /// Create a new SymbolTable.
    /// ```
    /// use gregtatum_symbol_table::SymbolTable;
    ///
    /// let symbol_table = SymbolTable::new();
    /// let hello = symbol_table.get("hello");
    /// let world = symbol_table.get("world");
    /// ```
    pub fn new() -> SymbolTable<'strings> {
        SymbolTable {
            ..Default::default()
        }
    }

    /// Interns a string into the [`SymbolTable`] if it doesn't yet exists and returns a
    /// [`Symbol`]. If the [`String`] has already been interned, then its index is looked
    /// up via a HashMap and a [`Symbol`] is returned.
    /// ```
    /// use gregtatum_symbol_table::SymbolTable;
    ///
    /// let symbol_table = SymbolTable::new();
    /// let hello = symbol_table.get("hello");
    /// let world = symbol_table.get("world");
    /// assert_eq!(hello, "hello");
    /// assert_eq!(world, "world");
    /// ```
    pub fn get<T: Into<String> + AsRef<str>>(&'strings self, string: T) -> Symbol<'strings> {
        if let Some(symbol) = self.maybe_get(string.as_ref()) {
            return symbol;
        }
        let index = self.len();
        let string: String = string.into();
        self.symbols.push(string.clone());
        self.indexes.insert(string, Box::new(index));
        Symbol::new(&self, index)
    }

    /// Gets an [`Symbol`] for a string only if it already exists.
    ///
    /// ```
    /// use gregtatum_symbol_table::SymbolTable;
    ///
    /// let symbol_table = SymbolTable::new();
    ///
    /// // Insert "hello"
    /// symbol_table.get("hello");
    ///
    /// let hello = symbol_table.maybe_get("hello");
    /// let world = symbol_table.maybe_get("world");
    /// assert_eq!(hello.unwrap(), "hello");
    /// assert_eq!(world, None);
    /// ```
    pub fn maybe_get<T: AsRef<str>>(&'strings self, string: T) -> Option<Symbol<'strings>> {
        self.indexes
            .get(string.as_ref())
            .map(|index| Symbol::new(&self, *index))
    }

    /// Check if the `SymbolTable` has a string.
    ///
    /// ```
    /// use gregtatum_symbol_table::SymbolTable;
    ///
    /// let symbol_table = SymbolTable::new();
    /// let hello = symbol_table.get("hello");
    /// assert!(symbol_table.has("hello"));
    /// assert!(!symbol_table.has("world"));
    /// ```
    pub fn has<T: AsRef<str>>(&'strings self, string: T) -> bool {
        self.maybe_get(string).is_some()
    }

    /// Get the amount of strings (not symbols) in the SymbolTable. Symbols can be
    /// created that are slices of strings. These are not counted as strings.
    ///
    /// ```
    /// use gregtatum_symbol_table::SymbolTable;
    ///
    /// let symbol_table = SymbolTable::new();
    /// assert_eq!(symbol_table.len(), 0);
    ///
    /// let hello = symbol_table.get("hello");
    /// let world = symbol_table.get("world");
    /// assert_eq!(symbol_table.len(), 2);
    ///
    /// // Symbols are not dupliclated.
    /// let hello = symbol_table.get("hello");
    /// assert_eq!(symbol_table.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Iterate through all of the strings. This does not iterate through [`Symbol`]s as
    /// the iterator could outlive the [`SymbolTable`].
    ///
    /// ```
    /// use gregtatum_symbol_table::SymbolTable;
    ///
    /// let symbol_table = SymbolTable::new();
    /// assert_eq!(symbol_table.len(), 0);
    ///
    /// let hello = symbol_table.get("hello");
    /// let world = symbol_table.get("world");
    /// let hello_world = symbol_table.get("hello world");
    ///
    /// let longest_string = symbol_table
    ///     .iter()
    ///     .max_by_key(|string| string.len())
    ///     .unwrap();
    ///
    /// // This is the longest string.
    /// assert_eq!(longest_string, "hello world");
    ///
    /// // A symbol can still be returned through a hash lookup.
    /// let hello_world = symbol_table.get(longest_string);
    /// assert_eq!(hello_world, "hello world");
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.symbols.iter()
    }

    fn str(&self, index: SymbolIndex) -> &str {
        match self.symbols.get(index) {
            Some(string) => string,
            None => "",
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get() {
        let symbol_table = SymbolTable::new();

        let hello = symbol_table.get("hello");
        let world = symbol_table.get("world");

        assert_eq!(format!("{:?}", hello), "\"hello\"");
        assert_eq!(format!("{:?}", world), "\"world\"");

        assert_eq!(format!("{}", hello), "hello");
        assert_eq!(format!("{}", world), "world");

        assert_eq!(hello, symbol_table.get("hello"));
        assert_ne!(hello, world);
    }

    #[test]
    fn test_slices() {
        let symbol_table = SymbolTable::new();
        let hello_word = symbol_table.get("hello world");

        assert_eq!(hello_word.slice(0..5).unwrap(), "hello");
        assert_eq!(hello_word.slice(6..11).unwrap(), "world");
        assert_eq!(hello_word.slice(12..16), None);
    }

    #[test]
    fn test_slice_equality() {
        let symbol_table = SymbolTable::new();
        let hello_world_1 = symbol_table.get("hello world 1");
        let hello_world_2 = symbol_table.get("hello world 2");

        assert_ne!(
            hello_world_1, hello_world_2,
            "The original strings are different."
        );

        let hello_1 = hello_world_1.slice(0..5).unwrap();
        let world_1 = hello_world_1.slice(6..11).unwrap();

        let hello_2 = hello_world_2.slice(0..5).unwrap();
        let world_2 = hello_world_2.slice(6..11).unwrap();

        assert_eq!(hello_1, hello_2, "Slices are full string equality checked");
        assert_eq!(world_1, world_2, "Slices are full string equality checked");

        assert_eq!(
            hello_1,
            symbol_table.get("hello"),
            "It works across symbols and sliced symbols"
        );
        assert_eq!(
            world_1,
            symbol_table.get("world"),
            "It works across symbols and sliced symbols"
        );

        let hellos = symbol_table.get("hello hello world");
        let hello_1 = hellos.slice(0..5).unwrap();
        let hello_2 = hellos.slice(6..11).unwrap();
        let world = hellos.slice(12..17).unwrap();

        assert_eq!(
            hello_1, hello_2,
            "Different slices into the same string are equality checked."
        );
        assert_ne!(
            hello_1, world,
            "Different slices into the same string are equality checked."
        );
    }

    #[test]
    fn test_deslicing() {
        let symbol_table = SymbolTable::new();
        let hello_world = symbol_table.get("hello world");
        let hello = hello_world.slice(0..5).unwrap();
        assert!(symbol_table.has("hello world"), "hello world is present");
        assert!(!symbol_table.has("hello"), "hello is not present");
        hello.deslice();
        assert!(symbol_table.has("hello"), "hello is now present");
    }

    #[test]
    fn test_multiple_slices() {
        let symbol_table = SymbolTable::new();
        let hello_world = symbol_table.get("hello world!");
        let world = hello_world.slice(6..11).unwrap();
        assert_eq!(world, "world");
        let orl = world.slice(1..3).unwrap();
        assert_eq!(orl, "orl");
        assert_eq!(
            world.slice(1..5),
            None,
            "The range can't go out of bounds into the original slice."
        );
    }

    #[test]
    fn test_traits() {
        fn as_str<T: AsRef<str>>(str: T, example: &str) {
            assert_eq!(str.as_ref(), example);
        }
        fn to_string<T: ToString>(str: T, example: &str) {
            assert_eq!(str.to_string(), example);
        }
        fn into_string<T: Into<String>>(str: T, example: &str) {
            assert_eq!(str.into(), example);
        }

        let symbol_table = SymbolTable::new();
        let hello = symbol_table.get("hello");
        as_str(hello, "hello");
        to_string(hello, "hello");
        into_string(hello, "hello");

        let _hello_str: String = hello.into();
    }
}
