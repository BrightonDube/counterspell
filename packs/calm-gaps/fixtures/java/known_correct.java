// Test-only known-correct Java fixture for calm-gaps.
public class Solution {
    public static long calm_haul(long[] marks) {
        long take = 0;
        long skip = 0;
        for (long m : marks) {
            long t = skip + m;
            skip = Math.max(skip, take);
            take = t;
        }
        return Math.max(take, skip);
    }
}
