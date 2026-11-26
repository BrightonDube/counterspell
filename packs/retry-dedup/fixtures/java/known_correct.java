// Test-only known-correct Java fixture for retry-dedup.
public class Solution {
    public static long retry_dedup(long[][] log) {
        java.util.Set<Long> seen = new java.util.HashSet<>();
        for (long[] row : log) seen.add(row[1]);
        return seen.size();
    }
}
