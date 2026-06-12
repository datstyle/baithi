// lib.rs - PC Rental Management Smart Contract (Soroban / Stellar)

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype,
    Address, Env, String, Vec,
    log,
};

// =====================
//  DATA STRUCTURES
// =====================

#[contracttype]
#[derive(Clone, Debug)]
pub struct PC {
    pub id: u64,
    pub name: String,           // e.g. "PC-01", "Gaming Rig A"
    pub specs: String,          // e.g. "RTX 4090, i9-13900K, 32GB RAM"
    pub price_per_hour: i128,   // in stroops (1 XLM = 10_000_000 stroops)
    pub is_available: bool,
    pub owner: Address,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Rental {
    pub id: u64,
    pub pc_id: u64,
    pub renter: Address,
    pub start_time: u64,        // ledger timestamp
    pub end_time: u64,          // estimated end time
    pub total_price: i128,      // pre-calculated total
    pub status: RentalStatus,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum RentalStatus {
    Active,
    Completed,
    Cancelled,
}

// Storage keys
#[contracttype]
pub enum DataKey {
    Admin,
    PCCount,
    RentalCount,
    PC(u64),
    Rental(u64),
    RenterHistory(Address),     // list of rental IDs for a renter
    PCRentals(u64),             // list of rental IDs for a PC
}

// =====================
//  CONTRACT
// =====================

#[contract]
pub struct PCRentalManagement;

#[contractimpl]
impl PCRentalManagement {

    // =====================
    //  INIT
    // =====================

    /// Initialize contract, set admin
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::PCCount, &0u64);
        env.storage().instance().set(&DataKey::RentalCount, &0u64);
        log!(&env, "Contract initialized. Admin: {}", admin);
    }

    // =====================
    //  ADMIN
    // =====================

    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    pub fn transfer_admin(env: Env, new_admin: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &new_admin);
        log!(&env, "Admin transferred to {}", new_admin);
    }

    // =====================
    //  PC MANAGEMENT
    // =====================

    /// Register a new PC (admin only)
    pub fn register_pc(
        env: Env,
        name: String,
        specs: String,
        price_per_hour: i128,
    ) -> u64 {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        if price_per_hour <= 0 {
            panic!("Price per hour must be greater than 0");
        }

        let pc_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PCCount)
            .unwrap_or(0);

        let new_id = pc_count + 1;

        let pc = PC {
            id: new_id,
            name,
            specs,
            price_per_hour,
            is_available: true,
            owner: admin.clone(),
        };

        env.storage().persistent().set(&DataKey::PC(new_id), &pc);
        env.storage().instance().set(&DataKey::PCCount, &new_id);

        // Init empty rental list for this PC
        let empty: Vec<u64> = Vec::new(&env);
        env.storage()
            .persistent()
            .set(&DataKey::PCRentals(new_id), &empty);

        log!(&env, "PC registered. ID: {}", new_id);
        new_id
    }

    /// Update PC info (admin only)
    pub fn update_pc(
        env: Env,
        pc_id: u64,
        name: Option<String>,
        specs: Option<String>,
        price_per_hour: Option<i128>,
    ) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let mut pc: PC = Self::get_pc_or_panic(&env, pc_id);

        if let Some(n) = name {
            pc.name = n;
        }
        if let Some(s) = specs {
            pc.specs = s;
        }
        if let Some(p) = price_per_hour {
            if p <= 0 {
                panic!("Price per hour must be greater than 0");
            }
            pc.price_per_hour = p;
        }

        env.storage().persistent().set(&DataKey::PC(pc_id), &pc);
        log!(&env, "PC {} updated", pc_id);
    }

    /// Set PC availability manually (admin only)
    pub fn set_pc_availability(env: Env, pc_id: u64, available: bool) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let mut pc: PC = Self::get_pc_or_panic(&env, pc_id);
        pc.is_available = available;
        env.storage().persistent().set(&DataKey::PC(pc_id), &pc);
        log!(&env, "PC {} availability set to {}", pc_id, available);
    }

    /// Remove a PC from system (admin only, only if not currently rented)
    pub fn remove_pc(env: Env, pc_id: u64) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let pc: PC = Self::get_pc_or_panic(&env, pc_id);
        if !pc.is_available {
            panic!("Cannot remove a PC that is currently being rented");
        }

        env.storage().persistent().remove(&DataKey::PC(pc_id));
        log!(&env, "PC {} removed", pc_id);
    }

    // =====================
    //  RENTAL MANAGEMENT
    // =====================

    /// Rent a PC — renter specifies how many hours they want
    pub fn rent_pc(
        env: Env,
        renter: Address,
        pc_id: u64,
        duration_hours: u64,
    ) -> u64 {
        renter.require_auth();

        if duration_hours == 0 {
            panic!("Duration must be at least 1 hour");
        }

        let mut pc: PC = Self::get_pc_or_panic(&env, pc_id);

        if !pc.is_available {
            panic!("PC is currently not available");
        }

        let rental_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::RentalCount)
            .unwrap_or(0);

        let new_rental_id = rental_count + 1;
        let now = env.ledger().timestamp();
        let end_time = now + duration_hours * 3600;
        let total_price = pc.price_per_hour * duration_hours as i128;

        let rental = Rental {
            id: new_rental_id,
            pc_id,
            renter: renter.clone(),
            start_time: now,
            end_time,
            total_price,
            status: RentalStatus::Active,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Rental(new_rental_id), &rental);
        env.storage()
            .instance()
            .set(&DataKey::RentalCount, &new_rental_id);

        // Mark PC as unavailable
        pc.is_available = false;
        env.storage().persistent().set(&DataKey::PC(pc_id), &pc);

        // Update renter history
        let mut renter_history: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::RenterHistory(renter.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        renter_history.push_back(new_rental_id);
        env.storage()
            .persistent()
            .set(&DataKey::RenterHistory(renter.clone()), &renter_history);

        // Update PC rental list
        let mut pc_rentals: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::PCRentals(pc_id))
            .unwrap_or_else(|| Vec::new(&env));
        pc_rentals.push_back(new_rental_id);
        env.storage()
            .persistent()
            .set(&DataKey::PCRentals(pc_id), &pc_rentals);

        log!(&env, "PC {} rented. Rental ID: {}", pc_id, new_rental_id);
        new_rental_id
    }

    /// Return a PC — marks rental as completed and frees the PC
    pub fn return_pc(env: Env, renter: Address, rental_id: u64) {
        renter.require_auth();

        let mut rental: Rental = Self::get_rental_or_panic(&env, rental_id);

        if rental.renter != renter {
            panic!("Only the renter can return this PC");
        }
        if rental.status != RentalStatus::Active {
            panic!("Rental is not active");
        }

        rental.status = RentalStatus::Completed;
        rental.end_time = env.ledger().timestamp(); // actual return time
        env.storage()
            .persistent()
            .set(&DataKey::Rental(rental_id), &rental);

        // Free up the PC
        let mut pc: PC = Self::get_pc_or_panic(&env, rental.pc_id);
        pc.is_available = true;
        env.storage()
            .persistent()
            .set(&DataKey::PC(rental.pc_id), &pc);

        log!(&env, "Rental {} completed. PC {} is now available", rental_id, rental.pc_id);
    }

    /// Cancel a rental (renter or admin)
    pub fn cancel_rental(env: Env, caller: Address, rental_id: u64) {
        caller.require_auth();

        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        let mut rental: Rental = Self::get_rental_or_panic(&env, rental_id);

        if caller != admin && caller != rental.renter {
            panic!("Only the renter or admin can cancel this rental");
        }
        if rental.status != RentalStatus::Active {
            panic!("Rental is not active");
        }

        rental.status = RentalStatus::Cancelled;
        env.storage()
            .persistent()
            .set(&DataKey::Rental(rental_id), &rental);

        // Free up the PC
        let mut pc: PC = Self::get_pc_or_panic(&env, rental.pc_id);
        pc.is_available = true;
        env.storage()
            .persistent()
            .set(&DataKey::PC(rental.pc_id), &pc);

        log!(&env, "Rental {} cancelled. PC {} is now available", rental_id, rental.pc_id);
    }

    // =====================
    //  QUERIES
    // =====================

    pub fn get_pc(env: Env, pc_id: u64) -> PC {
        Self::get_pc_or_panic(&env, pc_id)
    }

    pub fn get_pc_count(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::PCCount).unwrap_or(0)
    }

    pub fn get_rental(env: Env, rental_id: u64) -> Rental {
        Self::get_rental_or_panic(&env, rental_id)
    }

    pub fn get_rental_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::RentalCount)
            .unwrap_or(0)
    }

    pub fn is_available(env: Env, pc_id: u64) -> bool {
        Self::get_pc_or_panic(&env, pc_id).is_available
    }

    /// Get all rental IDs of a renter
    pub fn get_renter_history(env: Env, renter: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::RenterHistory(renter))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Get all rental IDs of a PC
    pub fn get_pc_rentals(env: Env, pc_id: u64) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::PCRentals(pc_id))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Calculate total price for a given duration (read-only helper)
    pub fn calculate_price(env: Env, pc_id: u64, duration_hours: u64) -> i128 {
        let pc: PC = Self::get_pc_or_panic(&env, pc_id);
        pc.price_per_hour * duration_hours as i128
    }

    // =====================
    //  PRIVATE HELPERS
    // =====================

    fn get_pc_or_panic(env: &Env, pc_id: u64) -> PC {
        env.storage()
            .persistent()
            .get(&DataKey::PC(pc_id))
            .unwrap_or_else(|| panic!("PC not found"))
    }

    fn get_rental_or_panic(env: &Env, rental_id: u64) -> Rental {
        env.storage()
            .persistent()
            .get(&DataKey::Rental(rental_id))
            .unwrap_or_else(|| panic!("Rental not found"))
    }
}

// =====================
//  TESTS
// =====================

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    #[test]
    fn test_full_rental_flow() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PCRentalManagement);
        let client = PCRentalManagementClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let renter = Address::generate(&env);

        // Init
        client.initialize(&admin);
        assert_eq!(client.get_admin(), admin);

        // Register PC
        let pc_id = client.register_pc(
            &String::from_str(&env, "Gaming Rig A"),
            &String::from_str(&env, "RTX 4090, i9-13900K, 32GB RAM"),
            &5_000_000i128, // 0.5 XLM/hour
        );
        assert_eq!(pc_id, 1);
        assert!(client.is_available(&pc_id));

        // Check price calculation
        let price = client.calculate_price(&pc_id, &3u64);
        assert_eq!(price, 15_000_000i128); // 1.5 XLM for 3 hours

        // Rent PC
        let rental_id = client.rent_pc(&renter, &pc_id, &3u64);
        assert_eq!(rental_id, 1);
        assert!(!client.is_available(&pc_id)); // PC is now occupied

        // Check rental details
        let rental = client.get_rental(&rental_id);
        assert_eq!(rental.status, RentalStatus::Active);
        assert_eq!(rental.total_price, 15_000_000i128);

        // Return PC
        client.return_pc(&renter, &rental_id);
        assert!(client.is_available(&pc_id)); // PC is free again

        let completed = client.get_rental(&rental_id);
        assert_eq!(completed.status, RentalStatus::Completed);
    }

    #[test]
    #[should_panic(expected = "PC is currently not available")]
    fn test_rent_unavailable_pc() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PCRentalManagement);
        let client = PCRentalManagementClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let renter1 = Address::generate(&env);
        let renter2 = Address::generate(&env);

        client.initialize(&admin);
        let pc_id = client.register_pc(
            &String::from_str(&env, "PC-01"),
            &String::from_str(&env, "GTX 1080, i7, 16GB"),
            &3_000_000i128,
        );

        client.rent_pc(&renter1, &pc_id, &2u64);
        client.rent_pc(&renter2, &pc_id, &1u64); // should panic
    }

    #[test]
    fn test_cancel_rental() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PCRentalManagement);
        let client = PCRentalManagementClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let renter = Address::generate(&env);

        client.initialize(&admin);
        let pc_id = client.register_pc(
            &String::from_str(&env, "PC-02"),
            &String::from_str(&env, "RX 7900 XT, Ryzen 9, 32GB"),
            &4_000_000i128,
        );

        let rental_id = client.rent_pc(&renter, &pc_id, &5u64);
        assert!(!client.is_available(&pc_id));

        // Admin cancels the rental
        client.cancel_rental(&admin, &rental_id);
        assert!(client.is_available(&pc_id));

        let rental = client.get_rental(&rental_id);
        assert_eq!(rental.status, RentalStatus::Cancelled);
    }
}