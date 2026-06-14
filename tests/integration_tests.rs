#![cfg(test)]

use soroban_sdk::testutils::{Address as TestAddr, Ledger};
use soroban_sdk::{symbol_short, Address, Env, String, Symbol};

use depthdrip_contract::{DepthDripContract, DepthDripContractClient};

fn setup() -> (Env, Address, DepthDripContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = <Address as TestAddr>::generate(&env);
    let contract_id = env.register(DepthDripContract, ());
    let client = DepthDripContractClient::new(&env, &contract_id);
    client.initialize(&admin);
    (env, admin, client)
}

fn npm() -> Symbol {
    symbol_short!("npm")
}

fn pkg(env: &Env, s: &str) -> String {
    String::from_str(env, s)
}

#[test]
fn register_and_lookup() {
    let (env, _, client) = setup();
    let owner = <Address as TestAddr>::generate(&env);
    client.register(&npm(), &pkg(&env, "lodash"), &owner);
    let addr = client.get_address(&npm(), &pkg(&env, "lodash"));
    assert_eq!(addr, Some(owner));
}

#[test]
fn duplicate_register_does_not_increment_count() {
    let (env, _, client) = setup();
    let owner = <Address as TestAddr>::generate(&env);
    client.register(&npm(), &pkg(&env, "lodash"), &owner);
    client.register(&npm(), &pkg(&env, "lodash"), &owner);
    let stats = client.get_stats();
    assert_eq!(stats.total_registered, 1);
}

#[test]
fn register_multiple_increments_count() {
    let (env, _, client) = setup();
    let a = <Address as TestAddr>::generate(&env);
    let b = <Address as TestAddr>::generate(&env);
    client.register(&npm(), &pkg(&env, "lodash"), &a);
    client.register(&npm(), &pkg(&env, "react"), &b);
    assert_eq!(client.get_stats().total_registered, 2);
}

#[test]
fn get_batch_returns_in_order() {
    let (env, _, client) = setup();
    let a = <Address as TestAddr>::generate(&env);
    client.register(&npm(), &pkg(&env, "lodash"), &a);

    let packages = soroban_sdk::vec![
        &env,
        (npm(), pkg(&env, "lodash")),
        (npm(), pkg(&env, "missing")),
    ];
    let results = client.get_batch(&packages);
    assert_eq!(results.get(0).unwrap(), Some(a));
    assert_eq!(results.get(1).unwrap(), None);
}

#[test]
fn remove_by_registrant() {
    let (env, _, client) = setup();
    let owner = <Address as TestAddr>::generate(&env);
    client.register(&npm(), &pkg(&env, "lodash"), &owner);
    client.remove(&npm(), &pkg(&env, "lodash"));
    assert_eq!(client.get_address(&npm(), &pkg(&env, "lodash")), None);
}

#[test]
fn verify_by_admin() {
    let (env, _, client) = setup();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    let owner = <Address as TestAddr>::generate(&env);
    client.register(&npm(), &pkg(&env, "lodash"), &owner);
    client.verify(&npm(), &pkg(&env, "lodash"));
    let rec = client.get_record(&npm(), &pkg(&env, "lodash")).unwrap();
    assert!(rec.verified);
    assert_eq!(rec.verified_at, 1_000_000);
}

#[test]
#[should_panic]
fn double_initialize_panics() {
    let (env, admin, client) = setup();
    client.initialize(&admin);
}
