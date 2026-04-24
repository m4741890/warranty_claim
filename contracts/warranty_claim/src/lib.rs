#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Map, String, Vec};

#[contract]
pub struct WarrantyClaim;

#[contractimpl]
impl WarrantyClaim {
    /// Initialize the contract
    pub fn init(env: Env) {
        if env.storage().instance().get::<_, bool>(&"initialized").is_some() {
            panic!("Already initialized");
        }
        env.storage().instance().set(&"initialized", &true);
    }

    /// Manufacturer registers a product with warranty duration
    pub fn register_product(env: Env, product_id: u64, manufacturer: Address, warranty_months: u64) {
        manufacturer.require_auth();

        let mut products: Map<u64, (Address, u64)> = env
            .storage()
            .instance()
            .get(&"products")
            .unwrap_or(Map::new(&env));

        products.set(product_id, (manufacturer, warranty_months));
        env.storage().instance().set(&"products", &products);
    }

    /// Owner registers warranty ownership for a product
    pub fn register_warranty(env: Env, product_id: u64, owner: Address) {
        owner.require_auth();

        let products: Map<u64, (Address, u64)> = env
            .storage()
            .instance()
            .get(&"products")
            .unwrap_or(Map::new(&env));

        if products.get(product_id).is_none() {
            panic!("Product not registered");
        }

        let mut warranty_owners: Map<u64, Address> = env
            .storage()
            .instance()
            .get(&"warranty_owners")
            .unwrap_or(Map::new(&env));

        warranty_owners.set(product_id, owner);
        env.storage().instance().set(&"warranty_owners", &warranty_owners);
    }

    /// Owner files a warranty claim for a product
    pub fn file_claim(env: Env, product_id: u64, owner: Address, description: String) {
        owner.require_auth();

        let warranty_owners: Map<u64, Address> = env
            .storage()
            .instance()
            .get(&"warranty_owners")
            .unwrap_or(Map::new(&env));

        let registered_owner = warranty_owners.get(product_id).unwrap_or_else(|| {
            panic!("Warranty not registered for this product");
        });

        if registered_owner != owner {
            panic!("Not the registered owner");
        }

        let mut claims: Map<(u64, u32), (String, String)> = env
            .storage()
            .instance()
            .get(&"claims")
            .unwrap_or(Map::new(&env));

        let claim_index = claims.keys().iter().filter(|(pid, _)| *pid == product_id).count() as u32;

        claims.set((product_id, claim_index), (description, String::from_str(&env, "pending")));
        env.storage().instance().set(&"claims", &claims);
    }

    /// Manufacturer resolves a claim (approves or rejects)
    pub fn resolve_claim(env: Env, product_id: u64, claim_index: u32, approved: bool) {
        let products: Map<u64, (Address, u64)> = env
            .storage()
            .instance()
            .get(&"products")
            .unwrap_or(Map::new(&env));

        let (manufacturer, _) = products.get(product_id).unwrap_or_else(|| {
            panic!("Product not registered");
        });

        manufacturer.require_auth();

        let mut claims: Map<(u64, u32), (String, String)> = env
            .storage()
            .instance()
            .get(&"claims")
            .unwrap_or(Map::new(&env));

        let key = (product_id, claim_index);
        let (description, _) = claims.get(key).unwrap_or_else(|| {
            panic!("Claim not found");
        });

        let status = if approved {
            String::from_str(&env, "approved")
        } else {
            String::from_str(&env, "rejected")
        };

        claims.set(key, (description, status));
        env.storage().instance().set(&"claims", &claims);
    }

    /// Get product info: returns (manufacturer, warranty_months)
    pub fn get_product(env: Env, product_id: u64) -> (Address, u64) {
        let products: Map<u64, (Address, u64)> = env
            .storage()
            .instance()
            .get(&"products")
            .unwrap_or(Map::new(&env));

        products.get(product_id).unwrap_or_else(|| {
            panic!("Product not found");
        })
    }

    /// Get all claims for a product: returns list of ((description, status), claim_index)
    pub fn get_claims(env: Env, product_id: u64) -> Vec<(u32, String, String)> {
        let claims: Map<(u64, u32), (String, String)> = env
            .storage()
            .instance()
            .get(&"claims")
            .unwrap_or(Map::new(&env));

        let mut result: Vec<(u32, String, String)> = Vec::new(&env);

        for ((pid, idx), (desc, status)) in claims.iter() {
            if pid == product_id {
                result.push_back((idx, desc, status));
            }
        }

        result
    }
}
