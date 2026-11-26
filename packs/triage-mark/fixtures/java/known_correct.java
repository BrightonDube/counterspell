// Test-only known-correct Java fixture for triage-mark.
public class Solution {
    public static long next_mark(long[] prios) {
        long best = -1;
        for (int i = 0; i < prios.length; i += 1) {
            if (best < 0 || prios[i] < prios[(int) best]) {
                best = i;
            }
        }
        return best;
    }
}
