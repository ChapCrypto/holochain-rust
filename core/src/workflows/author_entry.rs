use crate::{
    agent::actions::commit::commit_entry,
    context::Context,
    network::actions::publish::publish_entry,
    nucleus::actions::{
        build_validation_package::build_validation_package, validate::validate_entry,
    },
};

use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    entry::Entry,
    error::HolochainError,
    validation::{EntryAction, EntryLifecycle, ValidationData},
};
use std::sync::Arc;

pub async fn author_entry<'a>(
    entry: &'a Entry,
    context: &'a Arc<Context>,
) -> Result<Address, HolochainError> {
    // 1. Build the context needed for validation of the entry
    let validation_package = await!(build_validation_package(&entry, &context))?;
    let validation_data = ValidationData {
        package: validation_package,
        sources: vec![Address::from("<insert your agent key here>")],
        lifecycle: EntryLifecycle::Chain,
        action: EntryAction::Commit,
    };

    // 2. Validate the entry
    await!(validate_entry(entry.clone(), validation_data, &context))?;
    // 3. Commit the entry
    await!(commit_entry(entry.clone(), &context))?;

    // 4. Publish the valid entry to DHT
    await!(publish_entry(entry.address(), &context))
}

#[cfg(test)]
pub mod tests {
    use super::author_entry;
    use crate::nucleus::actions::tests::*;
    use futures::executor::block_on;
    use holochain_core_types::entry::test_entry;
    use std::{thread, time};

    #[test]
    /// test that a commit will publish and entry to the dht of a connected instance via the mock network
    fn test_commit_with_dht_publish() {
        let dna = test_dna();
        let (_instance1, context1) = instance_by_name("jill", dna.clone());
        let (_instance2, context2) = instance_by_name("jack", dna);

        let entry_address = block_on(author_entry(&test_entry(), &context1));

        println!("AUTHOR ENTRY ADDRESS: {:?}", entry_address);
        let entry_address = entry_address.unwrap();
        thread::sleep(time::Duration::from_millis(1000));

        let state = &context2.state().unwrap();
        let json = state
            .dht()
            .content_storage()
            .read()
            .unwrap()
            .fetch(&entry_address)
            .expect("could not fetch from CAS");

        let x: String = json.unwrap().to_string();
        assert_eq!(
            x,
            "{\"value\":\"\\\"test entry value\\\"\",\"entry_type\":\"testEntryType\"}".to_string()
        );
    }
}
