// Test-only known-correct Java fixture for bead-run.
public class Solution {
    public static long longest_bead(String strand) {
        long best = 0;
        long run = 0;
        char previous = 0;
        for (int i = 0; i < strand.length(); i += 1) {
            char bead = strand.charAt(i);
            run = bead == previous ? run + 1 : 1;
            if (run > best) {
                best = run;
            }
            previous = bead;
        }
        return best;
    }
}
