// Test-only known-correct Java fixture for rate-gate.
public class Solution {
    public static long rate_gate(long[][] events, long capacity, long refill) {
        long bucket = capacity, last = 0, passed = 0;
        for (long[] e : events) {
            bucket = Math.min(capacity, bucket + (e[0] - last) * refill);
            last = e[0];
            if (e[1] <= bucket) { bucket -= e[1]; passed++; }
        }
        return passed;
    }
}
