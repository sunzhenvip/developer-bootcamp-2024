const { Keypair } = require('@solana/web3.js');
const bs58 = require('bs58');
const fs = require('fs');

// Path to your id.json file
const keypairPath = '/home/sz/.config/solana/id.json'; // Adjust as needed

try {
    // Read the keypair from the JSON file
    const secretKeyJson = JSON.parse(fs.readFileSync(keypairPath, 'utf8'));

    // Create a Keypair object from the secret key array
    const keypair = Keypair.fromSecretKey(Uint8Array.from(secretKeyJson));

    // Get the Base58 encoded private key
    const base58PrivateKey = bs58.encode(keypair.secretKey);

    console.log('Base58 Private Key:', base58PrivateKey);
} catch (error) {
    console.error('Error converting keypair:', error);
}