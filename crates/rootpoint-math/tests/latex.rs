//! The closed LaTeX subset: what it reads, what it refuses, and its round trip.

use proptest::prelude::*;
use rootpoint_math::{
    Accent, Atom, Class, Delim, Environment, Formula, List, MathVariant, Space, command,
};

fn parse(text: &str) -> Formula {
    Formula::parse(text).unwrap_or_else(|e| panic!("{text}: {e}"))
}

/// The five reference formulas the layout is judged on, as `REFERENCE` of
/// `path@rootpoint-math@tests/layout.rs` holds them.
pub const REFERENCE: [&str; 5] = [
    r"\frac{\partial f_i}{\partial t} + \mathbf{c}_i \cdot \nabla_{\mathbf{x}} f_i = \Omega(f_i)",
    r"\int_{-\infty}^{\infty} P_n(x)\,\bar{\omega}(x)\,\mathrm{d}x = \sum_{i=1}^{N} w_i P_n(x_i)",
    r"f(x) &= (x+1)^2 \\ &= x^2 + 2x + 1 \\ &\leq \sqrt{\frac{a^2+b^2}{2}}",
    r"R(\theta) = \begin{pmatrix} \cos\theta & -\sin\theta \\ \sin\theta & \cos\theta \end{pmatrix}",
    r"|x| = \begin{cases} x & \text{if } x \geq 0 \\ -x & \text{otherwise} \end{cases}",
];

#[test]
fn the_reference_formulas_parse_with_no_unknown_command() {
    for text in REFERENCE {
        let f = parse(text);
        assert!(!f.has_unknown(), "{text}");
        println!("{}", f.to_latex());
        assert_eq!(parse(&f.to_latex()), f, "{text}");
    }
}

#[test]
fn an_unknown_command_survives_and_takes_no_argument() {
    let f = parse(r"\foo{x}");
    assert_eq!(
        f.rows,
        vec![vec![vec![
            Atom::Unknown("foo".to_owned()),
            Atom::Group(vec![sym('x')]),
        ]]]
    );
    assert_eq!(f.to_latex(), r"\foo{x}");
    assert!(f.has_unknown());
}

#[test]
fn a_second_script_on_one_nucleus_does_not_parse() {
    assert!(Formula::parse("x^a^b").is_err());
    assert!(Formula::parse("x_a_b").is_err());
    assert!(Formula::parse("x_a^b").is_ok());
    assert!(Formula::parse("x^b_a").is_ok());
}

#[test]
fn what_does_not_parse() {
    for text in [
        "{a",
        "a}",
        r"\end{pmatrix}",
        r"\begin{pmatrix} a",
        r"\begin{pmatrix} a \end{matrix}",
        r"\begin{array} a \end{array}",
        r"\right)",
        r"\left( a",
        r"\frac{a}",
        "x^",
        "{a & b}",
        r"\sqrt{a \\ b}",
        r"a \",
        r"\text{a $ b}",
    ] {
        assert!(Formula::parse(text).is_err(), "{text} parsed");
    }
}

#[test]
fn a_command_names_its_symbol_and_is_written_back_by_its_name() {
    let f = parse(r"a \le b");
    assert_eq!(
        f.rows[0][0][1],
        Atom::Symbol {
            ch: '≤',
            class: Class::Rel,
            name: Some("le")
        }
    );
    assert_eq!(f.to_latex(), r"a \le b");
}

#[test]
fn rows_and_alignment_points_make_the_grid() {
    let f = parse(r"a &= b \\ &= c");
    assert_eq!(f.rows.len(), 2);
    assert_eq!(f.rows[0].len(), 2);
    assert!(f.rows[1][0].is_empty());
    assert_eq!(f.to_latex(), r"a & = b \\ & = c");
}

#[test]
fn operator_names_are_atoms_of_their_own() {
    let f = parse(r"\lim_{n} \sin x");
    let [Atom::Scripts { nucleus, .. }, Atom::OperatorName("sin"), _] = f.rows[0][0].as_slice()
    else {
        panic!("{:?}", f.rows);
    };
    assert_eq!(nucleus, &vec![Atom::OperatorName("lim")]);
}

#[test]
fn a_text_run_keeps_its_characters() {
    let f = parse(r"\text{if } x");
    assert_eq!(f.rows[0][0][0], Atom::Text("if ".to_owned()));
    assert_eq!(f.to_latex(), r"\text{if } x");
    let f = parse(r"\text{\$5 \{a\}}");
    assert_eq!(f.rows[0][0][0], Atom::Text(r"\$5 \{a\}".to_owned()));
    assert_eq!(f.to_latex(), r"\text{\$5 \{a\}}");
}

fn sym(ch: char) -> Atom {
    Atom::Symbol {
        ch,
        class: rootpoint_math::Class::Ord,
        name: None,
    }
}

/// Formulas of every kind of atom, the trees `parse` can give.
fn atom(depth: u32) -> BoxedStrategy<Atom> {
    let leaf = prop_oneof![
        prop::sample::select(vec![
            'a', 'b', 'x', 'y', '1', '2', '(', ')', '+', '=', ',', 'α'
        ])
        .prop_map(|c| {
            let class = match c {
                '+' => Class::Bin,
                '=' => Class::Rel,
                '(' => Class::Open,
                ')' => Class::Close,
                ',' => Class::Punct,
                _ => Class::Ord,
            };
            Atom::Symbol {
                ch: c,
                class,
                name: None,
            }
        }),
        prop::sample::select(vec!["alpha", "leq", "sum", "int", "infty", "cdot", "to"]).prop_map(
            |n| {
                let c = command(n).unwrap();
                Atom::Symbol {
                    ch: c.ch,
                    class: c.class,
                    name: Some(c.name),
                }
            }
        ),
        prop::sample::select(vec!["sin", "lim", "max"]).prop_map(Atom::OperatorName),
        prop::sample::select(vec![Space::Thin, Space::Quad, Space::NegativeThin])
            .prop_map(Atom::Space),
        "[a-z ]{0,6}".prop_map(Atom::Text),
        "[a-z]{1,5}"
            .prop_filter("not a known name", |n| {
                Formula::parse(&format!("\\{n}"))
                    .map(|f| matches!(f.rows[0][0].as_slice(), [Atom::Unknown(_)]))
                    .unwrap_or(false)
            })
            .prop_map(Atom::Unknown),
    ];
    if depth == 0 {
        return leaf.boxed();
    }
    let l = move || list(depth - 1);
    prop_oneof![
        3 => leaf,
        1 => (l(), l()).prop_map(|(num, den)| Atom::Fraction { num, den }),
        1 => (prop::option::of(l()), l()).prop_map(|(index, body)| Atom::Radical { index, body }),
        1 => (l(), prop::option::of(l()), prop::option::of(l()))
            .prop_filter("a script", |(_, sub, sup)| sub.is_some() || sup.is_some())
            .prop_map(|(nucleus, sub, sup)| Atom::Scripts { nucleus, sub, sup }),
        1 => (
            prop::option::of(prop::sample::select(vec!['(', '[', '|'])),
            prop::option::of(prop::sample::select(vec![')', ']', '|'])),
            l()
        )
            .prop_map(|(o, c, body)| Atom::Delimited {
                open: o.map(|ch| Delim { ch, name: None }),
                close: c.map(|ch| Delim { ch, name: None }),
                body,
            }),
        1 => (
            prop::sample::select(vec![Accent::Hat, Accent::Bar, Accent::Overline, Accent::Vec]),
            l()
        )
            .prop_map(|(accent, body)| Atom::Accent { accent, body }),
        1 => (
            prop::sample::select(vec![MathVariant::Bold, MathVariant::Roman, MathVariant::DoubleStruck]),
            l()
        )
            .prop_map(|(font, body)| Atom::FontChange { font, body }),
        1 => (
            prop::sample::select(vec![
                Environment::Matrix,
                Environment::PMatrix,
                Environment::BMatrix,
                Environment::Cases
            ]),
            prop::collection::vec(prop::collection::vec(l(), 1..3), 1..3)
        )
            .prop_map(|(env, rows)| Atom::Matrix { env, rows }),
        1 => l().prop_map(Atom::Group),
    ]
    .boxed()
}

/// A list `parse` can give: no script follows a group, which `parse` reads as the script's
/// nucleus.
fn list(depth: u32) -> BoxedStrategy<List> {
    prop::collection::vec(atom(depth), 0..4)
        .prop_filter("a group before a script is its nucleus", |l| {
            !l.windows(2)
                .any(|w| matches!(w, [Atom::Group(_), Atom::Scripts { .. }]))
        })
        .prop_filter("a script after a script is its second", |l| {
            !l.windows(2)
                .any(|w| matches!(w, [Atom::Scripts { .. }, Atom::Scripts { nucleus, .. }] if nucleus.is_empty()))
        })
        .boxed()
}

fn formula() -> impl Strategy<Value = Formula> {
    prop::collection::vec(prop::collection::vec(list(2), 1..3), 1..3)
        .prop_map(|rows| Formula { rows })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// A formula written by `to_latex` reads back equal, and is written back the same.
    #[test]
    fn the_subset_round_trips(f in formula()) {
        let text = f.to_latex();
        let back = Formula::parse(&text).map_err(|e| TestCaseError::fail(format!("{text}: {e}")))?;
        prop_assert_eq!(&back, &f, "{}", text);
        prop_assert_eq!(back.to_latex(), text);
    }
}

#[test]
fn a_delimiter_is_an_open_a_close_or_a_bar() {
    let f = parse(r"\left< x \right>");
    let [Atom::Delimited { open, close, .. }] = f.rows[0][0].as_slice() else {
        panic!("{:?}", f.rows);
    };
    assert_eq!(open.map(|d| d.ch), Some('⟨'));
    assert_eq!(close.map(|d| d.ch), Some('⟩'));
    assert!(Formula::parse(r"\left\alpha x \right)").is_err());
    assert!(Formula::parse(r"\left⟨ x \right⟩").is_ok());
    assert!(Formula::parse(r"\left\lfloor x \right\rfloor").is_ok());
    assert!(Formula::parse(r"\left{ x \right)").is_err());
}

#[test]
fn a_quote_is_a_prime_in_the_superscript() {
    assert_eq!(parse("f'"), parse(r"f^{\prime}"));
    assert_eq!(parse("f''"), parse(r"f^{\prime \prime}"));
    assert_eq!(parse("f_i'"), parse(r"f_i^{\prime}"));
    // A superscript after primes joins them, as in TeX; a prime after a superscript is a second
    // superscript, refused as TeX refuses it.
    assert_eq!(parse("f'^2"), parse(r"f^{\prime 2}"));
    assert!(Formula::parse("f^2'").is_err());
    assert!(Formula::parse("x^{}'").is_err());
}

#[test]
fn an_escaped_character_is_written_back_escaped() {
    let f = parse(r"\{ a \} \% \_");
    assert_eq!(parse(&f.to_latex()), f);
    assert_eq!(f.to_latex(), r"\{ a\} \% \_");
}

/// A formula `parse` gives writes back to a text that reads back equal, a `]` in an index
/// included.
#[test]
fn a_bracket_in_an_index_round_trips() {
    for text in [r"\sqrt[{]}^2]{x}", r"\sqrt[{]}']{x}"] {
        let f = parse(text);
        assert_eq!(parse(&f.to_latex()), f, "{text}");
    }
}
