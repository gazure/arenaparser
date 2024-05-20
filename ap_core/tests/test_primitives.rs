use anyhow::Result;
use serde_json::{from_str, json};
use ap_core::mtga_events::primitives::AnnotationType;

#[test]
fn test_annotation_type() -> Result<()> {
    let annotation_type_strings =  json!{[
        "AnnotationType_ResolutionStart",
        "AnnotationType_ResolutionComplete",
        "AnnotationType_CardRevealed"
    ]};
    let annotation_types: Vec<AnnotationType> = serde_json::from_value(annotation_type_strings)?;
    Ok(())
}

