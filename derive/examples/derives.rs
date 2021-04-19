use enum_position::NumIdentity;


#[derive(NumIdentity)]
enum EnumTest {
    Unary,
    Tuple(u8, u16, u32),
    CStruct {
        field_1:String,
        field_2:usize,
        field_3:Vec<u8>
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use crate::EnumTest;

    #[test]
    fn it_works() {
        let cstruct = EnumTest::CStruct {
            field_1: "".to_string(),
            field_2: 0,
            field_3: vec![]
        };

        assert_eq!(cstruct.position(),2);

        let unary = EnumTest::Unary;

        assert_eq!(unary.position(),0);
    }
}