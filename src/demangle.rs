//
// written by everettjf
// email : everettjf@live.com
// created at 2022-01-02
//
use symbolic_common::Name;
use symbolic_demangle::{Demangle, DemangleOptions};

pub fn demangle_symbol(symbol: &str) -> String {
    let name = Name::from(symbol);
    let result = name.try_demangle(DemangleOptions::complete());
    result.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use symbolic_common::Language;

    #[test]
    fn demangle() {
        let name = Name::from("__ZN3std2io4Read11read_to_end17hb85a0f6802e14499E");
        assert_eq!(name.detect_language(), Language::Rust);
        assert_eq!(
            name.try_demangle(DemangleOptions::complete()),
            "std::io::Read::read_to_end"
        );
    }
}
