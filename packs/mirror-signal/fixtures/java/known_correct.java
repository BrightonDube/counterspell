// Test-only known-correct Java fixture for mirror-signal.
public class Solution {
    public static long mirror_signal(String signal) {
        long mismatches = 0;
        int n = signal.length();
        for (int left = 0; left < n / 2; left++) {
            if (signal.charAt(left) != signal.charAt(n - left - 1)) {
                mismatches += 1;
            }
        }
        return mismatches;
    }
}
