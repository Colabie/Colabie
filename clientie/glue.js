export async function get_raw(url) {
  const response = await fetch(url);
  return new Uint8Array(await response.arrayBuffer());
}

export async function post_raw(url, body) {
  const response = await fetch(url, {
    "method": "POST",
    "body": body,
  });
  return new Uint8Array(await response.arrayBuffer());
}

export function save_raw(key, value) {
  localStorage.setItem(key, JSON.stringify(value));
}

export function load_raw(key) {
  let map = JSON.parse(localStorage.getItem(key))
  return new Uint8Array(Object.keys(map).map((i) => map[i]))
}

// Security improvements for storing sensitive key material
// Uses IndexedDB with Web Crypto API for better security
let db = null;

// Initialize the IndexedDB database
async function initSecureDb() {
  return new Promise((resolve, reject) => {
    if (db) {
      resolve(db);
      return;
    }
    
    const request = indexedDB.open("SecureStorage", 1);
    
    request.onerror = event => {
      reject("Failed to open secure database: " + event.target.errorCode);
    };
    
    request.onupgradeneeded = event => {
      const db = event.target.result;
      if (!db.objectStoreNames.contains('secureKeys')) {
        db.createObjectStore('secureKeys');
      }
    };
    
    request.onsuccess = event => {
      db = event.target.result;
      resolve(db);
    };
  });
}

// Encrypt the data before storing
async function encryptData(data) {
  // Generate a random key for encryption
  const key = await window.crypto.subtle.generateKey(
    {
      name: "AES-GCM",
      length: 256
    },
    true,
    ["encrypt", "decrypt"]
  );
  
  // Convert the data to an ArrayBuffer
  const dataBuffer = new Uint8Array(data).buffer;
  
  // Generate a random IV
  const iv = window.crypto.getRandomValues(new Uint8Array(12));
  
  // Encrypt the data
  const encryptedData = await window.crypto.subtle.encrypt(
    {
      name: "AES-GCM",
      iv
    },
    key,
    dataBuffer
  );
  
  // Export the key to store it
  const exportedKey = await window.crypto.subtle.exportKey("raw", key);
  
  // Return the encrypted data, the IV, and the exported key
  return {
    encryptedData,
    iv,
    exportedKey
  };
}

// Decrypt the stored data
async function decryptData(encryptedData, iv, exportedKey) {
  // Import the key
  const key = await window.crypto.subtle.importKey(
    "raw",
    exportedKey,
    {
      name: "AES-GCM",
      length: 256
    },
    false,
    ["decrypt"]
  );
  
  // Decrypt the data
  const decryptedData = await window.crypto.subtle.decrypt(
    {
      name: "AES-GCM",
      iv
    },
    key,
    encryptedData
  );
  
  return new Uint8Array(decryptedData);
}

// Save data securely
export async function save_secure(key, value) {
  try {
    const database = await initSecureDb();
    const { encryptedData, iv, exportedKey } = await encryptData(value);
    
    return new Promise((resolve, reject) => {
      const transaction = database.transaction(['secureKeys'], 'readwrite');
      const store = transaction.objectStore('secureKeys');
      
      // Store the encrypted data, IV, and key
      const storeData = {
        encryptedData: Array.from(new Uint8Array(encryptedData)),
        iv: Array.from(iv),
        exportedKey: Array.from(new Uint8Array(exportedKey))
      };
      
      const request = store.put(storeData, key);
      
      request.onsuccess = () => resolve();
      request.onerror = () => reject(new Error("Failed to save secure data"));
    });
  } catch (error) {
    console.error("Error saving secure data:", error);
    throw new Error("Failed to save secure data: " + error.message);
  }
}

// Load data securely
export async function load_secure(key) {
  try {
    const database = await initSecureDb();
    
    return new Promise((resolve, reject) => {
      const transaction = database.transaction(['secureKeys'], 'readonly');
      const store = transaction.objectStore('secureKeys');
      const request = store.get(key);
      
      request.onsuccess = async () => {
        if (!request.result) {
          reject(new Error("Key not found"));
          return;
        }
        
        const { encryptedData, iv, exportedKey } = request.result;
        
        // Convert back to Uint8Array
        const encryptedBuffer = new Uint8Array(encryptedData).buffer;
        const ivBuffer = new Uint8Array(iv);
        const keyBuffer = new Uint8Array(exportedKey).buffer;
        
        try {
          const decryptedData = await decryptData(encryptedBuffer, ivBuffer, keyBuffer);
          resolve(decryptedData);
        } catch (error) {
          reject(new Error("Failed to decrypt data: " + error.message));
        }
      };
      
      request.onerror = () => reject(new Error("Failed to load secure data"));
    });
  } catch (error) {
    console.error("Error loading secure data:", error);
    throw new Error("Failed to load secure data: " + error.message);
  }
}

