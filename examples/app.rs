//! This is an example of using stored references to Symbols and Strings. The lifetime
//! is constrained to that of the entire app.

use gregtatum_symbol_table::{Symbol, SymbolTable};

fn main() {
    let symbol_table = SymbolTable::new();
    run_app(&symbol_table);
}

fn run_app(symbol_table: &SymbolTable) {
    let mut my_printer = MyPrinter::default();
    my_printer.add(symbol_table.get("hello"));
    my_printer.add(symbol_table.get("world"));
    my_printer.print_symbols();
    my_printer.print_strings();
}

#[derive(Default)]
struct MyPrinter<'app> {
    symbols: Vec<Symbol<'app>>,
    strings: Vec<&'app str>,
}

impl<'app> MyPrinter<'app> {
    fn add(&mut self, symbol: Symbol<'app>) {
        self.symbols.push(symbol);
        self.strings.push(symbol.str());
    }

    fn print_symbols(&self) {
        print!("Symbols: ");
        for symbol in &self.symbols {
            print!("{:?} ", symbol);
        }
        println!("");
    }

    fn print_strings(&self) {
        print!("Strings: ");
        for string in &self.strings {
            print!("{:?} ", string);
        }
        println!("");
    }
}
