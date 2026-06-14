

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol, Vec,
};

#[contracttype]
pub struct PackageRecord {
    pub ecosystem: Symbol,
    pub package_name: String,
    pub stellar_address: Address,
    pub registered_by: Address,
    pub registered_at: u64,
    pub verified: bool,
    pub verified_at: u64, // 0 means not verified
}

#[contracttype]
pub struct Stats {
    pub total_registered: u32,
    pub total_verified: u32,
    pub npm_count: u32,
    pub cargo_count: u32,
}

#[contracttype]
enum DataKey {
    Admin,
    Count,
    Package(Symbol, String),
}

#[contract]
pub struct DepthDripContract;

#[contractimpl]
impl DepthDripContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already_initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Count, &0u32);
    }

    pub fn register(
        env: Env,
        ecosystem: Symbol,
        package_name: String,
        stellar_address: Address,
    ) {
        stellar_address.require_auth();

        let key = DataKey::Package(ecosystem.clone(), package_name.clone());
        let now = env.ledger().timestamp();
        let is_new = !env.storage().persistent().has(&key);

        let rec = PackageRecord {
            ecosystem: ecosystem.clone(),
            package_name: package_name.clone(),
            stellar_address: stellar_address.clone(),
            registered_by: stellar_address.clone(),
            registered_at: now,
            verified: false,
            verified_at: 0,
        };

        env.storage().persistent().set(&key, &rec);

        if is_new {
            let mut count: u32 = env
                .storage()
                .instance()
                .get(&DataKey::Count)
                .unwrap_or(0u32);
            count = count.checked_add(1).expect("count overflow");
            env.storage().instance().set(&DataKey::Count, &count);
        }

        env.events().publish(
            (symbol_short!("PkgReg"),),
            (ecosystem, package_name, stellar_address, now),
        );
    }

    pub fn get_address(env: Env, ecosystem: Symbol, package_name: String) -> Option<Address> {
        let key = DataKey::Package(ecosystem, package_name);
        env.storage()
            .persistent()
            .get::<DataKey, PackageRecord>(&key)
            .map(|r| r.stellar_address)
    }

    pub fn get_record(
        env: Env,
        ecosystem: Symbol,
        package_name: String,
    ) -> Option<PackageRecord> {
        let key = DataKey::Package(ecosystem, package_name);
        env.storage().persistent().get(&key)
    }

    pub fn get_batch(
        env: Env,
        packages: Vec<(Symbol, String)>,
    ) -> Vec<Option<Address>> {
        let mut out = Vec::new(&env);
        for p in packages.iter() {
            let (eco, name) = p;
            let key = DataKey::Package(eco, name);
            let addr: Option<Address> = env
                .storage()
                .persistent()
                .get::<DataKey, PackageRecord>(&key)
                .map(|r| r.stellar_address);
            out.push_back(addr);
        }
        out
    }

    pub fn remove(env: Env, ecosystem: Symbol, package_name: String) {
        let key = DataKey::Package(ecosystem.clone(), package_name.clone());
        let rec: PackageRecord = env
            .storage()
            .persistent()
            .get(&key)
            .expect("not_found");

        let admin: Option<Address> = env.storage().instance().get(&DataKey::Admin);

        // caller must be the original registrant or admin
        if let Some(admin_addr) = admin {
            if rec.registered_by != admin_addr {
                rec.registered_by.require_auth();
            } else {
                admin_addr.require_auth();
            }
        } else {
            rec.registered_by.require_auth();
        }

        env.storage().persistent().remove(&key);
        env.events().publish(
            (symbol_short!("PkgRem"),),
            (ecosystem, package_name, env.ledger().timestamp()),
        );
    }

    pub fn verify(env: Env, ecosystem: Symbol, package_name: String) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not_initialized");
        admin.require_auth();

        let key = DataKey::Package(ecosystem.clone(), package_name.clone());
        let mut rec: PackageRecord = env.storage().persistent().get(&key).expect("not_found");
        rec.verified = true;
        let now = env.ledger().timestamp();
        rec.verified_at = now;
        env.storage().persistent().set(&key, &rec);

        env.events().publish(
            (symbol_short!("PkgVer"),),
            (ecosystem, package_name, now),
        );
    }

    pub fn get_stats(env: Env) -> Stats {
        let total_registered: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Count)
            .unwrap_or(0u32);
        Stats {
            total_registered,
            total_verified: 0,
            npm_count: 0,
            cargo_count: 0,
        }
    }
}
