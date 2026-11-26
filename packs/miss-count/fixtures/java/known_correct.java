// Test-only known-correct Java fixture for miss-count.
public class Solution {
    public static long miss_count(long[] marks, long c) {
        long[] slots = new long[(int) c];
        java.util.Arrays.fill(slots, -1);
        long misses = 0;
        for (long m : marks) {
            int line = (int) (m % c);
            long tag = m / c;
            if (slots[line] != tag) { misses += 1; slots[line] = tag; }
        }
        return misses;
    }
}
