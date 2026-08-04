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
    fn demangle_rust() {
        let name = Name::from("__ZN3std2io4Read11read_to_end17hb85a0f6802e14499E");
        assert_eq!(name.detect_language(), Language::Rust);
        assert_eq!(
            name.try_demangle(DemangleOptions::complete()),
            "std::io::Read::read_to_end"
        );
    }

    #[test]
    fn demangle_cpp() {
        let name = Name::from("_ZN3foo3barEv");
        assert_eq!(name.detect_language(), Language::Cpp);
        assert_eq!(name.try_demangle(DemangleOptions::complete()), "foo::bar()");
    }

    #[test]
    fn demangle_non_mangled_passthrough() {
        // Plain (non-mangled) symbols should pass through unchanged.
        assert_eq!(demangle_symbol("main"), "main");
        assert_eq!(demangle_symbol("_fixture_target"), "_fixture_target");
    }
}
