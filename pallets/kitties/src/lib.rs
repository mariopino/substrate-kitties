#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, StorageValue, StorageDoubleMap,
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
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		#[weight = 1000]
		pub fn create(origin) {
			let sender = ensure_signed(origin)?;

			// ensure kitty id does not overflow
			let kitty_id = Self::next_kitty_id();
			match kitty_id.checked_add(1) {
				Some(next_kitty_id) => {
					// Generate a random 128bit value
					let payload = (
						<pallet_randomness_collective_flip::Module<T> as Randomness<T::Hash>>::random_seed(),
						&sender,
						<frame_system::Module<T>>::extrinsic_index(),
					);
					let dna = payload.using_encoded(blake2_128);

					// Get gender based on first byte of dna, odd => male, even => female
					let gender = dna[0];
					println!("gender: {}", gender);

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
	}
}