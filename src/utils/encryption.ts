// src/utils/encryption.ts
export function encrypt(text: string): string {
  console.log('Encrypting:', text);
  return `encrypted(${text})`;
}

export function decrypt(encryptedText: string): string {
  console.log('Decrypting:', encryptedText);
  return `decrypted(${encryptedText})`;
}