#[macro_use]
extern crate hdk;
extern crate holochain_wasm_utils;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate boolinator;

use boolinator::Boolinator;
use hdk::globals::G_MEM_STACK;
use holochain_wasm_utils::{
    error::RibosomeErrorCode,
    holochain_core_types::hash::HashString,
    memory_serialization::*, memory_allocation::*
};
use hdk::RibosomeError;
use holochain_wasm_utils::holochain_core_types::json::JsonString;

#[no_mangle]
pub extern "C" fn check_global(encoded_allocation_of_input: u32) -> u32 {
    unsafe {
        G_MEM_STACK = Some(SinglePageStack::from_encoded_allocation(encoded_allocation_of_input).unwrap());
    }
    #[allow(unused_must_use)]
    {
        hdk::debug(hdk::APP_NAME);
        hdk::debug(hdk::APP_DNA_HASH);
        hdk::debug(hdk::APP_AGENT_ID_STR);
        hdk::debug(hdk::APP_AGENT_KEY_HASH);
        hdk::debug(hdk::APP_AGENT_INITIAL_HASH);
        hdk::debug(hdk::APP_AGENT_LATEST_HASH);
    }


    return 0;
}


#[derive(Deserialize, Serialize, Default)]
struct CommitOutputStruct {
    address: String,
}

impl From<CommitOutputStruct> for JsonString {
    fn from(commit_output_struct: CommitOutputStruct) -> JsonString {
        JsonString::from(
            serde_json::to_string(&commit_output_struct).expect("could not Jsonify CommitOutputStruct")
        )
    }
}

#[no_mangle]
pub extern "C" fn check_commit_entry(encoded_allocation_of_input: u32) -> u32 {

    #[derive(Deserialize, Default)]
    struct CommitInputStruct {
        entry_type_name: String,
        entry_content: String,
    }

    unsafe {
        G_MEM_STACK = Some(SinglePageStack::from_encoded_allocation(encoded_allocation_of_input).unwrap());
    }

    // Deserialize and check for an encoded error
    let result = load_json(encoded_allocation_of_input as u32);
    if let Err(e) = result {
        hdk::debug(format!("ERROR: {:?}", e)).expect("debug() must work");
        return RibosomeErrorCode::ArgumentDeserializationFailed as u32;
    }

    let input: CommitInputStruct = result.unwrap();
    let entry_content = serde_json::from_str::<serde_json::Value>(&input.entry_content);
    let entry_content = entry_content.unwrap();
    let res = hdk::commit_entry(&input.entry_type_name, entry_content);

    let res_obj = match res {
        Ok(hash_str) => CommitOutputStruct {address: hash_str.to_string()},
        Err(RibosomeError::RibosomeFailed(err_str)) => {
            unsafe {
                return store_json_into_encoded_allocation(&mut G_MEM_STACK.unwrap(), err_str) as u32;
            }
        },
       Err(_) => unreachable!(),
    };
    unsafe {
        return store_json_into_encoded_allocation(&mut G_MEM_STACK.unwrap(), res_obj) as u32;
    }
}

//
zome_functions! {
    check_commit_entry_macro: |entry_type_name: String, entry_content: String| {
        let entry_content = serde_json::from_str::<serde_json::Value>(&entry_content);
        hdk::commit_entry(&entry_type_name, entry_content.unwrap())
        // match res {
        //     Ok(hash_str) => json!({ "address": hash_str }),
        //     Err(RibosomeError::ValidationFailed(msg)) => json!({ "validation failed": msg}),
        //     Err(RibosomeError::RibosomeFailed(err_str)) => json!({ "error": err_str}),
        //     Err(_) => unreachable!(),
        // }
    }

    check_get_entry: |entry_hash: HashString| {
        let res = hdk::get_entry(entry_hash);
        match res {
            Ok(Some(serialized_entry)) => {
                let maybe_entry_value : Result<serde_json::Value, _> = serde_json::from_str(&String::from(serialized_entry));
                match maybe_entry_value {
                    Ok(entry_value) => entry_value,
                    Err(err) => json!({"error trying deserialize entry": err.to_string()}),
                }
            },
            Ok(None) => json!({"got back no entry": true}),
            Err(RibosomeError::RibosomeFailed(err_str)) => json!({"get entry Err": err_str}),
            Err(_) => unreachable!(),
        }
    }
}


#[derive(Serialize, Deserialize)]
struct TweetResponse {
    first: String,
    second: String,
}

zome_functions! {
    send_tweet: |author: String, content: String| {

        TweetResponse { first: author,  second: content}
    }
}

#[derive(Serialize, Deserialize)]
struct TestEntryType {
    stuff: String,
}

validations! {
    [ENTRY] validate_testEntryType {
        [hdk::ValidationPackage::Entry]
        |entry: TestEntryType, _ctx: hdk::ValidationData| {
            (entry.stuff != "FAIL")
                .ok_or_else(|| "FAIL content is not allowed".to_string())
        }
    }
}
