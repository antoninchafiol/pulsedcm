use std::sync::Arc;
use dashmap::DashMap;
use pulsedcm_core::{FileDicomObject, InMemDicomObject, PrimitiveValue, Value, Tag, VR, Result};
use phf::*;
use smallvec::smallvec;
use uuid::Uuid;


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
        uid_map: &Arc<DashMap<String, String>>
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
