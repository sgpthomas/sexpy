pub mod parser;

#[cfg(test)]
mod tests {
    use crate::parser::*;
    use lexpr::Value;

    #[derive(Debug)]
    pub struct PortDef {
        name: String,
        width: i64,
    }

    pub fn test_parse(v: Value) -> Result<Vec<PortDef>, ()> {
        let port_parser =
            match_head("port").then(match_var()).then(match_i64());

        let snort_parser =
            match_head("snort").then(match_var()).then(match_i64());

        let res = port_parser
            .or(snort_parser)
            .list()
            .call(v)
            .into_iter()
            .map(|(name, width)| PortDef { name, width })
            .collect();

        Ok(res)

        // let (name, width) = port_parser.call(v);
        // Ok(PortDef { name, width })
    }

    #[test]
    fn it_works() {
        let v = from_file("stdlib.rkt");
        println!("{:?}", v);
        let r = test_parse(v).expect("Parse unsucessful");
        println!("{:?}", r);
        assert_eq!(2, 3);
    }
}
