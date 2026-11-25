// Test-only known-correct Java fixture for peak-torch.
public class Solution {
    public static long peak_torch(long[] heights) {
        long lo = 0, hi = heights.length - 1;
        while (lo < hi) {
            long mid = (lo + hi) / 2;
            if (heights[(int) mid] < heights[(int) mid + 1]) { lo = mid + 1; } else { hi = mid; }
        }
        return lo;
    }
}
