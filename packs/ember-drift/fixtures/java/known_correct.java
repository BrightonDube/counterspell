// Test-only known-correct Java fixture for ember-drift (language-drivers T13).
// Convention: a public static method on `Solution` whose parameter and return
// types are the driver's mapped Java types (list[int] -> long[]).
public class Solution {
    public static long[] drift_dark(long[] marks) {
        long[] drifted = new long[marks.length];
        int write = 0;
        for (long mark : marks) {
            if (mark != 0) {
                drifted[write] = mark;
                write += 1;
            }
        }
        return drifted;
    }
}
