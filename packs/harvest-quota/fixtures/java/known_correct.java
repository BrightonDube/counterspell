// Test-only known-correct Java fixture for harvest-quota.
public class Solution {
    public static long longest_harvest(long[] marks, long quota) {
        int left = 0;
        long total = 0;
        long best = 0;
        for (int right = 0; right < marks.length; right += 1) {
            total += marks[right];
            while (total > quota) {
                total -= marks[left];
                left += 1;
            }
            best = Math.max(best, right - left + 1);
        }
        return best;
    }
}
