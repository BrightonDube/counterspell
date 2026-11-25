// Test-only known-correct Java fixture for sigil-groups (language-drivers T13).
// Convention: a public static method on `Solution` whose parameter and return
// types are the driver's mapped Java types (list[int] -> long[], int -> long).
public class Solution {
    public static long largest_sigil(long[] markings) {
        long[] groups = new long[28];
        for (long marking : markings) {
            long sigil = 0;
            for (long rest = marking; rest > 0; rest /= 10) sigil += rest % 10;
            groups[(int) sigil] += 1;
        }
        long best = 0;
        for (long tally : groups) best = Math.max(best, tally);
        return best;
    }
}
