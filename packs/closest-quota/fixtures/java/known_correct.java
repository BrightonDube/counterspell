// Test-only known-correct Java fixture for closest-quota (language-drivers T13).
// Convention: a public static method on `Solution` whose parameter and return
// types are the driver's mapped Java types (list[int] -> long[], int -> long).
public class Solution {
    public static long nearest_pair(long[] marks, long quota) {
        if (marks.length < 2) return 0;
        int left = 0, right = marks.length - 1;
        long best = 0;
        boolean seen = false;
        while (left < right) {
            long total = marks[left] + marks[right];
            if (!seen
                    || Math.abs(total - quota) < Math.abs(best - quota)
                    || (Math.abs(total - quota) == Math.abs(best - quota) && total < best)) {
                best = total;
                seen = true;
            }
            if (total < quota) left += 1;
            else right -= 1;
        }
        return best;
    }
}
