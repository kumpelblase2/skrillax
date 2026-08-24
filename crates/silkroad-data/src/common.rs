use crate::ParseError;
use num_enum_derive::TryFromPrimitive;
use silkroad_definitions::TypeId;
use std::str::FromStr;
use std::time::Duration;

#[derive(Clone)]
pub struct RefCommon {
    pub service: bool,          // column 0
    pub ref_id: u32,            // column 1
    pub id: String,             // column 2
    pub type_id: TypeId,        // column 9-12
    pub country: RefOrigin,     // column 14
    pub despawn_time: Duration, // column 13
}

impl RefCommon {
    pub fn from_columns(elements: &[&str]) -> Result<Self, ParseError> {
        let service = elements.get(0).ok_or(ParseError::MissingColumn(0))?.parse::<u8>()?;
        if service > 1 {
            return Err(ParseError::InvalidBoolean(service));
        }

        Ok(Self {
            service: service == 1,
            ref_id: elements.get(1).ok_or(ParseError::MissingColumn(1))?.parse()?,
            id: elements.get(2).ok_or(ParseError::MissingColumn(2))?.to_string(),
            type_id: TypeId(
                elements.get(9).ok_or(ParseError::MissingColumn(9))?.parse()?,
                elements.get(10).ok_or(ParseError::MissingColumn(10))?.parse()?,
                elements.get(11).ok_or(ParseError::MissingColumn(11))?.parse()?,
                elements.get(12).ok_or(ParseError::MissingColumn(12))?.parse()?,
            ),
            despawn_time: Duration::from_millis(elements.get(13).ok_or(ParseError::MissingColumn(13))?.parse()?),
            country: elements.get(14).ok_or(ParseError::MissingColumn(14))?.parse()?,
        })
    }
}

#[derive(TryFromPrimitive, Copy, Clone)]
#[repr(u8)]
pub enum RefOrigin {
    Chinese = 0,
    European = 1,
    General = 3,
}

impl FromStr for RefOrigin {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: u8 = s.parse()?;
        Ok(RefOrigin::try_from(value)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn common_row(service: &str) -> Vec<&str> {
        let mut elements = vec![""; 16];
        elements[0] = service;
        elements[1] = "42";
        elements[2] = "ITEM_ETC_TEST";
        elements[9] = "3";
        elements[10] = "3";
        elements[11] = "1";
        elements[12] = "1";
        elements[13] = "300000";
        elements[14] = "0";
        elements
    }

    #[test]
    fn parse_service_flag() {
        let common = RefCommon::from_columns(&common_row("1")).unwrap();
        assert!(common.service);

        let common = RefCommon::from_columns(&common_row("0")).unwrap();
        assert!(!common.service);
    }

    #[test]
    fn reject_invalid_service_value() {
        assert!(RefCommon::from_columns(&common_row("2")).is_err());
        assert!(RefCommon::from_columns(&common_row("true")).is_err());
    }
}
