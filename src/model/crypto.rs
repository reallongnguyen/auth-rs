use nanoid::nanoid;

// const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
//                         abcdefghijklmnopqrstuvwxyz\
//                         0123456789";
// pub const TOKEN_LEN: usize = 32;

// pub fn create_secure_token(token_len: usize) -> String {
//   let mut rng = rand::thread_rng();

//   (0..token_len)
//     .map(|_| {
//       let idx = rng.gen_range(0..CHARSET.len());
//       CHARSET[idx] as char
//     })
//     .collect()
// }

pub fn create_unique_token() -> String {
  nanoid!()
}
