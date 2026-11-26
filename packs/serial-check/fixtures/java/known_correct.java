// Test-only known-correct Java fixture for serial-check.
public class Solution {
    public static long serial_check(long[] events) {
        java.util.Set<Long> done = new java.util.HashSet<>();
        Long prev = null;
        for (long t : events) {
            if (done.contains(t)) {
                return 0;
            }
            if (prev == null || t != prev) {
                if (prev != null) {
                    done.add(prev);
                }
                prev = t;
            }
        }
        return 1;
    }
}
