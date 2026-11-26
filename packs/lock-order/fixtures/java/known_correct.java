// Test-only known-correct Java fixture for lock-order.
public class Solution {
    public static long lock_order(long[] wards, long[] lens) {
        java.util.Set<String> seen = new java.util.HashSet<>();
        int at = 0;
        for (long count : lens) {
            long[] row = java.util.Arrays.copyOfRange(wards, at, at + (int) count);
            at += count;
            for (int i = 0; i < row.length; i++) {
                for (int j = i + 1; j < row.length; j++) {
                    long a = row[i], b = row[j];
                    if (a != b && seen.contains(b + ":" + a)) return 0;
                    seen.add(a + ":" + b);
                }
            }
        }
        return 1;
    }
}
