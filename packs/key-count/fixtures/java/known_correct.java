// Test-only known-correct Java fixture for key-count.
public class Solution {
    public static long key_count(long[][] rows) {
        java.util.Set<Long> keys = new java.util.HashSet<>();
        for (long[] r : rows) {
            keys.add(r[0]);
        }
        return keys.size();
    }
}
