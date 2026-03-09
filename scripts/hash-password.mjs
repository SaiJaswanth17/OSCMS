// Quick script to generate an Argon2id hash compatible with the Rust argon2 crate
import { hash } from 'argon2';

const password = 'Admin@1234';
const h = await hash(password, {
  type: 2, // Argon2id
  memoryCost: 65536,
  timeCost: 2,
  parallelism: 1,
});
console.log(h);
