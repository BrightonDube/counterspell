// Test-only known-correct Java fixture for depend-rite.
public class Solution {
    public static long depend_rite(long[][] rows) {
        java.util.Map<Long, Long> seen = new java.util.HashMap<>();
        for (long[] r : rows) {
            if (seen.containsKey(r[0]) && seen.get(r[0]) != r[1]) {
                return 0;
            }
            seen.put(r[0], r[1]);
        }
        return 1;
    }
}
