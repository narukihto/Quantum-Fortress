import requests
import concurrent.futures
import time
import random
from collections import Counter

URL = "http://localhost:3000/v1/quantum-verify"
THREADS = 50 
TOTAL_REQUESTS = 1000

def generate_secure_payload(is_replay=False, nonce_id=""):
    high_entropy_data = [random.randint(0, 255) for _ in range(256)]
    return {
        "nonce": "ATTACK_REPLAY_TOKEN_X99" if is_replay else f"unique_{nonce_id}_{time.time_ns()}",
        "data": high_entropy_data,
        "signature": [0] * 2420,
        "public_key": [0] * 1312
    }

def attack_vector(is_replay=False, idx=0):
    payload = generate_secure_payload(is_replay, idx)
    try:
        response = requests.post(URL, json=payload, timeout=5)
        return response.status_code
    except:
        return "CONNECTION_ERROR"

print(f"🚀 Initializing Stress Test on {URL}...")
with concurrent.futures.ThreadPoolExecutor(max_workers=THREADS) as executor:
    tasks = [executor.submit(attack_vector, is_replay=(i >= 500), idx=i) for i in range(TOTAL_REQUESTS)]
    results = [t.result() for t in concurrent.futures.as_completed(tasks)]

stats = Counter(results)
print(f"✅ Unique Requests Processed: {stats[200] + stats[400]}")
print(f"🛡️ Replays Blocked: {stats[409] if 409 in stats else 'Check Dashboard'}")
