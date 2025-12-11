use std::sync::Arc;
use dashmap::DashMap;
use pulsedcm_core::{FileDicomObject, InMemDicomObject, PrimitiveValue, Value, Tag, VR, Result};
use phf::*;
use smallvec::smallvec;
use uuid::Uuid;
use sha2::{Sha256, Digest};


// ---------- Performance branch -----------------
pub enum Action {
    Empty,
    Remove, 
    ReplaceDummy,
    ReplaceUID,
    Keep,
}

pub static DEID_MAP: phf::Map<u32, Action> = phf_map!{
    0x00080018 => Action::ReplaceUID,   // SOPInstanceUID
    0x00080020 => Action::Empty,        // StudyDate
    0x00080030 => Action::Empty,        // StudyTime
    0x00080050 => Action::Empty,        // AccessionNumber
    0x00080080 => Action::Empty,        // InstitutionName
    0x00080090 => Action::Remove,       // Referring Physician
    0x00081030 => Action::Empty,        // StudyDescription
    0x00100010 => Action::Remove,       // PatientName
    0x00100020 => Action::Remove,       // PatientID
    0x00100030 => Action::Empty,        // PatientBirthDate
    0x00181000 => Action::Empty,        // DeviceSerialNumber
    0x0020000D => Action::ReplaceUID,   // StudyInstanceUID
    0x0020000E => Action::ReplaceUID,   // SeriesInstanceUID
};

impl Action {
    pub fn apply(&self, 
        data: &mut InMemDicomObject, 
        u32_tag: &u32, 
        vr: &VR,
        uid_map: &Arc<DashMap<String, String>>,
        uid_to_hash: &bool
    ) -> Result<()> {
        // Rebuilding a coherent taf from the u33 compressed version
        let built_tag = Tag {0: (u32_tag >> 16) as u16,
                             1: (u32_tag & 0xFFFF) as u16
        };
        match self {
            Action::Empty        => {
                data.update_value_at(built_tag, |v|{
                    *v.primitive_mut().unwrap() = PrimitiveValue::Empty;
                })?;
                Ok(())
            },
            Action::Remove       => {
                let _ = data.remove_element(built_tag);
                Ok(())
            },
            Action::ReplaceDummy => {
                data.update_value_at(built_tag, |v|{
                    *v.primitive_mut().unwrap() = dummy_from_vr(vr);
                })?;
                Ok(())
            },
            Action::ReplaceUID   => {
                // If the hash-uid option is given, then we hash it
                if *uid_to_hash { 
                    data.update_value_at(built_tag, |v|{
                        if let Some(old_val) = v.primitive_mut(){
                            *old_val = PrimitiveValue::from(hash_uid(old_val));
                        }
                    })?;
                }
                // If hash-uid is false, then we process the UID exactly as
                // the Action Code U from Chapter E
                else { 
                    if let Ok(old_val) = data.value_at(built_tag){
                        if let Some(old_val) = old_val.primitive(){
                            // Get the final String 
                            let old_val = old_val.to_string();

                            // Check if it's contained within the map
                            if let Some(uid_match) = uid_map.get(&old_val) {
                                // Found the right entry / replacing the current UID with the generated
                                // one from the uid map
                                data.update_value_at(built_tag, |v|{
                                    // I'm not using an "if let" statement here as 
                                    *v.primitive_mut().unwrap() = PrimitiveValue::from(uid_match.value().as_str());
                                })?;
                            } else {
                                // Generate a new UUID 
                                let new_generated_uid = generate_uid();
                                // Change the current field to the new one
                                data.update_value_at(built_tag, |v|{
                                    *v.primitive_mut().unwrap() = PrimitiveValue::from(new_generated_uid.clone());
                                })?;
                                // Add the change to the hashmap
                                uid_map.insert(old_val, new_generated_uid);
                            }
                        }
                    }
                }
                Ok(())
            },
            Action::Keep         => {
                Ok(())
            },
        }
    }
}

fn generate_uid() -> String {
    format!("2.25.{}", Uuid::new_v4().to_u128_le())
}

fn hash_uid(val: &mut PrimitiveValue) -> String {
    let digested = Sha256::digest(val.clone().to_bytes());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digested[..16]);
    let dec = u128::from_be_bytes(bytes).to_string();

    format!("2.25.{}", dec)
}

fn dummy_from_vr(vr: &VR) -> PrimitiveValue{
    match vr {
        // String-like
        VR::AE => PrimitiveValue::from("ANON_AE"),
        VR::AS => PrimitiveValue::from("000Y"),
        VR::CS => PrimitiveValue::from("UNKNOWN"),
        VR::DA => PrimitiveValue::from("19000101"),
        VR::DS => PrimitiveValue::from("0.0"),
        VR::DT => PrimitiveValue::from("19000101000000"),
        VR::IS => PrimitiveValue::from("0"),
        VR::LO => PrimitiveValue::from("Removed"),
        VR::LT => PrimitiveValue::from("Removed for Privacy"),
        VR::PN => PrimitiveValue::from("Anonymous^Patient"),
        VR::SH => PrimitiveValue::from("REMOVED"),
        VR::ST => PrimitiveValue::from("Removed"),
        VR::TM => PrimitiveValue::from("000000"),
        VR::UC => PrimitiveValue::from("Removed"),
        VR::UI => PrimitiveValue::from("2.25.999999999999999999"),
        VR::UR => PrimitiveValue::from("https://anon.invalid/"),
        VR::UT => PrimitiveValue::from("Removed for Privacy"),

        // Binary / Numeric
        VR::FL => PrimitiveValue::from(0.0_f32),
        VR::FD => PrimitiveValue::from(0.0_f64),
        VR::SL => PrimitiveValue::from(0_i32),
        VR::SS => PrimitiveValue::from(0_i16),
        VR::SV => PrimitiveValue::from(0_i64),
        VR::UL => PrimitiveValue::from(0_u32),
        VR::US => PrimitiveValue::from(0_u16),
        VR::UV => PrimitiveValue::from(0_u64),

        // Tag / Sequence
        VR::AT => PrimitiveValue::from("(0000,0000)"),

        VR::OB | VR::UN => PrimitiveValue::U8(smallvec![0_u8]),
        VR::OW => PrimitiveValue::U16(smallvec![0_u16]),
        VR::OF => PrimitiveValue::F32(smallvec![0.0_f32]),
        VR::OD => PrimitiveValue::F64(smallvec![0.0_f64]),
        // TODO: Redo for Value instead of PrimitiveValue
        VR::SQ => {
            PrimitiveValue::Empty
        }
        // Fallback for anything new or unknown
        _ => PrimitiveValue::Empty,    
    }
}

