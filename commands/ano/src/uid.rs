use pulsedcm_core::{PrimitiveValue};
use uuid::Uuid;
use sha2::{Sha256, Digest};

pub fn generate_uid() -> String {
    format!("2.25.{}", Uuid::new_v4().to_u128_le())
}

pub fn hash_uid(val: &mut PrimitiveValue) -> String {
    let digested = Sha256::digest(val.clone().to_bytes());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digested[..16]);
    let dec = u128::from_be_bytes(bytes).to_string();

    format!("2.25.{}", dec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_uid_is_valid(){
        assert!(generate_uid().starts_with("2.25"));
    }

    #[test]
    fn generate_different_uid(){
        let a = generate_uid();
        let b = generate_uid();
        assert!(a != b);
    }

    #[test]
    fn hash_uid_is_valid(){
        let mut p = PrimitiveValue::from("12345");
        let h = hash_uid(&mut p);
        assert!(h.starts_with("2.25"));
        assert!(h.contains("119071193728809738652551802460593484980"));
    }

    #[test]
    fn hash_uid_is_not_producing_duplicates(){
        let mut p = PrimitiveValue::from("12345");
        let a = hash_uid(&mut p);
        let b = hash_uid(&mut p);
        assert_eq!(a, b);
    }
}
