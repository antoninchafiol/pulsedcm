use pulsedcm_core::Tag;

pub enum ActionCode {
    D,       // Replace with dummy value
    Z,       // Zero-length or dummy
    X,       // Remove
    K,       // Keep
    C,       // Clean
    U,       // Replace with consistent UID
    ZD,      // Z unless D required for conformance
    XZ,      // X unless Z required for conformance
    XD,      // X unless D required for conformance
    XZD,     // X unless Z/D required for conformance
    XZU,     // X unless Z/U required for conformance
}

pub struct PolicyAction{
    tag: Tag, // the tag
    basic: ActionCode,  // Basic profile
    ret_sf_priv: Option<ActionCode>, //Retain safe private
    ret_uids: Option<ActionCode>, // Retain UID
    ret_dev_id: Option<ActionCode>, // Retain device identity
    ret_inst_id: Option<ActionCode>, // Retain institution ID
    ret_pt_char: Option<ActionCode>, // Retain patient characteristic
    ret_lg_full_dt: Option<ActionCode>, //retain long full dates
    ret_lg_mod_dt: Option<ActionCode>, // retain long modified dates
    cln_desc: Option<ActionCode>, // Clean Descriptors
    cln_struc_cnt: Option<ActionCode>, // Clean Structured content
    cln_graph: Option<ActionCode>, // Clean Graphics
}
