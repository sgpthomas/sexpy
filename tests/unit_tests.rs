use sexpy::Sexpy;

#[test]
fn simple_struct() {
    #[derive(Sexpy, Debug, PartialEq)]
    struct Portdef {
        name: String,
        width: u64,
    }

    let input = "(portdef foo 20)";
    let gold = Portdef {
        name: "foo".to_string(),
        width: 20,
    };
    assert_eq!(Portdef::parse(input), Ok(gold))
}

#[test]
fn simple_struct_one_field() {
    #[derive(Sexpy, Debug, PartialEq)]
    struct Portdef {
        name: String,
    }

    let input = "(portdef foo)";
    let gold = Portdef {
        name: "foo".to_string(),
    };
    assert_eq!(Portdef::parse(input), Ok(gold))
}

#[test]
fn simple_struct_no_fields() {
    #[derive(Sexpy, Debug, PartialEq)]
    struct Portdef {}

    assert_eq!(Portdef::parse("(portdef)"), Ok(Portdef {}));
    assert_eq!(Portdef::parse("(portdef   )"), Ok(Portdef {}));
    assert!(Portdef::parse("(portdef hi)").is_err());
}

#[test]
fn struct_rename_head() {
    #[derive(Sexpy, Debug, PartialEq)]
    #[sexpy(head = "port")]
    struct Portdef {
        name: String,
        width: i64,
    }

    assert_eq!(
        Portdef::parse("(port foo -32)"),
        Ok(Portdef {
            name: "foo".to_string(),
            width: -32,
        })
    )
}

#[test]
fn enum_rename_head() {
    #[derive(Sexpy, Debug, PartialEq)]
    #[sexpy(head = "plt")]
    enum Plant {
        PalmTree(String, u64),
        Cactus,
    }

    assert_eq!(
        Plant::parse("(plt test 4)"),
        Ok(Plant::PalmTree("test".to_string(), 4))
    );
    assert_eq!(Plant::parse("(plt)"), Ok(Plant::Cactus));
}

#[test]
fn unit_enum() {
    #[derive(Sexpy, Debug, PartialEq)]
    enum Plant {
        PalmTree,
        Cactus,
    }

    let input = "(plant)";
    assert_eq!(Plant::parse(input), Ok(Plant::PalmTree))
}

#[test]
fn named_enum_fields() {
    #[derive(Sexpy, Debug, PartialEq)]
    enum Plant {
        PalmTree { width: u64, name: String },
        Cactus { height: u64 },
    }

    assert_eq!(
        Plant::parse("(plant 200 cm)"),
        Ok(Plant::PalmTree {
            width: 200,
            name: "cm".to_string()
        })
    )
}

#[test]
fn same_prefix() {
    #[derive(Sexpy, Debug, PartialEq)]
    #[sexpy(nosurround, head = "foo")]
    struct Left {
        item: String,
    }

    #[derive(Sexpy, Debug, PartialEq)]
    #[sexpy(nosurround, head = "foo-bar")]
    struct Right {
        item: u64,
    }

    #[derive(Sexpy, Debug, PartialEq)]
    enum Either {
        Left { data: Left },
        Right { data: Right },
    }

    assert_eq!(
        Either::parse("(either foo hi)"),
        Ok(Either::Left {
            data: Left {
                item: "hi".to_string()
            }
        })
    );

    assert_eq!(
        Either::parse("(either foo-bar 32)"),
        Ok(Either::Right {
            data: Right { item: 32 }
        })
    );
}

#[test]
fn no_head() {
    #[derive(Sexpy, Debug, PartialEq)]
    #[sexpy(nohead)]
    enum Plant {
        #[sexpy(head = "cactus")]
        Cactus(String, u64),
        #[sexpy(head = "joshua-tree")]
        JoshuaTree(String, u64),
    }

    assert_eq!(
        Plant::parse("(cactus josh 400)"),
        Ok(Plant::Cactus("josh".to_string(), 400))
    );

    assert_eq!(
        Plant::parse("(joshua-tree carolina 4)"),
        Ok(Plant::JoshuaTree("carolina".to_string(), 4))
    );
}

#[test]
fn enum_differentiation() {
    #[derive(Sexpy, Debug, PartialEq)]
    enum Plant {
        #[sexpy(head = "cactus")]
        Cactus(String, u64),
        #[sexpy(head = "joshua-tree")]
        JoshuaTree(String, u64),
    }

    assert_eq!(
        Plant::parse("(plant cactus josh 400)"),
        Ok(Plant::Cactus("josh".to_string(), 400))
    );

    assert_eq!(
        Plant::parse("(plant joshua-tree carolina 4)"),
        Ok(Plant::JoshuaTree("carolina".to_string(), 4))
    );
}

#[test]
fn vector() {
    #[derive(Sexpy, Debug, PartialEq)]
    struct Song {
        name: String,
        #[sexpy(surround)]
        instrs: Vec<String>,
        notes: Vec<u64>,
    }

    assert_eq!(
        Song::parse("(song purr (piano cat) 11 12 13 12 13)"),
        Ok(Song {
            name: "purr".to_string(),
            instrs: vec!["piano".to_string(), "cat".to_string()],
            notes: vec![11, 12, 13, 12, 13]
        })
    )
}

#[test]
fn comments() {
    #[derive(Sexpy, Debug, PartialEq)]
    struct Song {
        name: String,
        #[sexpy(surround)]
        instrs: Vec<String>,
        notes: Vec<u64>,
    }

    assert_eq!(
            Song::parse(
                "; my cool song\n(song purr (piano cat) ; the good part!\n11 12 13 12 13)"
            ),
            Ok(Song {
                name: "purr".to_string(),
                instrs: vec!["piano".to_string(), "cat".to_string()],
                notes: vec![11, 12, 13, 12, 13]
            })
        )
}

#[test]
fn documentation() {
    /// This is some documentation
    #[derive(Sexpy, Debug, PartialEq)]
    #[sexpy(head = "port")]
    struct Portdef {
        /// this is a string field
        name: String,
        /// and this is a int field
        width: i64,
    }

    assert_eq!(
        Portdef::parse("(port foo -32)"),
        Ok(Portdef {
            name: "foo".to_string(),
            width: -32,
        })
    )
}
