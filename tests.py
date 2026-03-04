import requests
import time
import random

# Configuration
BASE_URL = "http://localhost:3000"
VERIFY_ENDPOINT = f"{BASE_URL}/v1/quantum-verify"
STATS_ENDPOINT = f"{BASE_URL}/api/stats"

def test_quantum_integrity():
    print("🚀 Starting Advanced Fortress Validation...")

    # 1. Test Valid Request (Simulating Dilithium2 data structure)
    print("\n[1] Testing Valid Quantum Payload...")
    payload = {
        "nonce": f"unique_{random.randint(1000, 9999)}",
        "data": [random.randint(0, 255) for _ in range(32)], # High entropy
        "signature": [0] * 2420, # Simulated signature size
        "public_key": [0] * 1312  # Simulated public key size
    }
    response = requests.post(VERIFY_ENDPOINT, json=payload)
    print(f"Status: {response.status_code}, Response: {response.json()}")

    # 2. Test Replay Attack (Sending the same nonce twice)
    print("\n[2] Testing Replay Attack Defense...")
    print("Sending same nonce again...")
    response = requests.post(VERIFY_ENDPOINT, json=payload)
    if response.status_code == 403:
        print("✅ SUCCESS: Replay attack blocked with 403 Forbidden.")
    else:
        print(f"❌ FAILED: Replay attack returned {response.status_code}")

    # 3. Test Low Entropy Attack (Predictable data)
    print("\n[3] Testing Low Entropy Defense...")
    bad_payload = payload.copy()
    bad_payload["nonce"] = "new_nonce_123"
    bad_payload["data"] = [0] * 32 # Totally predictable zeros
    response = requests.post(VERIFY_ENDPOINT, json=bad_payload)
    if response.status_code == 400:
        print("✅ SUCCESS: Low entropy data blocked with 400 Bad Request.")
    else:
        print(f"❌ FAILED: System accepted low entropy data.")

    # 4. Test Rate Limiting (Spamming requests)
    print("\n[4] Testing Rate Limiter (Spamming 20 requests)...")
    for i in range(20):
        requests.post(VERIFY_ENDPOINT, json=payload)
    
    # The last request should likely be blocked or handled by tower-governor
    print("Rate limit test finished. Check terminal for 'Too Many Requests' errors.")

    # 5. Final Stats Check
    print("\n[5] Fetching Live SOC Stats...")
    stats = requests.get(STATS_ENDPOINT).json()
    print(f"📊 Live Stats: {stats}")

if __name__ == "__main__":
    try:
        test_quantum_integrity()
    except Exception as e:
        print(f"❌ Connectivity Error: Is the server running? {e}")
