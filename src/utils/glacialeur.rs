use num_traits::cast::ToPrimitive;
use sha3::{Digest, Sha3_256};

/*
	Glacialeur
	Undefined identification number
	<bot version>
	1 => None
	2 => Silence
	4 => Avespoir
	8 => Estella
*/

fn rotr16 (value: u16, shift: u16) -> u16 {
	(value >> shift) | (value << (16 - shift))
}

fn rotl16 (value: u16, shift: u16) -> u16 {
	(value << shift) | (value >> (16 - shift))
}

fn rotr8 (value: u8, shift: u8) -> u8 {
	(value >> shift) | (value << (8 - shift))
}

fn rotl8 (value: u8, shift: u8) -> u8 {
	(value << shift) | (value >> (8 - shift))
}

fn nxor16 (x: u16, y: u16) -> u16 {
	!(x ^ y)
}

fn nxor8 (x: u8, y: u8) -> u8 {
	!(x ^ y)
}

fn before_hash_id_to_bytes(id: u128) -> Vec<u8> {
	let mut res = Vec::<u8>::new();
	for i in (0..14).rev() {
		res.push( ((id & (0xff << (i * 8))) >> (i * 8)).to_u8().unwrap());
	}

	return res;
}

fn id_to_hash(id: u128) -> Vec<u16> {
	let mut hasher = Sha3_256::new();
	hasher.update(before_hash_id_to_bytes(id));
	let hash = hasher.finalize().to_vec();
	let mut hash_block = Vec::<u16>::new();
	for i in (0..hash.len()).step_by(2) {
		hash_block.push(
			(hash[i].to_u16().unwrap() << 8) | hash[i + 1].to_u16().unwrap()
		);
	}
	return hash_block;
}

fn gen_check_hash(id: u128) -> u16 {
	let rb = 16;
	let mut block = id_to_hash(id);

	for i in 0..block.len() {
		block[i] = nxor16(
			rotr16(block[i], rb / 2),
			rotl16(block[i], rb / 4)) ^ (block[i] >> ((rb / 8) * 2)
		);
		block[i] = nxor16(
			rotl16(block[i], rb / 2) ^ rotr16(block[i], rb / 4),
			block[i] << ((rb / 8) * 2)
		);
	}

	let mut r: u16 = 0;

	for _ in 0..rb {
		r <<= 1;
		r += 1;
	}
	r = ((block[0] & (r >> (rb / 2))) << (rb / 2)) | (((!block[0]) & (r << (rb / 2))) >> (rb / 2));
	for i in 1..block.len() {
		r = (((block[i] & (r >> (rb / 2))) << (rb / 2)) | (((!block[i]) & (r << (rb / 2))) >> (rb / 2))) ^ block[i - 1];
	}

	return r;
}

fn gen_comp_disco_id(id: &u64) -> u8 {
	let rb = 8;
	let mut block = Vec::<u8>::new();

	for i in (0..(64 / 8)).rev() {
		block.push(((id & (0xFF << (i * 8))) >> (i * 8)).to_u8().unwrap());
	}

	for i in 0..block.len() {
		block[i] = nxor8(
			rotr8(block[i], rb / 2),
			rotl8(block[i], rb / 4)) ^ (block[i] >> ((rb / 8) * 2)
		);
		block[i] = nxor8(
			rotl8(block[i], rb / 2) ^ rotr8(block[i], rb / 4),
			block[i] << ((rb / 8) * 2)
		);
	}

	let mut r: u8 = 0;

	for _ in 0..rb {
		r <<= 1;
		r += 1;
	}
	r = ((block[0] & (r >> (rb / 2))) << (rb / 2)) | (((!block[0]) & (r << (rb / 2))) >> (rb / 2));
	for i in 1..block.len() {
		r = (((block[i] & (r >> (rb / 2))) << (rb / 2)) | (((!block[i]) & (r << (rb / 2))) >> (rb / 2))) ^ block[i - 1];
	}

	return r;
}

fn to_string_36(id: u64) -> String {
	let mut result = String::new();

	let mut num: u64 = *id;
	while num != 0 {
		let num_mod = num % 36;
		if num_mod >= 10 {
			result.push(
				std::char::from_u32(
					('A' as u32) + num_mod.to_u32().unwrap() - 10
				).unwrap()
			);
		}
		else {
			result.push(std::char::from_digit(num_mod.to_u32().unwrap(), 10).unwrap());
		}
		num /= 36;
	}

	return result.chars().rev().collect();
}

pub fn generate(discord_id_: &u64, version: u8, join_unixtime_: i64) -> String {
	let discord_id = *discord_id_;
	let join_unixtime = join_unixtime_.to_u32().unwrap();
	let comp_discord_id = gen_comp_disco_id(&discord_id);

	let mut check_hash_id: u128 = comp_discord_id.to_u128().unwrap() << 120;
	check_hash_id |= discord_id.to_u128().unwrap() << 56;
	check_hash_id |= join_unixtime.to_u128().unwrap() << 24;
	check_hash_id |= version.to_u128().unwrap() << 16;
	check_hash_id >>= 2 * 8;

	let check_hash = gen_check_hash(check_hash_id);

	let mut result_id: u64 = comp_discord_id.to_u64().unwrap() << 56;
	result_id |= join_unixtime.to_u64().unwrap() << 24;
	result_id |= version.to_u64().unwrap() << 16;
	result_id |= check_hash.to_u64().unwrap();

	result_id ^= discord_id;

	return to_string_36(result_id);
}
