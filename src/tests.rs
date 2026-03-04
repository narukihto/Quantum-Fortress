import requests
import concurrent.futures
import time
import random
from collections import Counter

# Configuration - Matches Port 3000 and the updated Route
URL = "http://localhost:3000/v1/quantum-verify"
THREADS = 50 
TOTAL_REQUESTS = 1000

def generate_secure_payload(is_replay=False, nonce_id=""):
    # Must have high entropy (>4.5) to pass entropy.rs
    high_entropy_data = [random.randint(0, 255) for _ in range(256)]
    
    return {
        # If is_replay is true, we use a fixed nonce to trigger the moka cache
        "nonce": "ATTACK_REPLAY_TOKEN_X99" if is_replay else f"unique_{nonce_id}_{time.time_ns()}",
        "data": high_entropy_data,
        "signature": [0] * 2420,  # Valid size for Dilithium2
        "public_key": [0] * 1312   # Valid size for Dilithium2
    }

def attack_vector(is_replay=False, idx=0):
    payload = generate_secure_payload(is_replay, idx)
    try:
        # We expect 200/400 for unique (depending on sig) 
        # but specifically looking for "error" messages for replays
        response = requests.post(URL, json=payload, timeout=5)
        
        # Checking body for specific "Replay attack detected" message
        if "Replay" in response.text:
            return 409 # Conflict/Blocked
        return response.status_code
    except:
        return "CONNECTION_ERROR"

print(f"🚀 Initializing Quantum Stress Test on {URL}...")
print(f"📡 Testing with {TOTAL_REQUESTS} requests ({THREADS} workers)...")



start_time = time.perf_counter()

with concurrent.futures.ThreadPoolExecutor(max_workers=THREADS) as executor:
    # First 500: Unique nonces
    # Last 500: Identical nonces (Replay Attack)
    tasks = [executor.submit(attack_vector, is_replay=(i >= 500), idx=i) for i in range(TOTAL_REQUESTS)]
    results = [t.result() for t in concurrent.futures.as_completed(tasks)]

duration = time.perf_counter() - start_time
stats = Counter(results)

print("\n" + "="*40)
print("📊 SECURITY PEN-TEST SUMMARY")
print("="*40)
print(f"⏱️ Duration: {duration:.2f}s")
print(f"✅ Unique Requests: {stats[200] + stats[400]}") # 400 is common due to mock sig
print(f"🛡️ Replays Blocked: {stats[409]}")
print("="*40)

if stats[409] > 0:
    print("🏆 SUCCESS: The Sentinel detected and blocked the replay flood.")
else:
    print("❌ FAILURE: Replay attacks were not detected.")
