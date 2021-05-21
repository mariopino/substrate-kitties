#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, ensure, StorageValue, StorageDoubleMap,
	traits::Randomness, RuntimeDebug
};
use sp_io::hashing::blake2_128;
use frame_system::ensure_signed;

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub struct Kitty(pub [u8; 16]);

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
		/// Stores all the kitties, key is the kitty id
		pub Kitties get(fn kitties): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) u32 => Option<Kitty>;
		/// Stores the next kitty id
		pub NextKittyId get(fn next_kitty_id): u32;
		KittyOwner get(fn owner_of): map hasher(blake2_128_concat) T::Hash => Option<T::AccountId>;
	}
}

decl_event! {
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
	{
		/// A kitty is created, \[owner, kitty_id, kitty\]
		KittyCreated(AccountId, u32, Kitty),
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// A kitty is created, \[owner, kitty_id, kitty\]
		KittiesIdOverflow,
		InvalidKittyId,
		RequireDifferentParent
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		fn generate_dna(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				<pallet_randomness_collective_flip::Module<T> as Randomness<T::Hash>>::random_seed(),
				&sender,
				<frame_system::Module<T>>::extrinsic_index(),
			);
			let dna = payload.using_encoded(blake2_128);
			return dna
		}

		fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
			(selector & dna1) | (!selector & dna2)
		}

		// #[weight = 1000]
		pub fn create(origin) {
			let sender = ensure_signed(origin)?;

			// ensure kitty id does not overflow
			let kitty_id = Self::next_kitty_id();
			match kitty_id.checked_add(1) {
				Some(next_kitty_id) => {
					// Generate dna
					let dna = Self::generate_dna(&sender);

					// Get gender based on first u8 item of dna array, odd => male, even => female
					let gender;
					if dna[0]&1 != 0 {
						gender = "male";
					} else {
						gender = "female";
					}
					frame_support::debug::native::debug!("gender {:?}", gender);

					// Create and store kitty and next kitty id
					let kitty = Kitty(dna);
					<Kitties<T>>::insert(&sender, kitty_id, kitty.clone());
					NextKittyId::put(next_kitty_id);

					// Emit event
					Self::deposit_event(RawEvent::KittyCreated(sender, kitty_id, kitty))
				}
				None => {
					// Overflow!
					return Err(Error::<T>::KittiesIdOverflow.into());
				}
			};
		}

		// #[weight = 1000]
		pub fn breed(origin, kitty_id_1: u32, kitty_id_2: u32) {
			let sender = ensure_signed(origin)?;


			// Ensure that sender is the owner of both kitties
			let kitty1 = Self::kitties(sender, kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
			let kitty2 = Self::kitties(sender, kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

			// Require different parents
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::RequireDifferentParent);

			// TODO: Require different gender parents

			let kitty_id = Self::next_kitty_id();

			let kitty1_dna = kitty1.0;
			let kitty2_dna = kitty2.0;

			// Generate a random 128bit value
			let dna = Self::generate_dna(&sender);
			let mut new_dna = [0u8; 16];

			// Combine parents and selector to create new kitty
			for i in 0..kitty1_dna.len() {
				new_dna[i] = combine_dna(kitty1_dna[i], kitty2_dna[i], dna[i]);
			}

			let kitty = Kitty(new_dna);
			<Kitties<T>>::insert(&sender, kitty_id, kitty.clone());

		}
	}
}