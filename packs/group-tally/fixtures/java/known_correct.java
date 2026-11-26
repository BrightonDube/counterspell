// Test-only known-correct Java fixture for group-tally.
public class Solution {
    public static long group_tally(long[][] rows) {
        java.util.Map<Long, Long> tally = new java.util.HashMap<>();
        for (long[] r : rows) {
            tally.merge(r[0], r[1], Long::sum);
        }
        return tally.values().stream().mapToLong(Long::longValue).max().orElse(0);
    }
}
