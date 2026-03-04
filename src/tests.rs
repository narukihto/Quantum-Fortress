import requests
import concurrent.futures
import time
from collections import Counter

# Configuration
URL = "http://localhost:3000/v1/quantum-verify"
THREADS = 50  # Simultaneous attackers
TOTAL_REQUESTS = 1000

# Mock PQC Payload (Dilithium2 specs)
payload_template = {
    "nonce": "static_replay_nonce_999",
    "data": "SECURE_TRANSACTION_DATA_001",
    "signature": [0] * 2420,  # Simulated ML-DSA Sig
    "public_key": [0] * 1312  # Simulated ML-DSA PK
}

def attack_vector(is_replay=False):
    data = payload_template.copy()
    # If not a replay, generate a unique cryptographically-spaced nonce
    if not is_replay:
        data["nonce"] = f"unique_{time.time_ns()}"
    
    try:
        response = requests.post(URL, json=data, timeout=5)
        return response.status_code
    except Exception as e:
        return "CONNECTION_ERROR"

print(f"🚀 Initializing Stress Test on {URL}...")
print(f"📡 Flooding with {TOTAL_REQUESTS} requests using {THREADS} parallel workers...")

start_time = time.perf_counter()

with concurrent.futures.ThreadPoolExecutor(max_workers=THREADS) as executor:
    # 500 Unique requests (Should be 200 OK)
    # 500 Replay requests (Should be 409 Conflict)
    tasks = [executor.submit(attack_vector, is_replay=(i >= 500)) for i in range(TOTAL_REQUESTS)]
    results = [t.result() for t in concurrent.futures.as_completed(tasks)]

duration = time.perf_counter() - start_time
stats = Counter(results)

print("\n" + "="*30)
print("📊 PEN-TEST RESULTS")
print("="*30)
print(f"⏱️ Total Duration:  {duration:.2f} seconds")
print(f"✅ Successful (200): {stats[200]}")
print(f"🛡️ Blocked Replay (409): {stats[409]}")
print(f"❌ Other/Errors:     {stats['CONNECTION_ERROR']}")
print("="*30)

if stats[409] == 500:
    print("🏆 VERDICT: FORTRESS IS IMPENETRABLE. All replay attacks mitigated.")
else:
    print("⚠️ VERDICT: VULNERABILITY DETECTED. Some replays bypassed the cache.")
