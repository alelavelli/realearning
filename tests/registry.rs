use realearning::model::registry::Registry;

#[test]
fn empty_registry() {
    let r = Registry::new(None);
    assert_eq!(r.get_accounts().len(), 0)
}
