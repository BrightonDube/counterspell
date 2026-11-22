// Test-only known-correct Java fixture for lantern-ledger.
public class Solution {
    public static long lantern_ledger(long[] changes, long start, long level) {
        long meter = start;
        long marks = 0;
        for (long change : changes) {
            meter += change;
            if (meter == level) {
                marks += 1;
            }
        }
        return marks;
    }
}
