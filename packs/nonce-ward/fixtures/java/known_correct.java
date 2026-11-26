// Test-only known-correct Java fixture for nonce-ward.
public class Solution {
    public static long nonce_ward(long[][] msgs) {
        java.util.Map<Long, Long> high = new java.util.HashMap<>();
        long replays = 0;
        for (long[] m : msgs) {
            long mark = high.getOrDefault(m[0], -1L);
            if (m[1] <= mark) replays++; else high.put(m[0], m[1]);
        }
        return replays;
    }
}
