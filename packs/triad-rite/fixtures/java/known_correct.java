// Test-only known-correct Java fixture for triad-rite (language-drivers T13).
// Convention: a public static method on `Solution` whose parameter and return
// types are the driver's mapped Java types (list[int] -> long[], int -> long).
public class Solution {
    public static long triads_at_quota(long[] marks, long quota) {
        long count = 0;
        int n = marks.length;
        for (int i = 0; i + 2 < n; i += 1) {
            long target = quota - marks[i];
            int left = i + 1, right = n - 1;
            while (left < right) {
                long pair = marks[left] + marks[right];
                if (pair < target) {
                    left += 1;
                } else if (pair > target) {
                    right -= 1;
                } else if (marks[left] == marks[right]) {
                    long span = right - left + 1;
                    count += span * (span - 1) / 2;
                    break;
                } else {
                    int lo = 1;
                    while (left + lo < right && marks[left + lo] == marks[left]) lo += 1;
                    int hi = 1;
                    while (right - hi > left && marks[right - hi] == marks[right]) hi += 1;
                    count += (long) lo * hi;
                    left += lo;
                    right -= hi;
                }
            }
        }
        return count;
    }
}
