use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", seq_form=false)]
enum ParentChildNoSeq {
    Title,
    #[serde(untagged)]
    SubStructure(ChildNoSeq),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", seq_form=false)]
enum ParentNoSeq {
    Title,
    #[serde(untagged)]
    SubStructure(Child),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum Parent {
    Title,
    #[serde(untagged)]
    SubStructure(Child),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", seq_form=true)]
enum ParentSeq {
    Title,
    #[serde(untagged)]
    SubStructure(Child),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "subtype", seq_form=false)]
enum ChildNoSeq {
    Topic, Sidebar
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "subtype")]
enum Child {
    Topic, Sidebar
}

#[test]
fn test() {
    // Using seq=false on every level will cause it to fail
    let v  = serde_json::from_str::<ParentChildNoSeq>("[0]");
    assert!(v.is_err());

    // Using seq=false will cause it to fail, but if the child is not seq=true, the  child will succeed
    let v  = serde_json::from_str::<ParentNoSeq>("[0]");
    assert!(matches!(v, Ok(ParentNoSeq::SubStructure(Child::Topic))));

    // Using default beahvior will succeed
    let v  = serde_json::from_str::<Parent>("[0]");
    assert!(v.is_ok());

    // Using seq=true beahvior will succeed
    let v  = serde_json::from_str::<ParentSeq>("[0]");
    assert!(v.is_ok());

    // As in the docs:
    // There is no explicit tag identifying which variant the data contains. Serde will try to
    // match the data against each variant in order and the first one that deserializes
    // successfully is the one returned.
    let v = serde_json::from_str::<Parent>("[\"Topic\"]");
    assert!(v.is_ok());
    let v = serde_json::from_str::<Child>("[\"Topic\"]");
    assert!(v.is_ok());

    // With seq=false it should fail
    let v = serde_json::from_str::<ChildNoSeq>("[\"Topic\"]");
    assert!(v.is_err());
    let v = serde_json::from_str::<ParentNoSeq>("[\"Topic\"]");
    assert!(v.is_ok());
    let v = serde_json::from_str::<ParentChildNoSeq>("[\"Topic\"]");
    assert!(v.is_err());
}
